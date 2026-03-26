//! Core domain types for the simulation engine.
//!
//! The type system is split into two layers:
//!
//! - **Config** types (`Scenario`, `NodeConfig`, `ConnectionConfig`,
//!   `RetryPolicy`, `TrafficConfig`) are immutable during a simulation
//!   run. They describe the architecture and parameters.
//!
//! - **Runtime** types (`RequestState`, `NodeRuntimeState`, `SimulationState`,
//!   `RequestLifecycle`) hold mutable data that changes as events are
//!   processed.
//!
//! This separation makes it trivial to reset a simulation (discard runtime
//! state, keep config) and to run the same scenario with different seeds.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

// ---------------------------------------------------------------------------
// Virtual time
// ---------------------------------------------------------------------------

/// Virtual simulation time in milliseconds.
///
/// The simulation advances its own virtual clock rather than depending on
/// real time. This enables deterministic replay and faster-than-real-time
/// execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VirtualTime(pub u64);

impl VirtualTime {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn millis(&self) -> u64 {
        self.0
    }

    pub fn add(&self, ms: u64) -> Self {
        Self(self.0 + ms)
    }

    pub fn seconds(&self) -> f64 {
        self.0 as f64 / 1000.0
    }
}

impl Default for VirtualTime {
    fn default() -> Self {
        Self::zero()
    }
}

impl std::fmt::Display for VirtualTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let secs = self.0 / 1000;
        let ms = self.0 % 1000;
        write!(f, "{:03}.{:03}", secs, ms)
    }
}

// ---------------------------------------------------------------------------
// Config types (immutable during a run)
// ---------------------------------------------------------------------------

/// The six initial component types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentKind {
    Client,
    Service,
    Queue,
    Cache,
    Database,
    ExternalApi,
}

/// Retry strategy for a service or client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryStrategy {
    /// Retry immediately with no delay.
    Immediate,
    /// Wait a fixed delay between retries.
    Fixed { delay_ms: u64 },
    /// Exponential backoff: delay = base * 2^attempt, capped at max_delay.
    Exponential { base_ms: u64, max_delay_ms: u64 },
}

/// Retry policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub strategy: RetryStrategy,
    pub max_retries: u32,
    /// Jitter fraction (0.0 = none, 0.3 = ±30%).
    pub jitter: f64,
    /// Maximum total retries across all in-flight requests (retry budget).
    pub budget: Option<u32>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            strategy: RetryStrategy::Immediate,
            max_retries: 3,
            jitter: 0.0,
            budget: None,
        }
    }
}

/// Load shedding policy when a node's queue is full.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SheddingPolicy {
    /// Drop the incoming request silently (default).
    #[default]
    Drop,
    /// Reject with an error — triggers retry logic if configured.
    Reject,
    /// Apply backpressure — tell upstream to slow down.
    /// In practice this drops the request and records a backpressure event.
    Backpressure,
}

/// Replication role for database nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReplicationRole {
    /// Standalone node — no replication (default).
    #[default]
    Standalone,
    /// Primary node — accepts writes and forwards to replicas.
    Leader,
    /// Replica node — receives replicated writes with a delay.
    Replica,
}

/// Immutable configuration for a node in the architecture graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub id: String,
    pub kind: ComponentKind,
    pub name: String,
    pub capacity: u32,
    pub latency_ms: u64,
    pub error_rate: f64,
    pub timeout_ms: u64,
    pub queue_limit: Option<u32>,
    pub cache_hit_rate: Option<f64>,
    /// Replication role for database nodes (leader/replica/standalone).
    #[serde(default)]
    pub replication_role: ReplicationRole,
    /// Replication lag in ms — delay before writes appear on replicas.
    #[serde(default)]
    pub replication_lag_ms: u64,
    #[serde(default)]
    pub retry_policy: RetryPolicy,
    #[serde(default)]
    pub shed_policy: SheddingPolicy,
}

/// Immutable configuration for a directed connection between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub from: String,
    pub to: String,
    pub latency_ms: u64,
    /// Probability of a packet being lost (0.0 = none, 1.0 = all).
    pub packet_loss: f64,
    /// Bandwidth limit in requests per second (0 = unlimited).
    #[serde(default)]
    pub bandwidth_rps: u32,
}

/// Traffic generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficConfig {
    pub start_rps: u32,
    pub target_rps: u32,
    pub ramp_seconds: u64,
}

/// A scenario definition — the immutable description of a simulation.
///
/// This is what the user creates in the visual editor or the DSL.
/// It is serialised to JSON and sent to the WASM engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub nodes: Vec<NodeConfig>,
    pub connections: Vec<ConnectionConfig>,
    pub traffic: TrafficConfig,
    pub seed: u64,
}

// ---------------------------------------------------------------------------
// Runtime types (mutable during a run)
// ---------------------------------------------------------------------------

