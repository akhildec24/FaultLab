//! Network model — connection-level runtime state and bandwidth limiting.
//!
//! The network model tracks the health of each connection and applies
//! bandwidth limits. Connections can be disconnected (network partition)
//! and restored mid-simulation.
//!
//! This is educational, not a real TCP/IP stack. We model:
//! - Connection up/down state
//! - Bandwidth as a simple token bucket (requests per second)
//! - Latency spikes (random surges in latency)

use crate::rng::Rng;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime state of a connection between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionState {
    pub from: String,
    pub to: String,
    /// Whether the connection is currently up.
    pub connected: bool,
    /// Additional latency injected mid-simulation (ms).
    pub injected_latency_ms: u64,
    /// Additional packet loss injected mid-simulation (0.0–1.0).
    pub injected_packet_loss: f64,
    /// Requests sent through this connection since last refill.
    pub tokens_used: u64,
    /// Virtual time of last token refill.
    pub last_refill_time: u64,
}

impl ConnectionState {
    pub fn new(from: String, to: String) -> Self {
        Self {
            from,
            to,
            connected: true,
            injected_latency_ms: 0,
            injected_packet_loss: 0.0,
            tokens_used: 0,
            last_refill_time: 0,
        }
    }

    /// Key for HashMap — "from->to".
    pub fn key(&self) -> String {
        format!("{}->{}", self.from, self.to)
    }

    /// Effective latency = base + injected.
    pub fn effective_latency(&self, base_latency_ms: u64) -> u64 {
        base_latency_ms + self.injected_latency_ms
    }

    /// Effective packet loss = base + injected.
    pub fn effective_packet_loss(&self, base_packet_loss: f64) -> f64 {
        (base_packet_loss + self.injected_packet_loss).min(1.0)
    }
}

/// The full network state during a simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkState {
    pub connections: HashMap<String, ConnectionState>,
}

impl NetworkState {
    /// Build network state from scenario connections.
    pub fn from_connections(connections: &[ConnectionConfig]) -> Self {
        let mut map = HashMap::new();
        for conn in connections {
            let state = ConnectionState::new(conn.from.clone(), conn.to.clone());
            map.insert(state.key(), state);
        }
        Self { connections: map }
    }

    /// Get a connection state by from/to.
    pub fn get(&self, from: &str, to: &str) -> Option<&ConnectionState> {
        self.connections.get(&format!("{}->{}", from, to))
    }

    /// Get a mutable connection state by from/to.
    pub fn get_mut(&mut self, from: &str, to: &str) -> Option<&mut ConnectionState> {
        self.connections.get_mut(&format!("{}->{}", from, to))
    }

    /// Disconnect a connection (network partition).
    pub fn disconnect(&mut self, from: &str, to: &str) -> bool {
        if let Some(conn) = self.get_mut(from, to) {
            conn.connected = false;
            true
        } else {
            false
        }
    }

    /// Reconnect a connection.
    pub fn reconnect(&mut self, from: &str, to: &str) -> bool {
        if let Some(conn) = self.get_mut(from, to) {
            conn.connected = true;
            true
        } else {
            false
        }
    }

    /// Inject additional latency on a connection.
    pub fn inject_latency(&mut self, from: &str, to: &str, latency_ms: u64) -> bool {
        if let Some(conn) = self.get_mut(from, to) {
            conn.injected_latency_ms += latency_ms;
            true
        } else {
            false
        }
    }

    /// Inject additional packet loss on a connection.
    pub fn inject_packet_loss(&mut self, from: &str, to: &str, rate: f64) -> bool {
        if let Some(conn) = self.get_mut(from, to) {
            conn.injected_packet_loss = (conn.injected_packet_loss + rate).min(1.0);
            true
        } else {
            false
        }
    }

    /// Check if a connection is up.
    pub fn is_connected(&self, from: &str, to: &str) -> bool {
        self.get(from, to).is_some_and(|c| c.connected)
    }

