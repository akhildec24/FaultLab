//! The simulation engine — ties together the scheduler, state, router,
//! traffic generator, and RNG to drive a full simulation.
//!
//! The engine owns the immutable `Scenario` (config) and the mutable
//! `SimulationState` (runtime). It processes events from the scheduler
//! and updates state accordingly.
//!
//! When `start()` is called, the engine schedules all traffic events.
//! Each `step()` processes one event and may schedule follow-on events
//! (arrivals, completions, timeouts, retries).

use crate::rng::Rng;
use crate::routing::{processing_time, retry_delay, should_fail, Router};
use crate::scheduler::Scheduler;
use crate::traffic::TrafficGenerator;
use crate::types::*;

/// The simulation engine.
pub struct Engine {
    scenario: Scenario,
    scheduler: Scheduler,
    state: SimulationState,
    router: Router,
    rng: Rng,
    running: bool,
    traffic_scheduled: bool,
    /// Recent events for external consumers (e.g. WASM UI).
    recent_events: Vec<(VirtualTime, Event)>,
    /// Max number of recent events to retain.
    recent_events_cap: usize,
}

impl Engine {
    pub fn new(scenario: Scenario) -> Self {
        let state = SimulationState::from_scenario(&scenario);
        let router = Router::from_connections(&scenario.connections);
        let rng = Rng::new(scenario.seed);
        Self {
            scenario,
            scheduler: Scheduler::new(),
            state,
            router,
            rng,
            running: false,
            traffic_scheduled: false,
            recent_events: Vec::new(),
            recent_events_cap: 256,
        }
    }

    pub fn load(scenario: Scenario) -> Self {
        Self::new(scenario)
    }

    /// Start the simulation. Schedules all traffic events.
    pub fn start(&mut self) {
        if !self.traffic_scheduled {
            self.schedule_traffic();
            self.traffic_scheduled = true;
        }
        self.running = true;
    }

    pub fn pause(&mut self) {
        self.running = false;
    }