/// State of a node during simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeState {
    #[default]
    Healthy,
    /// Partially degraded — increased latency or reduced capacity.
    Degraded,
    /// Completely non-functional.
    Failed,
    /// Coming back online after a failure.
    Recovering,
}

/// Where a request is in its journey through the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestPhase {
    /// Created at a client, travelling to the first hop.
    InTransit,
    /// Arrived at a node, waiting for processing.
    Queued,
    /// Being processed by a node.
    Processing,
    /// Querying a downstream node (e.g. service → database).
    AwaitingDownstream,
    /// Completed successfully, travelling back.
    Returning,
    /// Failed and waiting to be retried.
    PendingRetry,
    /// Completed (success or failure) — terminal state.
    Done,
}

/// The outcome of a completed request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestOutcome {
    Success,
    Failed,
    TimedOut,
    Dropped,
}

/// Runtime state of a request moving through the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestState {
    pub id: u64,
    pub origin: String,
    pub created_at: VirtualTime,
    pub current_node: Option<String>,
    pub phase: RequestPhase,
    pub retry_count: u32,
    pub outcome: Option<RequestOutcome>,
    /// Nodes visited by this request, in order.
    pub visited: Vec<String>,
    /// Latency contributions from each hop.
    pub hop_latencies: Vec<u64>,
}

impl RequestState {
    pub fn new(id: u64, origin: String, created_at: VirtualTime) -> Self {
        Self {
            id,
            origin,
            created_at,
            current_node: None,
            phase: RequestPhase::InTransit,
            retry_count: 0,
            outcome: None,
            visited: Vec::new(),
            hop_latencies: Vec::new(),
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.phase == RequestPhase::Done
    }

    pub fn total_latency(&self) -> u64 {
        self.hop_latencies.iter().sum()
    }
}

/// Runtime state of a node during a simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRuntimeState {
    pub id: String,
    pub state: NodeState,
    /// Current number of requests being processed.
    pub active_requests: u32,
    /// Current queue depth.
    pub queue_depth: u32,
    /// Total requests received.
    pub total_received: u64,
    /// Total requests completed successfully.
    pub total_completed: u64,
    /// Total requests that failed.
    pub total_failed: u64,
    /// Total requests timed out.
    pub total_timed_out: u64,
    /// Total requests dropped (queue overflow).
    pub total_dropped: u64,
    /// Total requests shed by load shedding policy.
    pub total_shedded: u64,
    /// Total cache hits.
    pub total_cache_hits: u64,
    /// Total cache misses.
    pub total_cache_misses: u64,
    /// Total stale reads served from replica.
    pub total_stale_reads: u64,
    /// Remaining retry budget.
    pub retry_budget_remaining: Option<u32>,
}

impl NodeRuntimeState {
    pub fn new(id: String, retry_budget: Option<u32>) -> Self {
        Self {
            id,
            state: NodeState::Healthy,
            active_requests: 0,
            queue_depth: 0,
            total_received: 0,
            total_completed: 0,
            total_failed: 0,
            total_timed_out: 0,
            total_dropped: 0,
            total_shedded: 0,
            total_cache_hits: 0,
            total_cache_misses: 0,
            total_stale_reads: 0,
            retry_budget_remaining: retry_budget,
        }
    }

    pub fn utilisation(&self, capacity: u32) -> f64 {
        if capacity == 0 {
            return 0.0;
        }
        self.active_requests as f64 / capacity as f64
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Events processed by the simulation engine.
///
/// Each event carries the data needed to process it. The scheduler
/// orders events by virtual time and insertion sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    RequestCreated {
        request_id: u64,
        origin: String,
    },
    RequestArrived {
        request_id: u64,
        node_id: String,
    },
    RequestStarted {
        request_id: u64,
        node_id: String,
    },
    RequestCompleted {
        request_id: u64,
        node_id: String,
        success: bool,
    },
    RequestTimedOut {
        request_id: u64,
        node_id: String,
    },
    RetryScheduled {
        request_id: u64,
        node_id: String,
        retry_count: u32,
    },
    NodeFailed {
        node_id: String,
    },
    NodeRecovered {
        node_id: String,
    },
    MessageQueued {
        request_id: u64,
        queue_id: String,
    },
    MessageDropped {
        request_id: u64,
        queue_id: String,
    },
    RequestShedded {
        request_id: u64,
        node_id: String,
        policy: SheddingPolicy,
    },
    RequestDequeued {
        request_id: u64,
        node_id: String,
    },
    /// A cache hit occurred — request served from cache without downstream.
    CacheHit {
        request_id: u64,
        node_id: String,
    },
    /// A cache miss — request must proceed to downstream.
    CacheMiss {
        request_id: u64,
        node_id: String,
    },
    /// A stale read was served from a replica.
    StaleRead {
        request_id: u64,
        node_id: String,
    },
    ConnectionFailed {
        from: String,
        to: String,
    },
    ConnectionRestored {
        from: String,
        to: String,
    },
}

// ---------------------------------------------------------------------------
// Failure injection
// ---------------------------------------------------------------------------

