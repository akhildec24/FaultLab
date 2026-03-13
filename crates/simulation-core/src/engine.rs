//! The simulation engine — ties together the scheduler, state, and metrics.
//!
//! The engine owns the immutable `Scenario` (config) and the mutable
//! `SimulationState` (runtime). It processes events from the scheduler
//! and updates state accordingly.
//!
//! Full request lifecycle simulation arrives on Day 5.

use crate::scheduler::Scheduler;
use crate::types::*;

/// The simulation engine.
pub struct Engine {
    scenario: Scenario,
    scheduler: Scheduler,
    state: SimulationState,
    running: bool,
}

impl Engine {
    pub fn new(scenario: Scenario) -> Self {
        let state = SimulationState::from_scenario(&scenario);
        Self {
            scenario,
            scheduler: Scheduler::new(),
            state,
            running: false,
        }
    }

    pub fn load(scenario: Scenario) -> Self {
        Self::new(scenario)
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn pause(&mut self) {
        self.running = false;
    }

    pub fn reset(&mut self) {
        self.scheduler = Scheduler::new();
        self.state = SimulationState::from_scenario(&self.scenario);
        self.running = false;
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

    fn handle_event(&mut self, time: VirtualTime, event: Event) {
        self.state.current_time = time;
        match event {
            Event::RequestCreated { request_id, origin } => {
                let req = RequestState::new(request_id, origin, time);
                self.state.requests.insert(request_id, req);
                self.state.metrics.total_requests += 1;
            }
            Event::RequestCompleted {
                request_id,
                node_id,
                success,
            } => {
                if let Some(req) = self.state.requests.get_mut(&request_id) {
                    req.phase = RequestPhase::Done;
                    req.outcome = Some(if success {
                        RequestOutcome::Success
                    } else {
                        RequestOutcome::Failed
                    });
                    let latency = req.total_latency();
                    self.state.completed_latencies.push(latency);
                }
                if let Some(node) = self.state.nodes.get_mut(&node_id) {
                    node.active_requests = node.active_requests.saturating_sub(1);
                    if success {
                        node.total_completed += 1;
                        self.state.metrics.successful += 1;
                    } else {
                        node.total_failed += 1;
                        self.state.metrics.failed += 1;
                    }
                }
            }
            Event::RequestTimedOut {
                request_id,
                node_id,
            } => {
                if let Some(req) = self.state.requests.get_mut(&request_id) {
                    req.phase = RequestPhase::Done;
                    req.outcome = Some(RequestOutcome::TimedOut);
                }
                if let Some(node) = self.state.nodes.get_mut(&node_id) {
                    node.total_timed_out += 1;
                    node.active_requests = node.active_requests.saturating_sub(1);
                }
                self.state.metrics.timed_out += 1;
            }
            Event::RetryScheduled { .. } => {
                self.state.metrics.retries += 1;
            }
            Event::MessageDropped { .. } => {
                self.state.metrics.dropped += 1;
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
            _ => {}
        }
    }

    pub fn scenario(&self) -> &Scenario {
        &self.scenario
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_scenario() -> Scenario {
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
            }],
            connections: vec![],
            traffic: TrafficConfig {
                start_rps: 10,
                target_rps: 100,
                ramp_seconds: 30,
            },
            seed: 42,
        }
    }

    #[test]
    fn engine_starts_and_pauses() {
        let mut engine = Engine::new(simple_scenario());
        assert!(!engine.is_running());
        engine.start();
        assert!(engine.is_running());
        engine.pause();
        assert!(!engine.is_running());
    }

    #[test]
    fn reset_clears_state() {
        let mut engine = Engine::new(simple_scenario());
        engine.start();
        engine.state.metrics.total_requests = 10;
        engine.reset();
        assert!(!engine.is_running());
        assert_eq!(engine.metrics().total_requests, 0);
    }

    #[test]
    fn step_returns_false_when_empty() {
        let mut engine = Engine::new(simple_scenario());
        assert!(!engine.step());
    }

    #[test]
    fn request_created_adds_to_state() {
        let mut engine = Engine::new(simple_scenario());
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
        let req = &engine.state().requests[&1];
        assert_eq!(req.phase, RequestPhase::InTransit);
    }

    #[test]
    fn node_failed_updates_state() {
        let mut engine = Engine::new(simple_scenario());
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
    fn request_completed_records_latency() {
        let mut engine = Engine::new(simple_scenario());

        engine.scheduler.schedule(
            VirtualTime(0),
            Event::RequestCreated {
                request_id: 1,
                origin: "client".into(),
            },
        );
        engine.scheduler.schedule(
            VirtualTime(100),
            Event::RequestCompleted {
                request_id: 1,
                node_id: "client".into(),
                success: true,
            },
        );

        engine.step();
        engine.step();

        assert_eq!(engine.metrics().successful, 1);
        assert_eq!(
            engine.state().requests[&1].outcome,
            Some(RequestOutcome::Success)
        );
        assert_eq!(engine.state().requests[&1].phase, RequestPhase::Done);
    }
}