    /// Check bandwidth availability and consume a token if available.
    ///
    /// Uses a simple token bucket: refills `bandwidth_rps` tokens per
    /// second of virtual time. Returns `true` if the request can proceed,
    /// `false` if bandwidth is exhausted (request should be delayed or dropped).
    pub fn check_bandwidth(
        &mut self,
        from: &str,
        to: &str,
        bandwidth_rps: u32,
        current_time: u64,
    ) -> bool {
        if bandwidth_rps == 0 {
            return true; // Unlimited
        }

        let key = format!("{}->{}", from, to);
        let conn = match self.connections.get_mut(&key) {
            Some(c) => c,
            None => return true, // No state = no limit
        };

        // Refill tokens based on elapsed time
        let elapsed_ms = current_time.saturating_sub(conn.last_refill_time);
        if elapsed_ms >= 1000 {
            let refill = (elapsed_ms / 1000) * bandwidth_rps as u64;
            conn.tokens_used = conn.tokens_used.saturating_sub(refill);
            conn.last_refill_time = current_time;
        }

        if conn.tokens_used >= bandwidth_rps as u64 {
            return false; // Bandwidth exhausted
        }

        conn.tokens_used += 1;
        true
    }

    /// Calculate transit time for a connection, applying network state.
    ///
    /// Returns `None` if the packet is lost or the connection is down.
    pub fn transit_time(
        &mut self,
        from: &str,
        to: &str,
        config: &ConnectionConfig,
        rng: &mut Rng,
        current_time: u64,
    ) -> Option<u64> {
        // Check connection state
        if !self.is_connected(from, to) {
            return None;
        }

        // Check bandwidth
        if !self.check_bandwidth(from, to, config.bandwidth_rps, current_time) {
            // Bandwidth exhausted — treat as a drop
            return None;
        }

        // Get effective values
        let base_latency = self
            .get(from, to)
            .map(|c| c.effective_latency(config.latency_ms))
            .unwrap_or(config.latency_ms);

        let packet_loss = self
            .get(from, to)
            .map(|c| c.effective_packet_loss(config.packet_loss))
            .unwrap_or(config.packet_loss);

        // Packet loss check
        if packet_loss > 0.0 && rng.chance(packet_loss) {
            return None;
        }

        // Latency with ±10% jitter
        let jitter = rng.range_f64(0.9, 1.1);
        let latency = (base_latency as f64 * jitter).round() as u64;

        Some(latency.max(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_conn(
        from: &str,
        to: &str,
        latency_ms: u64,
        packet_loss: f64,
        bandwidth_rps: u32,
    ) -> ConnectionConfig {
        ConnectionConfig {
            from: from.into(),
            to: to.into(),
            latency_ms,
            packet_loss,
            bandwidth_rps,
        }
    }

    #[test]
    fn network_state_builds_from_connections() {
        let conns = vec![
            make_conn("a", "b", 10, 0.0, 0),
            make_conn("b", "c", 20, 0.0, 0),
        ];
        let net = NetworkState::from_connections(&conns);
        assert!(net.get("a", "b").is_some());
        assert!(net.get("b", "c").is_some());
        assert!(net.get("a", "c").is_none());
    }

    #[test]
    fn connection_starts_connected() {
        let net = NetworkState::from_connections(&[make_conn("a", "b", 10, 0.0, 0)]);
        assert!(net.is_connected("a", "b"));
    }

    #[test]
    fn disconnect_breaks_connection() {
        let mut net = NetworkState::from_connections(&[make_conn("a", "b", 10, 0.0, 0)]);
        assert!(net.disconnect("a", "b"));
        assert!(!net.is_connected("a", "b"));
    }

    #[test]
    fn reconnect_restores_connection() {
        let mut net = NetworkState::from_connections(&[make_conn("a", "b", 10, 0.0, 0)]);
        net.disconnect("a", "b");
        assert!(!net.is_connected("a", "b"));
        assert!(net.reconnect("a", "b"));
        assert!(net.is_connected("a", "b"));
    }

    #[test]
    fn disconnect_nonexistent_returns_false() {
        let mut net = NetworkState::from_connections(&[]);
        assert!(!net.disconnect("a", "b"));
    }

    #[test]
    fn inject_latency_adds_to_base() {
        let mut net = NetworkState::from_connections(&[make_conn("a", "b", 100, 0.0, 0)]);
        assert!(net.inject_latency("a", "b", 50));
        let conn = net.get("a", "b").unwrap();
        assert_eq!(conn.effective_latency(100), 150);
    }

    #[test]
    fn inject_packet_loss_adds_to_base() {
        let mut net = NetworkState::from_connections(&[make_conn("a", "b", 10, 0.1, 0)]);
        net.inject_packet_loss("a", "b", 0.3);
        let conn = net.get("a", "b").unwrap();
        assert!((conn.effective_packet_loss(0.1) - 0.4).abs() < 0.001);
    }

    #[test]
    fn inject_packet_loss_capped_at_one() {
        let mut net = NetworkState::from_connections(&[make_conn("a", "b", 10, 0.5, 0)]);
        net.inject_packet_loss("a", "b", 0.8);
        let conn = net.get("a", "b").unwrap();
        assert_eq!(conn.effective_packet_loss(0.5), 1.0);
    }

    #[test]
    fn transit_time_disconnected_returns_none() {
        let mut net = NetworkState::from_connections(&[make_conn("a", "b", 100, 0.0, 0)]);
        let mut rng = Rng::new(42);
        net.disconnect("a", "b");
        let config = make_conn("a", "b", 100, 0.0, 0);
        assert!(net.transit_time("a", "b", &config, &mut rng, 0).is_none());
    }

    #[test]
    fn transit_time_connected_returns_latency() {
        let mut net = NetworkState::from_connections(&[make_conn("a", "b", 100, 0.0, 0)]);
        let mut rng = Rng::new(42);
        let config = make_conn("a", "b", 100, 0.0, 0);
        let t = net.transit_time("a", "b", &config, &mut rng, 0).unwrap();
        assert!(t >= 90 && t <= 110, "transit time {t} out of range");
    }

    #[test]
    fn transit_time_with_injected_latency() {
        let mut net = NetworkState::from_connections(&[make_conn("a", "b", 100, 0.0, 0)]);
        net.inject_latency("a", "b", 200);
        let mut rng = Rng::new(42);
        let config = make_conn("a", "b", 100, 0.0, 0);
        let t = net.transit_time("a", "b", &config, &mut rng, 0).unwrap();
        // (100 + 200) * jitter = 270–330
        assert!(
            t >= 270 && t <= 330,
            "transit time {t} out of range with injected latency"
        );
    }

    #[test]
    fn bandwidth_unlimited_when_zero() {
        let mut net = NetworkState::from_connections(&[make_conn("a", "b", 10, 0.0, 0)]);
        // Should allow many requests
        for _ in 0..100 {
            assert!(net.check_bandwidth("a", "b", 0, 0));
        }
    }

    #[test]
    fn bandwidth_limits_requests() {
        let mut net = NetworkState::from_connections(&[make_conn("a", "b", 10, 0.0, 5)]);
        // 5 rps — first 5 should succeed
        for _ in 0..5 {
            assert!(
                net.check_bandwidth("a", "b", 5, 0),
                "should allow within budget"
            );
        }
        // 6th should fail
        assert!(
            !net.check_bandwidth("a", "b", 5, 0),
            "should reject over budget"
        );
    }

    #[test]
    fn bandwidth_refills_after_one_second() {
        let mut net = NetworkState::from_connections(&[make_conn("a", "b", 10, 0.0, 5)]);
        // Exhaust bandwidth
        for _ in 0..5 {
            assert!(net.check_bandwidth("a", "b", 5, 0));
        }
        assert!(!net.check_bandwidth("a", "b", 5, 0));

        // After 1 second, should refill
        assert!(net.check_bandwidth("a", "b", 5, 1000));
    }

    #[test]
    fn transit_time_bandwidth_exhausted_returns_none() {
        let mut net = NetworkState::from_connections(&[make_conn("a", "b", 10, 0.0, 2)]);
        let mut rng = Rng::new(42);
        let config = make_conn("a", "b", 10, 0.0, 2);

        // First 2 requests succeed
        assert!(net.transit_time("a", "b", &config, &mut rng, 0).is_some());
        assert!(net.transit_time("a", "b", &config, &mut rng, 0).is_some());
        // 3rd should be dropped
        assert!(net.transit_time("a", "b", &config, &mut rng, 0).is_none());
    }
}
