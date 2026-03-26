//! AST — abstract syntax tree for the FaultLab DSL.
//!
//! The AST is a direct representation of the parsed source code.
//! It is converted to the simulation-core `Scenario` struct by the
//! `converter` module after semantic validation.

use simulation_core::{
    ComponentKind, ConnectionConfig, FailureInjection, NodeConfig, ReplicationRole,
    RetryPolicy, RetryStrategy, SheddingPolicy, TrafficConfig,
};

/// A parsed scenario AST.
#[derive(Debug, Clone)]
pub struct AstScenario {
    pub name: String,
    pub seed: u64,
    pub nodes: Vec<AstNode>,
    pub edges: Vec<AstEdge>,
    pub traffic: Option<AstTraffic>,
    pub failures: Vec<AstFailure>,
}

impl Default for AstScenario {
    fn default() -> Self {
        Self {
            name: "unnamed".into(),
            seed: 42,
            nodes: Vec::new(),
            edges: Vec::new(),
            traffic: None,
            failures: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub id: String,
    pub kind: ComponentKind,
    pub name: Option<String>,
    pub capacity: Option<u32>,
    pub latency_ms: Option<u64>,
    pub error_rate: Option<f64>,
    pub timeout_ms: Option<u64>,
    pub queue_limit: Option<u32>,
    pub cache_hit_rate: Option<f64>,
    pub replication_role: Option<ReplicationRole>,
    pub replication_lag_ms: Option<u64>,
    pub retry_policy: Option<AstRetryPolicy>,
    pub shed_policy: Option<SheddingPolicy>,
}

#[derive(Debug, Clone)]
pub struct AstRetryPolicy {
    pub strategy: RetryStrategy,
    pub max_retries: u32,
    pub jitter: f64,
    pub budget: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct AstEdge {
    pub from: String,
    pub to: String,
    pub latency_ms: Option<u64>,
    pub packet_loss: Option<f64>,
    pub bandwidth_rps: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct AstTraffic {
    pub start_rps: u32,
    pub target_rps: u32,
    pub ramp_seconds: u64,
}

impl Default for AstTraffic {
    fn default() -> Self {
        Self {
            start_rps: 10,
            target_rps: 100,
            ramp_seconds: 30,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AstFailure {
    pub at_ms: u64,
    pub failure: FailureInjection,
}

// --- Conversion to simulation-core types ---

impl AstScenario {
    /// Convert AST to a `Scenario` struct suitable for the engine.
    pub fn to_scenario(&self) -> simulation_core::Scenario {
        let nodes: Vec<NodeConfig> = self
            .nodes
            .iter()
            .map(|n| NodeConfig {
                id: n.id.clone(),
                kind: n.kind,
                name: n.name.clone().unwrap_or_else(|| n.id.clone()),
                capacity: n.capacity.unwrap_or(100),
                latency_ms: n.latency_ms.unwrap_or(10),
                error_rate: n.error_rate.unwrap_or(0.0),
                timeout_ms: n.timeout_ms.unwrap_or(5000),
                queue_limit: n.queue_limit,
                cache_hit_rate: n.cache_hit_rate,
                replication_role: n.replication_role.unwrap_or_default(),
                replication_lag_ms: n.replication_lag_ms.unwrap_or(0),
                retry_policy: n.retry_policy.as_ref().map(|r| RetryPolicy {
                    strategy: r.strategy,
                    max_retries: r.max_retries,
                    jitter: r.jitter,
                    budget: r.budget,
                }).unwrap_or_default(),
                shed_policy: n.shed_policy.clone().unwrap_or_default(),
            })
            .collect();

        let connections: Vec<ConnectionConfig> = self
            .edges
            .iter()
            .map(|e| ConnectionConfig {
                from: e.from.clone(),
                to: e.to.clone(),
                latency_ms: e.latency_ms.unwrap_or(0),
                packet_loss: e.packet_loss.unwrap_or(0.0),
                bandwidth_rps: e.bandwidth_rps.unwrap_or(0),
            })
            .collect();

        let traffic = self.traffic.clone().map(|t| TrafficConfig {
            start_rps: t.start_rps,
            target_rps: t.target_rps,
            ramp_seconds: t.ramp_seconds,
        }).unwrap_or(TrafficConfig {
            start_rps: 10,
            target_rps: 100,
            ramp_seconds: 30,
        });

        simulation_core::Scenario {
            name: self.name.clone(),
            nodes,
            connections,
            traffic,
            seed: self.seed,
        }
    }

    /// Get the list of scheduled failures (to be injected at runtime).
    pub fn scheduled_failures(&self) -> Vec<(u64, FailureInjection)> {
        self.failures
            .iter()
            .map(|f| (f.at_ms, f.failure.clone()))
            .collect()
    }
}