    pub fn reset(&mut self) {
        self.scheduler = Scheduler::new();
        self.state = SimulationState::from_scenario(&self.scenario);
        self.rng = Rng::new(self.scenario.seed);
        self.running = false;
        self.traffic_scheduled = false;
        self.recent_events.clear();
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn now(&self) -> VirtualTime {
        self.scheduler.now()
    }

    pub fn metrics(&self) -> &Metrics {
        &self.state.metrics
    }

    pub fn state(&self) -> &SimulationState {
        &self.state
    }

    pub fn scenario(&self) -> &Scenario {
        &self.scenario
    }

    /// Recent events for external consumers (e.g. WASM UI).
    pub fn recent_events(&self) -> &[(VirtualTime, Event)] {
        &self.recent_events
    }

    /// Drain recent events (consume and return them, clearing the buffer).
    pub fn drain_recent_events(&mut self) -> Vec<(VirtualTime, Event)> {
        std::mem::take(&mut self.recent_events)
    }

    /// Number of pending events in the scheduler queue.
    pub fn pending(&self) -> usize {
        self.scheduler.pending()
    }

    /// Process the next pending event. Returns `false` when no events remain.
    pub fn step(&mut self) -> bool {
        match self.scheduler.next_event() {
            Some((time, event)) => {
                self.handle_event(time, event);
                true
            }
            None => false,
        }
    }

    /// Run until all events are processed, `max_steps` is reached,
    /// or the engine is paused. Returns the number of steps executed.
    pub fn run(&mut self, max_steps: usize) -> usize {
        let mut steps = 0;
        while steps < max_steps && self.running && self.step() {
            steps += 1;
        }
        steps
    }

    /// Inject a failure mid-simulation. This mutates runtime state
    /// (node health, connection state, capacity) and records the
    /// corresponding event in the event history.
    pub fn inject_failure(&mut self, failure: &FailureInjection) {
        let time = self.now();
        match failure {
            FailureInjection::Crash { node_id } => {
                if let Some(node) = self.state.nodes.get_mut(node_id) {
                    node.state = NodeState::Failed;
                }
                self.handle_event(
                    time,
                    Event::NodeFailed {
                        node_id: node_id.clone(),
                    },
                );
            }
            FailureInjection::Recover { node_id } => {
                if let Some(node) = self.state.nodes.get_mut(node_id) {
                    node.state = NodeState::Healthy;
                }
                self.handle_event(
                    time,
                    Event::NodeRecovered {
                        node_id: node_id.clone(),
                    },
                );
            }
            FailureInjection::AddLatency {
                node_id,
                latency_ms,
            } => {
                if let Some(node) = self.scenario.nodes.iter_mut().find(|n| &n.id == node_id) {
                    node.latency_ms += *latency_ms;
                }
            }
            FailureInjection::AddPacketLoss { from, to, rate } => {
                self.state.network.inject_packet_loss(from, to, *rate);
            }
            FailureInjection::Disconnect { from, to } => {
                self.state.network.disconnect(from, to);
                self.handle_event(
                    time,
                    Event::ConnectionFailed {
                        from: from.clone(),
                        to: to.clone(),
                    },
                );
            }
            FailureInjection::ReduceCapacity {
                node_id,
                new_capacity,
            } => {
                if let Some(node) = self.scenario.nodes.iter_mut().find(|n| &n.id == node_id) {
                    node.capacity = *new_capacity;
                }
            }
        }
    }

    fn schedule_traffic(&mut self) {
        let client_ids: Vec<String> = self
            .scenario
            .nodes
            .iter()
            .filter(|n| n.kind == ComponentKind::Client)
            .map(|n| n.id.clone())
            .collect();

        if client_ids.is_empty() {
            return;
        }

        let gen = TrafficGenerator::new(self.scenario.traffic.clone(), client_ids);
        let mut next_id = self.state.next_request_id;
        gen.schedule_all(&mut self.scheduler, &mut self.rng, &mut next_id);
        self.state.next_request_id = next_id;
    }

    fn handle_event(&mut self, time: VirtualTime, event: Event) {
        self.state.current_time = time;
        // Log event for recent_events buffer
        if self.recent_events.len() >= self.recent_events_cap {
            self.recent_events.remove(0);
        }
        self.recent_events.push((time, event.clone()));
        match event {
            Event::RequestCreated { request_id, origin } => {
                let req = RequestState::new(request_id, origin.clone(), time);
                self.state.requests.insert(request_id, req);
                self.state.metrics.total_requests += 1;

                // Route the request to the first downstream node
                let downstream = self.router.downstream(&origin).to_vec();
                if downstream.is_empty() {
                    // Client with no downstream — complete immediately
                    self.complete_request_success(request_id, time);
                } else {
                    for (dest_id, _) in &downstream {
                        if let Some(transit) = self.network_transit(&origin, dest_id, time) {
                            let arrival_time = time.add(transit);
                            self.scheduler.schedule(
                                arrival_time,
                                Event::RequestArrived {
                                    request_id,
                                    node_id: dest_id.clone(),
                                },
                            );
                        } else {
                            // Packet lost or connection down — mark as failed
                            self.complete_request_failed(request_id);
                        }
                    }
                }
            }

            Event::RequestArrived {
                request_id,
                node_id,
            } => {
                if let Some(req) = self.state.requests.get_mut(&request_id) {
                    req.current_node = Some(node_id.clone());
                    req.visited.push(node_id.clone());
                    req.phase = RequestPhase::Queued;
                }
                if let Some(node) = self.state.nodes.get_mut(&node_id) {
                    node.total_received += 1;

                    // Check capacity
                    let config = self.scenario.nodes.iter().find(|n| n.id == node_id);
                    let capacity = config.map(|c| c.capacity).unwrap_or(0);
                    if node.active_requests >= capacity {
                        // Over capacity — check queue
                        let queue_limit = config.and_then(|c| c.queue_limit).unwrap_or(0);
                        if node.queue_depth < queue_limit {
                            // Enqueue — add to waiting queue
                            node.queue_depth += 1;
                            if let Some(queue) = self.state.waiting_queues.get_mut(&node_id) {
                                queue.push_back(request_id);
                            } else {
                                let mut q = std::collections::VecDeque::new();
                                q.push_back(request_id);
                                self.state.waiting_queues.insert(node_id.clone(), q);
                            }
                            self.scheduler.schedule(
                                time,
                                Event::MessageQueued {
                                    request_id,
                                    queue_id: node_id.clone(),
                                },
                            );
                        } else {
                            // Queue full — apply shedding policy
                            let policy = config
                                .map(|c| c.shed_policy.clone())
                                .unwrap_or(SheddingPolicy::Drop);
                            node.total_shedded += 1;
                            self.scheduler.schedule(
                                time,
                                Event::RequestShedded {
                                    request_id,
                                    node_id: node_id.clone(),
                                    policy: policy.clone(),
                                },
                            );
                        }
                        return;
                    }
                }

                // Start processing
                self.start_processing(request_id, &node_id, time);
            }

            Event::RequestStarted {
                request_id,
                node_id,
            } => {
                // This event is handled inline by start_processing; nothing extra here.
                let _ = (request_id, node_id);
            }

            Event::RequestCompleted {
                request_id,
                node_id,
                success,
            } => {
                // Ignore stale completion if request already done (timed out)
                let phase = self.state.requests.get(&request_id).map(|r| r.phase);
                if phase == Some(RequestPhase::Done) || phase == Some(RequestPhase::PendingRetry) {
                    return;
                }

                // Decrement active requests and update node counters
                self.finish_processing(request_id, &node_id, success);

                if success {
                    if self.router.has_downstream(&node_id) {
                        // Forward to downstream nodes
                        let downstream = self.router.downstream(&node_id).to_vec();
                        let mut forwarded = false;
                        for (dest_id, _) in &downstream {
                            if let Some(transit) = self.network_transit(&node_id, dest_id, time) {
                                if let Some(req) = self.state.requests.get_mut(&request_id) {
                                    req.phase = RequestPhase::InTransit;
                                }
                                self.scheduler.schedule(
                                    time.add(transit),
                                    Event::RequestArrived {
                                        request_id,
                                        node_id: dest_id.clone(),
                                    },
                                );
                                forwarded = true;
                            }
                        }
                        if !forwarded {
                            // All downstream connections lost packets
                            self.complete_request_failed(request_id);
                        }
                    } else {
                        // Leaf node — request is fully done
                        self.complete_request_success(request_id, time);
                    }
                } else {
                    // Failure — try retry or mark done
                    self.maybe_retry(request_id, &node_id, time);
                }
            }

            Event::RequestTimedOut {
                request_id,
                node_id,
            } => {
                if let Some(req) = self.state.requests.get_mut(&request_id) {
                    // Only timeout if still processing at this node
                    if req.phase == RequestPhase::Processing {
                        req.phase = RequestPhase::PendingRetry;
                        req.outcome = Some(RequestOutcome::TimedOut);
                    } else {
                        return; // Already completed, ignore stale timeout
                    }
                }
                if let Some(node) = self.state.nodes.get_mut(&node_id) {
                    node.total_timed_out += 1;
                    node.active_requests = node.active_requests.saturating_sub(1);
                }

                self.try_dequeue(&node_id);
                self.maybe_retry(request_id, &node_id, time);
            }

            Event::RetryScheduled {
                request_id,
                node_id,
                retry_count,
            } => {
                if let Some(req) = self.state.requests.get_mut(&request_id) {
                    req.retry_count = retry_count;
                    req.phase = RequestPhase::InTransit;
                    req.outcome = None;
                }
                self.state.metrics.retries += 1;

                // Re-send to the same node
                let origin = self
                    .state
                    .requests
                    .get(&request_id)
                    .map(|r| r.origin.clone())
                    .unwrap_or_default();

                if let Some(transit) = self.network_transit(&origin, &node_id, time) {
                    self.scheduler.schedule(
                        time.add(transit),
                        Event::RequestArrived {
                            request_id,
                            node_id,
                        },
                    );
                } else {
                    // Packet lost or connection down during retry transit
                    self.complete_request_failed(request_id);
                }
            }

            Event::MessageQueued { .. } => {
                // Request is queued; will be dequeued when capacity frees up.
            }

            Event::MessageDropped {
                request_id,
                queue_id: _,
            } => {
                if let Some(req) = self.state.requests.get_mut(&request_id) {
                    req.phase = RequestPhase::Done;
                    req.outcome = Some(RequestOutcome::Dropped);
                }
                self.state.metrics.dropped += 1;
            }

            Event::RequestShedded {
                request_id,
                node_id,
                policy,
            } => {
                self.state.metrics.shedded += 1;
                match policy {
                    SheddingPolicy::Drop => {
                        // Silently drop the request
                        if let Some(req) = self.state.requests.get_mut(&request_id) {
                            req.phase = RequestPhase::Done;
                            req.outcome = Some(RequestOutcome::Dropped);
                        }
                    }
                    SheddingPolicy::Reject => {
                        // Reject with error — triggers retry if configured
                        if let Some(req) = self.state.requests.get_mut(&request_id) {
                            req.phase = RequestPhase::PendingRetry;
                            req.outcome = Some(RequestOutcome::Failed);
                        }
                        self.maybe_retry(request_id, &node_id, time);
                    }
                    SheddingPolicy::Backpressure => {
                        // Drop and record backpressure — in a real system
                        // this would signal upstream to slow down
                        if let Some(req) = self.state.requests.get_mut(&request_id) {
                            req.phase = RequestPhase::Done;
                            req.outcome = Some(RequestOutcome::Dropped);
                        }
                    }
                }
            }

            Event::RequestDequeued {
                request_id,
                node_id,
            } => {
                // A queued request now has capacity — start processing
                self.start_processing(request_id, &node_id, time);
            }

            Event::NodeFailed { node_id } => {
                if let Some(node) = self.state.nodes.get_mut(&node_id) {
                    node.state = NodeState::Failed;
                }
            }

            Event::NodeRecovered { node_id } => {
                if let Some(node) = self.state.nodes.get_mut(&node_id) {
                    node.state = NodeState::Healthy;
                }
            }

            Event::ConnectionFailed { from, to } => {
                self.state.network.disconnect(&from, &to);
            }

            Event::ConnectionRestored { from, to } => {
                self.state.network.reconnect(&from, &to);
            }
        }
    }

    /// Calculate transit time through the network model.
    /// Returns `None` if the connection is down or the packet is lost.
    fn network_transit(&mut self, from: &str, to: &str, time: VirtualTime) -> Option<u64> {
        let config = self.router.connection(from, to)?.clone();
        self.state
            .network
            .transit_time(from, to, &config, &mut self.rng, time.millis())
    }

    fn start_processing(&mut self, request_id: u64, node_id: &str, time: VirtualTime) {
        let config = match self.scenario.nodes.iter().find(|n| n.id == node_id) {
            Some(c) => c,
            None => {
                // Node doesn't exist — mark request as failed
                self.complete_request_failed(request_id);
                return;
            }
        };

        if let Some(req) = self.state.requests.get_mut(&request_id) {
            req.phase = RequestPhase::Processing;
        }
        if let Some(node) = self.state.nodes.get_mut(node_id) {
            node.active_requests += 1;
        }

        // Schedule completion
        let proc_time = processing_time(config, &mut self.rng);
        let success = !should_fail(config, &mut self.rng);

        // Schedule timeout
        let timeout_time = time.add(config.timeout_ms);
        self.scheduler.schedule(
            timeout_time,
            Event::RequestTimedOut {
                request_id,
                node_id: node_id.to_string(),
            },
        );

        // Schedule completion (may race with timeout — first one wins)
        let completion_time = time.add(proc_time);
        self.scheduler.schedule(
            completion_time,
            Event::RequestCompleted {
                request_id,
                node_id: node_id.to_string(),
                success,
            },
        );

        // Record hop latency
        if let Some(req) = self.state.requests.get_mut(&request_id) {
            req.hop_latencies.push(proc_time);
        }
    }

    /// Finish processing at an intermediate node — decrement active,
    /// update counters, but don't mark the request as Done.
    fn finish_processing(&mut self, _request_id: u64, node_id: &str, success: bool) {
        if let Some(node) = self.state.nodes.get_mut(node_id) {
            node.active_requests = node.active_requests.saturating_sub(1);
            if success {
                node.total_completed += 1;
            } else {
                node.total_failed += 1;
            }
        }
        // Try to dequeue a waiting request now that capacity freed up
        self.try_dequeue(node_id);
    }

    /// Try to dequeue a waiting request from a node's queue.
    /// Called when a request completes or times out, freeing capacity.
    fn try_dequeue(&mut self, node_id: &str) {
        // Check if there's capacity and a waiting queue
        let capacity = self
            .scenario
            .nodes
            .iter()
            .find(|n| n.id == node_id)
            .map(|c| c.capacity)
            .unwrap_or(0);

        let active = self
            .state
            .nodes
            .get(node_id)
            .map(|n| n.active_requests)
            .unwrap_or(0);

        if active >= capacity {
            return;
        }

        // Pop from waiting queue
        let dequeued = self
            .state
            .waiting_queues
            .get_mut(node_id)
            .and_then(|q| q.pop_front());

        if let Some(req_id) = dequeued {
            if let Some(node) = self.state.nodes.get_mut(node_id) {
                node.queue_depth = node.queue_depth.saturating_sub(1);
            }
            // Schedule dequeue event at current time
            self.scheduler.schedule(
                self.state.current_time,
                Event::RequestDequeued {
                    request_id: req_id,
                    node_id: node_id.to_string(),
                },
            );
        }
    }

    /// Mark a request as successfully completed (terminal state).
    fn complete_request_success(&mut self, request_id: u64, time: VirtualTime) {
        if let Some(req) = self.state.requests.get_mut(&request_id) {
            req.phase = RequestPhase::Done;
            req.outcome = Some(RequestOutcome::Success);
            let latency = time.millis().saturating_sub(req.created_at.millis());
            self.state.completed_latencies.push(latency);
        }
        self.state.metrics.successful += 1;
    }

    /// Mark a request as failed (terminal state, no more retries).
    fn complete_request_failed(&mut self, request_id: u64) {
        if let Some(req) = self.state.requests.get_mut(&request_id) {
            req.phase = RequestPhase::Done;
            if req.outcome.is_none() {
                req.outcome = Some(RequestOutcome::Failed);
            }
        }
        self.state.metrics.failed += 1;
    }

    fn maybe_retry(&mut self, request_id: u64, node_id: &str, time: VirtualTime) {
        let config = match self.scenario.nodes.iter().find(|n| n.id == node_id) {
            Some(c) => c,
            None => return,
        };

        let req = match self.state.requests.get(&request_id) {
            Some(r) => r,
            None => return,
        };

        let max_retries = config.retry_policy.max_retries;
        let budget_exhausted = config.retry_policy.budget.is_some()
            && self
                .state
                .nodes
                .get(node_id)
                .is_some_and(|n| n.retry_budget_remaining.is_some_and(|r| r == 0));

        if req.retry_count >= max_retries || budget_exhausted {
            // No more retries — mark as done with the right metric
            let was_timeout = req.outcome == Some(RequestOutcome::TimedOut);
            if let Some(req) = self.state.requests.get_mut(&request_id) {
                req.phase = RequestPhase::Done;
                if req.outcome.is_none() {
                    req.outcome = Some(RequestOutcome::Failed);
                }
            }
            if was_timeout {
                self.state.metrics.timed_out += 1;
            } else {
                self.state.metrics.failed += 1;
            }
            return;
        }

        // Decrement retry budget if configured
        if config.retry_policy.budget.is_some() {
            if let Some(node) = self.state.nodes.get_mut(node_id) {
                if let Some(remaining) = node.retry_budget_remaining {
                    node.retry_budget_remaining = Some(remaining - 1);
                }
            }
        }

        let retry_count = req.retry_count + 1;
        let delay = retry_delay(&config.retry_policy, retry_count, &mut self.rng);
        self.scheduler.schedule(
            time.add(delay),
            Event::RetryScheduled {
                request_id,
                node_id: node_id.to_string(),
                retry_count,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_node_scenario() -> Scenario {
        Scenario {
            name: "test".into(),
            nodes: vec![
                NodeConfig {
                    id: "client".into(),
                    kind: ComponentKind::Client,
                    name: "Client".into(),
                    capacity: 1000,
                    latency_ms: 5,
                    error_rate: 0.0,
                    timeout_ms: 5000,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy::default(),
                    shed_policy: SheddingPolicy::default(),
                },
                NodeConfig {
                    id: "svc".into(),
                    kind: ComponentKind::Service,
                    name: "Service".into(),
                    capacity: 100,
                    latency_ms: 20,
                    error_rate: 0.0,
                    timeout_ms: 1000,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy::default(),
                    shed_policy: SheddingPolicy::default(),
                },
            ],
            connections: vec![ConnectionConfig {
                from: "client".into(),
                to: "svc".into(),
                latency_ms: 10,
                packet_loss: 0.0,
                bandwidth_rps: 0,
            }],
            traffic: TrafficConfig {
                start_rps: 5,
                target_rps: 5,
                ramp_seconds: 0,
            },
            seed: 42,
        }
    }

    fn single_node_scenario() -> Scenario {
        Scenario {
            name: "test".into(),
            nodes: vec![NodeConfig {
                id: "client".into(),
                kind: ComponentKind::Client,
                name: "Client".into(),
                capacity: 100,
                latency_ms: 10,
                error_rate: 0.0,
                timeout_ms: 1000,
                queue_limit: None,
                cache_hit_rate: None,
                retry_policy: RetryPolicy::default(),
                shed_policy: SheddingPolicy::default(),
            }],
            connections: vec![],
            traffic: TrafficConfig {
                start_rps: 3,
                target_rps: 3,
                ramp_seconds: 0,
            },
            seed: 42,
        }
    }

    fn three_node_scenario() -> Scenario {
        Scenario {
            name: "client-service-db".into(),
            nodes: vec![
                NodeConfig {
                    id: "client".into(),
                    kind: ComponentKind::Client,
                    name: "Customer".into(),
                    capacity: 1000,
                    latency_ms: 5,
                    error_rate: 0.0,
                    timeout_ms: 5000,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy::default(),
                    shed_policy: SheddingPolicy::default(),
                },
                NodeConfig {
                    id: "checkout-api".into(),
                    kind: ComponentKind::Service,
                    name: "Checkout API".into(),
                    capacity: 200,
                    latency_ms: 20,
                    error_rate: 0.0,
                    timeout_ms: 1000,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy::default(),
                    shed_policy: SheddingPolicy::default(),
                },
                NodeConfig {
                    id: "orders-db".into(),
                    kind: ComponentKind::Database,
                    name: "Orders DB".into(),
                    capacity: 100,
                    latency_ms: 30,
                    error_rate: 0.0,
                    timeout_ms: 2000,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy::default(),
                    shed_policy: SheddingPolicy::default(),
                },
            ],
            connections: vec![
                ConnectionConfig {
                    from: "client".into(),
                    to: "checkout-api".into(),
                    latency_ms: 10,
                    packet_loss: 0.0,
                    bandwidth_rps: 0,
                },
                ConnectionConfig {
                    from: "checkout-api".into(),
                    to: "orders-db".into(),
                    latency_ms: 5,
                    packet_loss: 0.0,
                    bandwidth_rps: 0,
                },
            ],
            traffic: TrafficConfig {
                start_rps: 5,
                target_rps: 5,
                ramp_seconds: 0,
            },
            seed: 42,
        }
    }

    fn retry_storm_scenario() -> Scenario {
        Scenario {
            name: "retry-storm".into(),
            nodes: vec![
                NodeConfig {
                    id: "client".into(),
                    kind: ComponentKind::Client,
                    name: "Customer".into(),
                    capacity: 1000,
                    latency_ms: 5,
                    error_rate: 0.0,
                    timeout_ms: 5000,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy::default(),
                    shed_policy: SheddingPolicy::default(),
                },
                NodeConfig {
                    id: "checkout-api".into(),
                    kind: ComponentKind::Service,
                    name: "Checkout API".into(),
                    capacity: 120,
                    latency_ms: 40,
                    error_rate: 0.01,
                    timeout_ms: 800,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy {
                        strategy: RetryStrategy::Immediate,
                        max_retries: 3,
                        jitter: 0.0,
                        budget: None,
                    },
                    shed_policy: SheddingPolicy::default(),
                },
                NodeConfig {
                    id: "orders-db".into(),
                    kind: ComponentKind::Database,
                    name: "Orders DB".into(),
                    capacity: 80,
                    latency_ms: 25,
                    error_rate: 0.005,
                    timeout_ms: 2000,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy::default(),
                    shed_policy: SheddingPolicy::default(),
                },
            ],
            connections: vec![
                ConnectionConfig {
                    from: "client".into(),
                    to: "checkout-api".into(),
                    latency_ms: 10,
                    packet_loss: 0.0,
                    bandwidth_rps: 0,
                },
                ConnectionConfig {
                    from: "checkout-api".into(),
                    to: "orders-db".into(),
                    latency_ms: 10,
                    packet_loss: 0.0,
                    bandwidth_rps: 0,
                },
            ],
            traffic: TrafficConfig {
                start_rps: 20,
                target_rps: 500,
                ramp_seconds: 30,
            },
            seed: 42,
        }
    }

    #[test]
    fn engine_starts_and_pauses() {
        let mut engine = Engine::new(single_node_scenario());
        assert!(!engine.is_running());
        engine.start();
        assert!(engine.is_running());
        engine.pause();
        assert!(!engine.is_running());
    }

    #[test]
    fn reset_clears_state() {
        let mut engine = Engine::new(single_node_scenario());
        engine.start();
        engine.run(100);
        engine.reset();
        assert!(!engine.is_running());
        assert_eq!(engine.metrics().total_requests, 0);
        assert!(!engine.traffic_scheduled);
    }

    #[test]
    fn start_schedules_traffic() {
        let mut engine = Engine::new(single_node_scenario());
        assert!(engine.scheduler.is_empty());
        engine.start();
        assert!(!engine.scheduler.is_empty());
    }

    #[test]
    fn run_processes_all_events() {
        let mut engine = Engine::new(single_node_scenario());
        engine.start();
        let steps = engine.run(10000);
        assert!(steps > 0);
        // 3 rps for (0+30)s = 90 requests default duration
        assert!(engine.metrics().total_requests > 0);
        assert!(engine.scheduler.is_empty());
    }

    #[test]
    fn two_node_simulation_completes_requests() {
        let mut engine = Engine::new(two_node_scenario());
        engine.start();
        engine.run(100000);

        let m = engine.metrics();
        assert!(m.total_requests > 0, "should have generated requests");
        assert!(
            m.successful + m.failed + m.timed_out + m.dropped > 0,
            "should have completed some requests"
        );
    }

    #[test]
    fn two_node_simulation_visits_both_nodes() {
        let mut engine = Engine::new(two_node_scenario());
        engine.start();
        engine.run(100000);

        // At least one request should have visited the service
        let visited_svc = engine
            .state()
            .requests
            .values()
            .any(|r| r.visited.contains(&"svc".to_string()));
        assert!(visited_svc, "at least one request should visit the service");
    }

    #[test]
    fn deterministic_same_seed_same_results() {
        let scenario = two_node_scenario();

        let mut engine_a = Engine::new(scenario.clone());
        engine_a.start();
        engine_a.run(100000);

        let mut engine_b = Engine::new(scenario);
        engine_b.start();
        engine_b.run(100000);

        assert_eq!(
            engine_a.metrics().total_requests,
            engine_b.metrics().total_requests
        );
        assert_eq!(engine_a.metrics().successful, engine_b.metrics().successful);
        assert_eq!(engine_a.metrics().failed, engine_b.metrics().failed);
    }

    #[test]
    fn node_failed_updates_state() {
        let mut engine = Engine::new(single_node_scenario());
        engine.scheduler.schedule(
            VirtualTime(50),
            Event::NodeFailed {
                node_id: "client".into(),
            },
        );
        assert!(engine.step());
        assert_eq!(engine.state().nodes["client"].state, NodeState::Failed);
    }

    #[test]
    fn request_created_adds_to_state() {
        let mut engine = Engine::new(single_node_scenario());
        engine.scheduler.schedule(
            VirtualTime(100),
            Event::RequestCreated {
                request_id: 1,
                origin: "client".into(),
            },
        );
        assert!(engine.step());
        assert_eq!(engine.metrics().total_requests, 1);
        assert!(engine.state().requests.contains_key(&1));
    }

    // --- Day 5: End-to-end request lifecycle tests ---

    #[test]
    fn three_node_simulation_visits_all_nodes() {
        let mut engine = Engine::new(three_node_scenario());
        engine.start();
        engine.run(100000);

        // At least one request should have visited both service and database
        let visited_both = engine.state().requests.values().any(|r| {
            r.visited.contains(&"checkout-api".to_string())
                && r.visited.contains(&"orders-db".to_string())
        });
        assert!(
            visited_both,
            "at least one request should visit both service and database"
        );
    }

    #[test]
    fn three_node_simulation_completes_all_requests() {
        let mut engine = Engine::new(three_node_scenario());
        engine.start();
        engine.run(100000);

        let m = engine.metrics();
        assert!(m.total_requests > 0, "should have generated requests");

        // Every request should be in a terminal state
        let all_done = engine
            .state()
            .requests
            .values()
            .all(|r| r.phase == RequestPhase::Done);
        assert!(all_done, "all requests should be Done");

        // successful + failed + timed_out + dropped should equal total
        let accounted = m.successful + m.failed + m.timed_out + m.dropped;
        assert_eq!(
            accounted, m.total_requests,
            "all requests should be accounted for"
        );
    }

    #[test]
    fn three_node_simulation_records_latency() {
        let mut engine = Engine::new(three_node_scenario());
        engine.start();
        engine.run(100000);

        // Completed latencies should be non-empty
        assert!(
            !engine.state().completed_latencies.is_empty(),
            "should have recorded latencies"
        );

        // Every successful request should have hop latencies
        let successful_with_hops = engine
            .state()
            .requests
            .values()
            .filter(|r| r.outcome == Some(RequestOutcome::Success))
            .filter(|r| !r.hop_latencies.is_empty())
            .count();
        assert!(
            successful_with_hops > 0,
            "successful requests should have hop latencies"
        );

        // Latency should be at least: transit(10) + svc(20) + transit(5) + db(30) = 65ms
        // (with jitter, could be slightly less, but not dramatically)
        let min_expected = 50;
        let any_reasonable = engine
            .state()
            .completed_latencies
            .iter()
            .any(|&l| l >= min_expected);
        assert!(
            any_reasonable,
            "at least one latency should be >= {min_expected}ms"
        );
    }

    #[test]
    fn three_node_simulation_no_timeouts_with_healthy_config() {
        let mut engine = Engine::new(three_node_scenario());
        engine.start();
        engine.run(100000);

        let m = engine.metrics();
        // With zero error rates and generous timeouts, nothing should time out
        assert_eq!(m.timed_out, 0, "no timeouts expected with healthy config");
        assert_eq!(m.failed, 0, "no failures expected with zero error rate");
        assert_eq!(m.dropped, 0, "no drops expected with sufficient capacity");
        assert_eq!(
            m.successful, m.total_requests,
            "all requests should succeed"
        );
    }

    #[test]
    fn three_node_deterministic_replay() {
        let scenario = three_node_scenario();

        let mut engine_a = Engine::new(scenario.clone());
        engine_a.start();
        engine_a.run(100000);

        let mut engine_b = Engine::new(scenario);
        engine_b.start();
        engine_b.run(100000);

        assert_eq!(
            engine_a.metrics().total_requests,
            engine_b.metrics().total_requests
        );
        assert_eq!(engine_a.metrics().successful, engine_b.metrics().successful);
        assert_eq!(engine_a.metrics().failed, engine_b.metrics().failed);
        assert_eq!(
            engine_a.state().completed_latencies,
            engine_b.state().completed_latencies,
            "latency vectors should match exactly"
        );
    }

    #[test]
    fn retry_storm_scenario_runs_to_completion() {
        let mut engine = Engine::new(retry_storm_scenario());
        engine.start();
        engine.run(500000);

        let m = engine.metrics();
        assert!(
            m.total_requests > 100,
            "retry storm should generate substantial traffic, got {}",
            m.total_requests
        );

        // With error rates > 0, we expect some failures or retries
        let accounted = m.successful + m.failed + m.timed_out + m.dropped;
        assert_eq!(
            accounted, m.total_requests,
            "all requests should be accounted for"
        );

        // All requests should be in terminal state
        let all_done = engine
            .state()
            .requests
            .values()
            .all(|r| r.phase == RequestPhase::Done);
        assert!(all_done, "all requests should be Done after retry storm");

        // The scheduler should be empty
        assert!(engine.scheduler.is_empty(), "scheduler should be drained");
    }

    #[test]
    fn retry_storm_deterministic_replay() {
        let scenario = retry_storm_scenario();

        let mut engine_a = Engine::new(scenario.clone());
        engine_a.start();
        engine_a.run(500000);

        let mut engine_b = Engine::new(scenario);
        engine_b.start();
        engine_b.run(500000);

        assert_eq!(
            engine_a.metrics().total_requests,
            engine_b.metrics().total_requests
        );
        assert_eq!(engine_a.metrics().successful, engine_b.metrics().successful);
        assert_eq!(engine_a.metrics().retries, engine_b.metrics().retries);
        assert_eq!(engine_a.metrics().timed_out, engine_b.metrics().timed_out);
    }

    // --- Day 6: Network model tests ---

    #[test]
    fn connection_failed_disconnects() {
        let mut engine = Engine::new(three_node_scenario());
        engine.scheduler.schedule(
            VirtualTime(0),
            Event::ConnectionFailed {
                from: "client".into(),
                to: "checkout-api".into(),
            },
        );
        engine.step();
        assert!(!engine
            .state()
            .network
            .is_connected("client", "checkout-api"));
    }

    #[test]
    fn connection_restored_reconnects() {
        let mut engine = Engine::new(three_node_scenario());
        engine.scheduler.schedule(
            VirtualTime(0),
            Event::ConnectionFailed {
                from: "client".into(),
                to: "checkout-api".into(),
            },
        );
        engine.step();
        engine.scheduler.schedule(
            VirtualTime(100),
            Event::ConnectionRestored {
                from: "client".into(),
                to: "checkout-api".into(),
            },
        );
        engine.step();
        assert!(engine
            .state()
            .network
            .is_connected("client", "checkout-api"));
    }

    #[test]
    fn disconnected_connection_drops_requests() {
        let mut engine = Engine::new(three_node_scenario());

        // Disconnect before starting
        engine.scheduler.schedule(
            VirtualTime(0),
            Event::ConnectionFailed {
                from: "client".into(),
                to: "checkout-api".into(),
            },
        );
        engine.step();

        // Now schedule a single request
        engine.scheduler.schedule(
            VirtualTime(1),
            Event::RequestCreated {
                request_id: 999,
                origin: "client".into(),
            },
        );
        engine.step();

        // Request should be failed since connection is down
        let req = &engine.state().requests[&999];
        assert_eq!(req.phase, RequestPhase::Done);
        assert_eq!(req.outcome, Some(RequestOutcome::Failed));
    }

    #[test]
    fn bandwidth_limit_drops_excess_requests() {
        let scenario = Scenario {
            name: "bandwidth-test".into(),
            nodes: vec![
                NodeConfig {
                    id: "client".into(),
                    kind: ComponentKind::Client,
                    name: "Client".into(),
                    capacity: 1000,
                    latency_ms: 5,
                    error_rate: 0.0,
                    timeout_ms: 5000,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy::default(),
                    shed_policy: SheddingPolicy::default(),
                },
                NodeConfig {
                    id: "svc".into(),
                    kind: ComponentKind::Service,
                    name: "Service".into(),
                    capacity: 1000,
                    latency_ms: 20,
                    error_rate: 0.0,
                    timeout_ms: 5000,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy::default(),
                    shed_policy: SheddingPolicy::default(),
                },
            ],
            connections: vec![ConnectionConfig {
                from: "client".into(),
                to: "svc".into(),
                latency_ms: 10,
                packet_loss: 0.0,
                bandwidth_rps: 3, // Only 3 requests per second
            }],
            traffic: TrafficConfig {
                start_rps: 10,
                target_rps: 10,
                ramp_seconds: 0,
            },
            seed: 42,
        };

        let mut engine = Engine::new(scenario);
        engine.start();
        engine.run(100000);

        // With 10 rps but bandwidth limited to 3 rps, some requests should fail
        let m = engine.metrics();
        assert!(m.failed > 0, "bandwidth limit should cause some failures");
    }

    #[test]
    fn network_state_initialised_from_scenario() {
        let engine = Engine::new(three_node_scenario());
        // Should have 2 connections
        assert!(engine
            .state()
            .network
            .is_connected("client", "checkout-api"));
        assert!(engine
            .state()
            .network
            .is_connected("checkout-api", "orders-db"));
    }

    #[test]
    fn network_injected_latency_increases_transit() {
        let mut engine = Engine::new(three_node_scenario());

        // Inject 500ms latency on client → checkout-api
        engine
            .state
            .network
            .inject_latency("client", "checkout-api", 500);

        // Schedule a request and check it takes longer
        engine.scheduler.schedule(
            VirtualTime(0),
            Event::RequestCreated {
                request_id: 1,
                origin: "client".into(),
            },
        );
        engine.step();

        // The RequestArrived event should be at ~510ms (10 base + 500 injected)
        // Check the next event time
        let next = engine.scheduler.peek();
        assert!(next.is_some());
        let (t, _) = next.unwrap();
        assert!(
            t.millis() >= 450 && t.millis() <= 560,
            "transit time should include injected latency, got {}ms",
            t.millis()
        );
    }

    #[test]
    fn inject_crash_sets_node_failed_and_emits_event() {
        let mut engine = Engine::new(three_node_scenario());
        engine.inject_failure(&FailureInjection::Crash {
            node_id: "checkout-api".into(),
        });
        assert_eq!(
            engine.state().nodes.get("checkout-api").unwrap().state,
            NodeState::Failed
        );
        // Should have a NodeFailed event in recent_events
        let events = engine.recent_events();
        assert!(events.iter().any(|(_, e)| matches!(
            e,
            Event::NodeFailed { node_id } if node_id == "checkout-api"
        )));
    }

    #[test]
    fn inject_recover_sets_node_healthy_and_emits_event() {
        let mut engine = Engine::new(three_node_scenario());
        engine.inject_failure(&FailureInjection::Crash {
            node_id: "checkout-api".into(),
        });
        engine.inject_failure(&FailureInjection::Recover {
            node_id: "checkout-api".into(),
        });
        assert_eq!(
            engine.state().nodes.get("checkout-api").unwrap().state,
            NodeState::Healthy
        );
        let events = engine.recent_events();
        assert!(events.iter().any(|(_, e)| matches!(
            e,
            Event::NodeRecovered { node_id } if node_id == "checkout-api"
        )));
    }

    #[test]
    fn inject_disconnect_breaks_connection_and_emits_event() {
        let mut engine = Engine::new(three_node_scenario());
        engine.inject_failure(&FailureInjection::Disconnect {
            from: "client".into(),
            to: "checkout-api".into(),
        });
        assert!(!engine
            .state()
            .network
            .is_connected("client", "checkout-api"));
        let events = engine.recent_events();
        assert!(events.iter().any(|(_, e)| matches!(
            e,
            Event::ConnectionFailed { from, to } if from == "client" && to == "checkout-api"
        )));
    }

    #[test]
    fn inject_reduce_capacity_changes_scenario() {
        let mut engine = Engine::new(three_node_scenario());
        engine.inject_failure(&FailureInjection::ReduceCapacity {
            node_id: "checkout-api".into(),
            new_capacity: 1,
        });
        let cap = engine
            .scenario()
            .nodes
            .iter()
            .find(|n| n.id == "checkout-api")
            .unwrap()
            .capacity;
        assert_eq!(cap, 1);
    }

    #[test]
    fn inject_add_latency_affects_node_latency() {
        let mut engine = Engine::new(three_node_scenario());
        engine.inject_failure(&FailureInjection::AddLatency {
            node_id: "checkout-api".into(),
            latency_ms: 500,
        });
        let latency = engine
            .scenario()
            .nodes
            .iter()
            .find(|n| n.id == "checkout-api")
            .unwrap()
            .latency_ms;
        assert_eq!(
            latency,
            20 + 500,
            "node latency should include injected amount"
        );
    }

    #[test]
    fn inject_add_packet_loss_increases_loss_rate() {
        let mut engine = Engine::new(three_node_scenario());
        engine.inject_failure(&FailureInjection::AddPacketLoss {
            from: "client".into(),
            to: "checkout-api".into(),
            rate: 0.9,
        });
        // With 90% injected loss, most requests should fail
        engine.start();
        engine.run(100);
        let m = engine.metrics();
        assert!(
            m.failed > 0 || m.dropped > 0,
            "high packet loss should cause failures"
        );
    }

    fn aggressive_retry_scenario() -> Scenario {
        Scenario {
            name: "aggressive-retry".into(),
            nodes: vec![
                NodeConfig {
                    id: "client".into(),
                    kind: ComponentKind::Client,
                    name: "Client".into(),
                    capacity: 1000,
                    latency_ms: 5,
                    error_rate: 0.0,
                    timeout_ms: 2000,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy {
                        strategy: RetryStrategy::Immediate,
                        max_retries: 10,
                        jitter: 0.0,
                        budget: None,
                    },
                    shed_policy: SheddingPolicy::default(),
                },
                NodeConfig {
                    id: "svc".into(),
                    kind: ComponentKind::Service,
                    name: "Service".into(),
                    capacity: 20,
                    latency_ms: 50,
                    error_rate: 0.9,
                    timeout_ms: 500,
                    queue_limit: Some(50),
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy {
                        strategy: RetryStrategy::Immediate,
                        max_retries: 3,
                        jitter: 0.0,
                        budget: None,
                    },
                    shed_policy: SheddingPolicy::default(),
                },
            ],
            connections: vec![ConnectionConfig {
                from: "client".into(),
                to: "svc".into(),
                latency_ms: 10,
                packet_loss: 0.0,
                bandwidth_rps: 0,
            }],
            traffic: TrafficConfig {
                start_rps: 10,
                target_rps: 50,
                ramp_seconds: 5,
            },
            seed: 42,
        }
    }

    #[test]
    fn retry_storm_produces_many_retries() {
        let mut engine = Engine::new(aggressive_retry_scenario());
        engine.start();
        engine.run(500);
        let m = engine.metrics();
        // With 50% error rate and 10 max retries, we should see
        // significantly more retries than total requests
        assert!(m.retries > 0, "should have retries with 50% error rate");
        assert!(
            m.failed > 0 || m.timed_out > 0,
            "should have failures when retries are exhausted"
        );
    }

    #[test]
    fn retry_budget_limits_total_retries() {
        let scenario = Scenario {
            name: "budget-test".into(),
            nodes: vec![
                NodeConfig {
                    id: "client".into(),
                    kind: ComponentKind::Client,
                    name: "Client".into(),
                    capacity: 1000,
                    latency_ms: 5,
                    error_rate: 0.0,
                    timeout_ms: 5000,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy::default(),
                    shed_policy: SheddingPolicy::default(),
                },
                NodeConfig {
                    id: "svc".into(),
                    kind: ComponentKind::Service,
                    name: "Service".into(),
                    capacity: 100,
                    latency_ms: 20,
                    error_rate: 0.8,
                    timeout_ms: 1000,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy {
                        strategy: RetryStrategy::Immediate,
                        max_retries: 10,
                        jitter: 0.0,
                        budget: Some(5),
                    },
                    shed_policy: SheddingPolicy::default(),
                },
            ],
            connections: vec![ConnectionConfig {
                from: "client".into(),
                to: "svc".into(),
                latency_ms: 10,
                packet_loss: 0.0,
                bandwidth_rps: 0,
            }],
            traffic: TrafficConfig {
                start_rps: 5,
                target_rps: 5,
                ramp_seconds: 0,
            },
            seed: 42,
        };

        let mut engine = Engine::new(scenario);
        engine.start();
        engine.run(200);
        let m = engine.metrics();
        // With budget=5, total retries should not exceed 5
        assert!(
            m.retries <= 5,
            "retry budget should limit total retries, got {}",
            m.retries
        );
    }

    #[test]
    fn exponential_backoff_produces_increasing_delays() {
        let policy = RetryPolicy {
            strategy: RetryStrategy::Exponential {
                base_ms: 100,
                max_delay_ms: 10000,
            },
            max_retries: 5,
            jitter: 0.0,
            budget: None,
        };
        let mut rng = Rng::new(42);
        let d0 = retry_delay(&policy, 0, &mut rng);
        let d1 = retry_delay(&policy, 1, &mut rng);
        let d2 = retry_delay(&policy, 2, &mut rng);
        // Exponential: 100*1=100, 100*2=200, 100*4=400
        assert_eq!(d0, 100);
        assert_eq!(d1, 200);
        assert_eq!(d2, 400);
    }

    #[test]
    fn zero_max_retries_means_no_retry() {
        let scenario = Scenario {
            name: "no-retry".into(),
            nodes: vec![
                NodeConfig {
                    id: "client".into(),
                    kind: ComponentKind::Client,
                    name: "Client".into(),
                    capacity: 1000,
                    latency_ms: 5,
                    error_rate: 0.0,
                    timeout_ms: 5000,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy {
                        strategy: RetryStrategy::Immediate,
                        max_retries: 0,
                        jitter: 0.0,
                        budget: None,
                    },
                    shed_policy: SheddingPolicy::default(),
                },
                NodeConfig {
                    id: "svc".into(),
                    kind: ComponentKind::Service,
                    name: "Service".into(),
                    capacity: 100,
                    latency_ms: 20,
                    error_rate: 1.0,
                    timeout_ms: 1000,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy {
                        strategy: RetryStrategy::Immediate,
                        max_retries: 0,
                        jitter: 0.0,
                        budget: None,
                    },
                    shed_policy: SheddingPolicy::default(),
                },
            ],
            connections: vec![ConnectionConfig {
                from: "client".into(),
                to: "svc".into(),
                latency_ms: 10,
                packet_loss: 0.0,
                bandwidth_rps: 0,
            }],
            traffic: TrafficConfig {
                start_rps: 5,
                target_rps: 5,
                ramp_seconds: 0,
            },
            seed: 42,
        };

        let mut engine = Engine::new(scenario);
        engine.start();
        engine.run(100);
        let m = engine.metrics();
        assert_eq!(m.retries, 0, "no retries should occur with max_retries=0");
        assert!(
            m.failed > 0,
            "requests should fail with 100% error rate and no retries"
        );
    }

    fn queue_scenario(shed_policy: SheddingPolicy) -> Scenario {
        Scenario {
            name: "queue-test".into(),
            nodes: vec![
                NodeConfig {
                    id: "client".into(),
                    kind: ComponentKind::Client,
                    name: "Client".into(),
                    capacity: 1000,
                    latency_ms: 5,
                    error_rate: 0.0,
                    timeout_ms: 5000,
                    queue_limit: None,
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy::default(),
                    shed_policy: SheddingPolicy::default(),
                },
                NodeConfig {
                    id: "svc".into(),
                    kind: ComponentKind::Service,
                    name: "Service".into(),
                    capacity: 1,
                    latency_ms: 50,
                    error_rate: 0.0,
                    timeout_ms: 5000,
                    queue_limit: Some(5),
                    cache_hit_rate: None,
                    retry_policy: RetryPolicy::default(),
                    shed_policy,
                },
            ],
            connections: vec![ConnectionConfig {
                from: "client".into(),
                to: "svc".into(),
                latency_ms: 10,
                packet_loss: 0.0,
                bandwidth_rps: 0,
            }],
            traffic: TrafficConfig {
                start_rps: 50,
                target_rps: 50,
                ramp_seconds: 0,
            },
            seed: 42,
        }
    }

    #[test]
    fn queue_dequeues_when_capacity_frees() {
        let mut engine = Engine::new(queue_scenario(SheddingPolicy::Drop));
        engine.start();
        engine.run(2000);
        let m = engine.metrics();
        assert!(m.successful > 0, "dequeued requests should succeed");
        assert!(
            m.shedded > 0,
            "excess requests beyond queue should be shedded"
        );
    }

    #[test]
    fn shed_policy_drop_silently_drops() {
        let mut engine = Engine::new(queue_scenario(SheddingPolicy::Drop));
        engine.start();
        engine.run(500);
        let m = engine.metrics();
        assert!(m.shedded > 0, "Drop policy should shed requests");
        assert_eq!(m.retries, 0, "Drop policy should not trigger retries");
    }

    #[test]
    fn shed_policy_reject_triggers_retry() {
        let mut engine = Engine::new(queue_scenario(SheddingPolicy::Reject));
        engine.start();
        engine.run(500);
        let m = engine.metrics();
        assert!(m.shedded > 0, "Reject policy should shed requests");
        assert!(m.retries > 0, "Reject policy should trigger retries");
    }

    #[test]
    fn shed_policy_backpressure_drops_without_retry() {
        let mut engine = Engine::new(queue_scenario(SheddingPolicy::Backpressure));
        engine.start();
        engine.run(500);
        let m = engine.metrics();
        assert!(m.shedded > 0, "Backpressure policy should shed requests");
        assert_eq!(
            m.retries, 0,
            "Backpressure policy should not trigger retries"
        );
    }
}