/// Types of failures that can be injected mid-simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FailureInjection {
    Crash { node_id: String },
    Recover { node_id: String },
    AddLatency { node_id: String, latency_ms: u64 },
    AddPacketLoss { from: String, to: String, rate: f64 },
    Disconnect { from: String, to: String },
    ReduceCapacity { node_id: String, new_capacity: u32 },
}

// ---------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------

/// Metrics collected during a simulation run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metrics {
    pub total_requests: u64,
    pub successful: u64,
    pub failed: u64,
    pub timed_out: u64,
    pub retries: u64,
    pub dropped: u64,
    pub shedded: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub stale_reads: u64,
    pub current_rps: f64,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub queue_depths: HashMap<String, u32>,
    pub node_utilisation: HashMap<String, f64>,
}

// ---------------------------------------------------------------------------
// Simulation state (the full runtime snapshot)
// ---------------------------------------------------------------------------

/// The complete mutable state of a simulation run.
///
/// This struct is what the engine mutates as it processes events.
/// Resetting a simulation means replacing this with a fresh instance
/// derived from the immutable `Scenario`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub current_time: VirtualTime,
    pub requests: HashMap<u64, RequestState>,
    pub nodes: HashMap<String, NodeRuntimeState>,
    pub network: crate::network::NetworkState,
    pub next_request_id: u64,
    pub metrics: Metrics,
    pub completed_latencies: Vec<u64>,
    /// Per-node FIFO queue of request IDs waiting for capacity.
    pub waiting_queues: HashMap<String, VecDeque<u64>>,
}

impl SimulationState {
    pub fn from_scenario(scenario: &Scenario) -> Self {
        let nodes = scenario
            .nodes
            .iter()
            .map(|n| {
                let budget = n.retry_policy.budget;
                (n.id.clone(), NodeRuntimeState::new(n.id.clone(), budget))
            })
            .collect();

        Self {
            current_time: VirtualTime::zero(),
            requests: HashMap::new(),
            nodes,
            network: crate::network::NetworkState::from_connections(&scenario.connections),
            next_request_id: 1,
            metrics: Metrics::default(),
            completed_latencies: Vec::new(),
            waiting_queues: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_time_display() {
        let t = VirtualTime(12_400);
        assert_eq!(format!("{}", t), "012.400");
    }

    #[test]
    fn virtual_time_seconds() {
        let t = VirtualTime(2500);
        assert_eq!(t.seconds(), 2.5);
    }

    #[test]
    fn request_state_starts_in_transit() {
        let req = RequestState::new(1, "client".into(), VirtualTime::zero());
        assert_eq!(req.phase, RequestPhase::InTransit);
        assert!(!req.is_terminal());
        assert_eq!(req.retry_count, 0);
    }

    #[test]
    fn request_state_done_is_terminal() {
        let mut req = RequestState::new(1, "client".into(), VirtualTime::zero());
        req.phase = RequestPhase::Done;
        assert!(req.is_terminal());
    }

    #[test]
    fn request_total_latency_sums_hops() {
        let mut req = RequestState::new(1, "client".into(), VirtualTime::zero());
        req.hop_latencies = vec![10, 25, 15, 30];
        assert_eq!(req.total_latency(), 80);
    }

    #[test]
    fn node_utilisation_calculates_ratio() {
        let mut node = NodeRuntimeState::new("svc".into(), None);
        node.active_requests = 25;
        assert_eq!(node.utilisation(100), 0.25);
    }

    #[test]
    fn node_utilisation_zero_capacity() {
        let node = NodeRuntimeState::new("svc".into(), None);
        assert_eq!(node.utilisation(0), 0.0);
    }

    #[test]
    fn simulation_state_initialises_from_scenario() {
        let scenario = Scenario {
            name: "test".into(),
            nodes: vec![NodeConfig {
                id: "client".into(),
                kind: ComponentKind::Client,
                name: "Client".into(),
                capacity: 100,
                latency_ms: 5,
                error_rate: 0.0,
                timeout_ms: 5000,
                queue_limit: None,
                cache_hit_rate: None,
                retry_policy: RetryPolicy::default(),
                shed_policy: SheddingPolicy::default(),
                replication_role: ReplicationRole::default(),
                replication_lag_ms: 0,
            }],
            connections: vec![],
            traffic: TrafficConfig {
                start_rps: 10,
                target_rps: 100,
                ramp_seconds: 30,
            },
            seed: 42,
        };

        let state = SimulationState::from_scenario(&scenario);
        assert_eq!(state.current_time, VirtualTime::zero());
        assert!(state.requests.is_empty());
        assert_eq!(state.next_request_id, 1);
        assert!(state.nodes.contains_key("client"));
        assert_eq!(state.nodes["client"].state, NodeState::Healthy);
    }

    #[test]
    fn retry_policy_default_is_immediate() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.strategy, RetryStrategy::Immediate);
        assert_eq!(policy.max_retries, 3);
    }
}
