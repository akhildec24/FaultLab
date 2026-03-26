//! Request routing — the connection graph and next-hop resolution.
//!
//! The routing table is built from a scenario's `ConnectionConfig` list.
//! It maps each node to its downstream neighbours. When a request
//! completes at a node, the router determines where to send it next.
//!
//! Routing is intentionally simple: a node forwards to all of its
//! downstream connections. In the first version, a service calls its
//! single downstream dependency. More sophisticated routing (load
//! balancing, conditional forwarding) arrives in later weeks.

use crate::rng::Rng;
use crate::types::*;
use std::collections::HashMap;

/// The routing graph built from a scenario's connections.
#[derive(Debug, Clone)]
pub struct Router {
    /// Map from node_id → list of (downstream_id, connection config).
    adjacency: HashMap<String, Vec<(String, ConnectionConfig)>>,
}

impl Router {
    /// Build a router from a list of connection configs.
    pub fn from_connections(connections: &[ConnectionConfig]) -> Self {
        let mut adjacency: HashMap<String, Vec<(String, ConnectionConfig)>> = HashMap::new();
        for conn in connections {
            adjacency
                .entry(conn.from.clone())
                .or_default()
                .push((conn.to.clone(), conn.clone()));
        }
        Self { adjacency }
    }

    /// Get the downstream neighbours of a node.
    pub fn downstream(&self, node_id: &str) -> &[(String, ConnectionConfig)] {
        self.adjacency
            .get(node_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Whether a node has any downstream connections.
    pub fn has_downstream(&self, node_id: &str) -> bool {
        self.adjacency.get(node_id).is_some_and(|v| !v.is_empty())
    }

    /// Get the connection config between two nodes, if it exists.
    pub fn connection(&self, from: &str, to: &str) -> Option<&ConnectionConfig> {
        self.adjacency
            .get(from)?
            .iter()
            .find(|(id, _)| id == to)
            .map(|(_, c)| c)
    }

    /// Calculate the transit time for a request travelling along a connection.
    ///
    /// Applies the connection's base latency plus jitter from the RNG.
    /// Returns `None` if the packet is lost.
    pub fn transit_time(&self, from: &str, to: &str, rng: &mut Rng) -> Option<u64> {
        let conn = self.connection(from, to)?;

        // Packet loss check
        if conn.packet_loss > 0.0 && rng.chance(conn.packet_loss) {
            return None;
        }

        // Base latency with ±10% jitter
        let jitter = rng.range_f64(0.9, 1.1);
        let latency = (conn.latency_ms as f64 * jitter).round() as u64;

        Some(latency.max(1))
    }

    /// Find all client nodes (nodes with no incoming connections).
    pub fn entry_points(
        &self,
        nodes: &[NodeConfig],
        connections: &[ConnectionConfig],
    ) -> Vec<String> {
        let has_incoming: std::collections::HashSet<&str> =
            connections.iter().map(|c| c.to.as_str()).collect();
        nodes
            .iter()
            .filter(|n| n.kind == ComponentKind::Client || !has_incoming.contains(n.id.as_str()))
            .map(|n| n.id.clone())
            .collect()
    }
}

/// Determine the processing time for a request at a given node.
///
/// Applies the node's base latency with ±10% jitter.
pub fn processing_time(node: &NodeConfig, rng: &mut Rng) -> u64 {
    let jitter = rng.range_f64(0.9, 1.1);
    let latency = (node.latency_ms as f64 * jitter).round() as u64;
    latency.max(1)
}

/// Determine if a request should fail at a given node based on error rate.
pub fn should_fail(node: &NodeConfig, rng: &mut Rng) -> bool {
    if node.error_rate <= 0.0 {
        return false;
    }
    rng.chance(node.error_rate)
}

/// Calculate the retry delay for a request based on the retry policy.
pub fn retry_delay(policy: &RetryPolicy, attempt: u32, rng: &mut Rng) -> u64 {
    let base = match policy.strategy {
        RetryStrategy::Immediate => 0,
        RetryStrategy::Fixed { delay_ms } => delay_ms,
        RetryStrategy::Exponential {
            base_ms,
            max_delay_ms,
        } => {
            let exp = 2u64.saturating_pow(attempt);
            let delay = base_ms.saturating_mul(exp);
            delay.min(max_delay_ms)
        }
    };

    // Apply jitter
    if policy.jitter > 0.0 && base > 0 {
        let jitter_range = base as f64 * policy.jitter;
        let jittered = rng.range_f64(
            (base as f64 - jitter_range).max(0.0),
            base as f64 + jitter_range,
        );
        jittered.round() as u64
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_connection(
        from: &str,
        to: &str,
        latency_ms: u64,
        packet_loss: f64,
    ) -> ConnectionConfig {
        ConnectionConfig {
            from: from.into(),
            to: to.into(),
            latency_ms,
            packet_loss,
            bandwidth_rps: 0,
        }
    }

    #[test]
    fn router_builds_adjacency() {
        let conns = vec![
            make_connection("a", "b", 10, 0.0),
            make_connection("b", "c", 20, 0.0),
        ];
        let router = Router::from_connections(&conns);

        assert!(router.has_downstream("a"));
        assert!(router.has_downstream("b"));
        assert!(!router.has_downstream("c"));
        assert_eq!(router.downstream("a").len(), 1);
        assert_eq!(router.downstream("b").len(), 1);
    }

    #[test]
    fn router_downstream_empty_for_leaf() {
        let router = Router::from_connections(&[make_connection("a", "b", 10, 0.0)]);
        assert!(!router.has_downstream("b"));
        assert!(router.downstream("b").is_empty());
    }

    #[test]
    fn transit_time_returns_latency_with_jitter() {
        let mut rng = Rng::new(42);
        let router = Router::from_connections(&[make_connection("a", "b", 100, 0.0)]);
        let t = router.transit_time("a", "b", &mut rng).unwrap();
        // ±10% of 100 = [90, 110]
        assert!(
            t >= 90 && t <= 110,
            "transit time {t} out of expected range"
        );
    }

    #[test]
    fn transit_time_packet_loss_returns_none() {
        let mut rng = Rng::new(42);
        let router = Router::from_connections(&[make_connection("a", "b", 100, 1.0)]);
        assert!(router.transit_time("a", "b", &mut rng).is_none());
    }

    #[test]
    fn transit_time_no_packet_loss() {
        let mut rng = Rng::new(42);
        let router = Router::from_connections(&[make_connection("a", "b", 100, 0.0)]);
        assert!(router.transit_time("a", "b", &mut rng).is_some());
    }

    #[test]
    fn transit_time_missing_connection_returns_none() {
        let mut rng = Rng::new(42);
        let router = Router::from_connections(&[]);
        assert!(router.transit_time("a", "b", &mut rng).is_none());
    }

    #[test]
    fn entry_points_finds_clients() {
        let nodes = vec![
            NodeConfig {
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
            },
            NodeConfig {
                id: "svc".into(),
                kind: ComponentKind::Service,
                name: "Service".into(),
                capacity: 50,
                latency_ms: 20,
                error_rate: 0.0,
                timeout_ms: 1000,
                queue_limit: None,
                cache_hit_rate: None,
                retry_policy: RetryPolicy::default(),
                shed_policy: SheddingPolicy::default(),
            },
        ];
        let conns = vec![make_connection("client", "svc", 10, 0.0)];
        let router = Router::from_connections(&conns);
        let entries = router.entry_points(&nodes, &conns);
        assert!(entries.contains(&"client".to_string()));
        assert!(!entries.contains(&"svc".to_string()));
    }

    #[test]
    fn processing_time_applies_jitter() {
        let mut rng = Rng::new(42);
        let node = NodeConfig {
            id: "svc".into(),
            kind: ComponentKind::Service,
            name: "Service".into(),
            capacity: 50,
            latency_ms: 50,
            error_rate: 0.0,
            timeout_ms: 1000,
            queue_limit: None,
            cache_hit_rate: None,
            retry_policy: RetryPolicy::default(),
            shed_policy: SheddingPolicy::default(),
        };
        let t = processing_time(&node, &mut rng);
        // ±10% of 50 = [45, 55]
        assert!(t >= 45 && t <= 55, "processing time {t} out of range");
    }

    #[test]
    fn should_fail_zero_error_rate() {
        let mut rng = Rng::new(42);
        let node = NodeConfig {
            id: "svc".into(),
            kind: ComponentKind::Service,
            name: "Service".into(),
            capacity: 50,
            latency_ms: 20,
            error_rate: 0.0,
            timeout_ms: 1000,
            queue_limit: None,
            cache_hit_rate: None,
            retry_policy: RetryPolicy::default(),
            shed_policy: SheddingPolicy::default(),
        };
        for _ in 0..100 {
            assert!(!should_fail(&node, &mut rng));
        }
    }

    #[test]
    fn should_fail_high_error_rate() {
        let mut rng = Rng::new(42);
        let node = NodeConfig {
            id: "svc".into(),
            kind: ComponentKind::Service,
            name: "Service".into(),
            capacity: 50,
            latency_ms: 20,
            error_rate: 1.0,
            timeout_ms: 1000,
            queue_limit: None,
            cache_hit_rate: None,
            retry_policy: RetryPolicy::default(),
            shed_policy: SheddingPolicy::default(),
        };
        for _ in 0..100 {
            assert!(should_fail(&node, &mut rng));
        }
    }

    #[test]
    fn retry_delay_immediate_is_zero() {
        let mut rng = Rng::new(42);
        let policy = RetryPolicy::default();
        assert_eq!(retry_delay(&policy, 0, &mut rng), 0);
        assert_eq!(retry_delay(&policy, 3, &mut rng), 0);
    }

    #[test]
    fn retry_delay_fixed() {
        let mut rng = Rng::new(42);
        let policy = RetryPolicy {
            strategy: RetryStrategy::Fixed { delay_ms: 200 },
            max_retries: 3,
            jitter: 0.0,
            budget: None,
        };
        assert_eq!(retry_delay(&policy, 0, &mut rng), 200);
        assert_eq!(retry_delay(&policy, 1, &mut rng), 200);
    }

    #[test]
    fn retry_delay_exponential_grows() {
        let mut rng = Rng::new(42);
        let policy = RetryPolicy {
            strategy: RetryStrategy::Exponential {
                base_ms: 100,
                max_delay_ms: 10000,
            },
            max_retries: 5,
            jitter: 0.0,
            budget: None,
        };
        let d0 = retry_delay(&policy, 0, &mut rng);
        let d1 = retry_delay(&policy, 1, &mut rng);
        let d2 = retry_delay(&policy, 2, &mut rng);
        assert!(d0 < d1, "exponential should grow: {d0} < {d1}");
        assert!(d1 < d2, "exponential should grow: {d1} < {d2}");
    }

    #[test]
    fn retry_delay_exponential_capped() {
        let mut rng = Rng::new(42);
        let policy = RetryPolicy {
            strategy: RetryStrategy::Exponential {
                base_ms: 100,
                max_delay_ms: 500,
            },
            max_retries: 10,
            jitter: 0.0,
            budget: None,
        };
        // 2^10 * 100 = 102400, but capped at 500
        let delay = retry_delay(&policy, 10, &mut rng);
        assert_eq!(delay, 500);
    }
}
