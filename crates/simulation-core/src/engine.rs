//! The simulation engine — ties together the scheduler, nodes, and metrics.
//!
//! This is a minimal scaffold. Full request lifecycle simulation arrives
//! on Day 5.

use crate::scheduler::Scheduler;
use crate::types::*;

/// The simulation engine.
pub struct Engine {
    scenario: Scenario,
    scheduler: Scheduler,
    metrics: Metrics,
    running: bool,
}

impl Engine {
    pub fn new(scenario: Scenario) -> Self {
        Self {
            scenario,
            scheduler: Scheduler::new(),
            metrics: Metrics::default(),
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
        self.metrics = Metrics::default();
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn now(&self) -> VirtualTime {
        self.scheduler.now()
    }

    pub fn metrics(&self) -> &Metrics {
        &self.metrics
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

    fn handle_event(&mut self, _time: VirtualTime, event: Event) {
        match event {
            Event::RequestCreated { .. } => {
                self.metrics.total_requests += 1;
            }
            Event::RequestCompleted { success, .. } => {
                if success {
                    self.metrics.successful += 1;
                } else {
                    self.metrics.failed += 1;
                }
            }
            Event::RequestTimedOut { .. } => {
                self.metrics.timed_out += 1;
            }
            Event::RetryScheduled { .. } => {
                self.metrics.retries += 1;
            }
            Event::MessageDropped { .. } => {
                self.metrics.dropped += 1;
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
            nodes: vec![Node {
                id: "client".into(),
                kind: ComponentKind::Client,
                name: "Client".into(),
                state: NodeState::Healthy,
                capacity: 100,
                latency_ms: 10,
                error_rate: 0.0,
                timeout_ms: 1000,
                queue_limit: None,
                cache_hit_rate: None,
            }],
            connections: vec![],
            traffic_start_rps: 10,
            traffic_target_rps: 100,
            traffic_ramp_seconds: 30,
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
        engine.metrics.total_requests = 10;
        engine.reset();
        assert!(!engine.is_running());
        assert_eq!(engine.metrics().total_requests, 0);
    }

    #[test]
    fn step_returns_false_when_empty() {
        let mut engine = Engine::new(simple_scenario());
        assert!(!engine.step());
    }
}
