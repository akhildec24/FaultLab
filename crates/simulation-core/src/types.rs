//! Core domain types for the simulation engine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Virtual simulation time in milliseconds.
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
}

impl Default for VirtualTime {
    fn default() -> Self {
        Self::zero()
    }
}

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

/// State of a node during simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeState {
    #[default]
    Healthy,
    Degraded,
    Failed,
    Recovering,
}

/// A node in the architecture graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: ComponentKind,
    pub name: String,
    pub state: NodeState,
    pub capacity: u32,
    pub latency_ms: u64,
    pub error_rate: f64,
    pub timeout_ms: u64,
    pub queue_limit: Option<u32>,
    pub cache_hit_rate: Option<f64>,
}

/// A directed connection between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: String,
    pub to: String,
    pub latency_ms: u64,
    pub packet_loss: f64,
}

/// A request moving through the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    pub origin: String,
    pub created_at: VirtualTime,
    pub retry_count: u32,
}

/// Events processed by the simulation engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    RequestCreated {
        request: Request,
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
}

impl Event {
    /// The virtual time at which this event should be processed.
    pub fn scheduled_time(&self) -> VirtualTime {
        match self {
            Event::RequestCreated { request } => request.created_at,
            Event::RequestArrived { .. } => VirtualTime::zero(),
            Event::RequestStarted { .. } => VirtualTime::zero(),
            Event::RequestCompleted { .. } => VirtualTime::zero(),
            Event::RequestTimedOut { .. } => VirtualTime::zero(),
            Event::RetryScheduled { .. } => VirtualTime::zero(),
            Event::NodeFailed { .. } => VirtualTime::zero(),
            Event::NodeRecovered { .. } => VirtualTime::zero(),
            Event::MessageQueued { .. } => VirtualTime::zero(),
            Event::MessageDropped { .. } => VirtualTime::zero(),
        }
    }
}

/// A scenario definition — the full description of a simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
    pub traffic_start_rps: u32,
    pub traffic_target_rps: u32,
    pub traffic_ramp_seconds: u64,
    pub seed: u64,
}

/// Metrics collected during a simulation run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metrics {
    pub total_requests: u64,
    pub successful: u64,
    pub failed: u64,
    pub timed_out: u64,
    pub retries: u64,
    pub dropped: u64,
    pub current_rps: f64,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub queue_depths: HashMap<String, u32>,
    pub node_utilisation: HashMap<String, f64>,
}
