//! Invalid configuration and edge-case tests.
//!
//! These verify the engine handles degenerate inputs gracefully:
//! empty scenarios, missing nodes, orphan connections, zero traffic, etc.

use simulation_core::engine::Engine;
use simulation_core::types::*;

fn make_node(id: &str, kind: ComponentKind, capacity: u32) -> NodeConfig {
    NodeConfig {
        id: id.into(),
        kind,
        name: id.into(),
        capacity,
        latency_ms: 10,
        error_rate: 0.0,
        timeout_ms: 5000,
        queue_limit: None,
        cache_hit_rate: None,
        retry_policy: RetryPolicy::default(),
        shed_policy: SheddingPolicy::default(),
        replication_role: ReplicationRole::default(),
        replication_lag_ms: 0,
    }
}

#[test]
fn empty_scenario_runs_without_crash() {
    let scenario = Scenario {
        name: "empty".into(),
        nodes: vec![],
        connections: vec![],
        traffic: TrafficConfig {
            start_rps: 0,
            target_rps: 0,
            ramp_seconds: 0,
        },
        seed: 42,
    };

    let mut engine = Engine::new(scenario);
    engine.start();
    let steps = engine.run(1000);
    assert_eq!(steps, 0, "empty scenario should process no events");
    assert_eq!(engine.metrics().total_requests, 0);
}

#[test]
fn single_client_no_connections_completes_immediately() {
    let scenario = Scenario {
        name: "lonely-client".into(),
        nodes: vec![make_node("client", ComponentKind::Client, 100)],
        connections: vec![],
        traffic: TrafficConfig {
            start_rps: 5,
            target_rps: 5,
            ramp_seconds: 0,
        },
        seed: 42,
    };

    let mut engine = Engine::new(scenario);
    engine.start();
    engine.run(10000);

    let m = engine.metrics();
    assert!(m.total_requests > 0, "should generate requests");
    assert_eq!(
        m.successful, m.total_requests,
        "all should succeed immediately"
    );
}

#[test]
fn connection_to_nonexistent_node_doesnt_crash() {
    let scenario = Scenario {
        name: "dangling-conn".into(),
        nodes: vec![make_node("a", ComponentKind::Client, 100)],
        connections: vec![ConnectionConfig {
            from: "a".into(),
            to: "nonexistent".into(),
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
    // Should not crash during run
    engine.run(10000);

    // Requests are created but arrive at a nonexistent node.
    // The engine should handle this gracefully.
    let m = engine.metrics();
    assert!(m.total_requests > 0);
    // All requests should reach a terminal state
    let all_done = engine
        .state()
        .requests
        .values()
        .all(|r| r.phase == RequestPhase::Done);
    assert!(
        all_done,
        "all requests should terminate even with dangling connection"
    );
}

#[test]
fn zero_traffic_generates_no_requests() {
    let scenario = Scenario {
        name: "no-traffic".into(),
        nodes: vec![
            make_node("client", ComponentKind::Client, 100),
            make_node("svc", ComponentKind::Service, 100),
        ],
        connections: vec![ConnectionConfig {
            from: "client".into(),
            to: "svc".into(),
            latency_ms: 10,
            packet_loss: 0.0,
            bandwidth_rps: 0,
        }],
        traffic: TrafficConfig {
            start_rps: 0,
            target_rps: 0,
            ramp_seconds: 0,
        },
        seed: 42,
    };

    let mut engine = Engine::new(scenario);
    engine.start();
    engine.run(10000);
    assert_eq!(engine.metrics().total_requests, 0);
}

#[test]
fn zero_capacity_node_drops_all_requests() {
    let scenario = Scenario {
        name: "zero-capacity".into(),
        nodes: vec![
            make_node("client", ComponentKind::Client, 100),
            NodeConfig {
                id: "svc".into(),
                kind: ComponentKind::Service,
                name: "svc".into(),
                capacity: 0,
                latency_ms: 10,
                error_rate: 0.0,
                timeout_ms: 5000,
                queue_limit: Some(0), // No queue either
                cache_hit_rate: None,
                retry_policy: RetryPolicy::default(),
                shed_policy: SheddingPolicy::default(),
                replication_role: ReplicationRole::default(),
                replication_lag_ms: 0,
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
    engine.run(10000);

    let m = engine.metrics();
    assert!(m.total_requests > 0);
    // All requests should be dropped or shedded (capacity 0, queue 0)
    assert_eq!(
        m.dropped + m.shedded,
        m.total_requests,
        "all should be dropped or shedded"
    );
}

#[test]
fn reset_restores_initial_state() {
    let scenario = Scenario {
        name: "reset-test".into(),
        nodes: vec![
            make_node("client", ComponentKind::Client, 100),
            make_node("svc", ComponentKind::Service, 100),
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
            target_rps: 10,
            ramp_seconds: 0,
        },
        seed: 42,
    };

    let mut engine = Engine::new(scenario);
    engine.start();
    engine.run(5000);
    assert!(engine.metrics().total_requests > 0);

    engine.reset();
    assert_eq!(engine.metrics().total_requests, 0);
    assert!(engine.state().requests.is_empty());
    assert!(!engine.is_running());
    assert_eq!(engine.metrics().total_requests, 0);

    // Should be able to start again
    engine.start();
    engine.run(5000);
    assert!(
        engine.metrics().total_requests > 0,
        "should generate after reset"
    );
}

#[test]
fn pause_stops_processing() {
    let scenario = Scenario {
        name: "pause-test".into(),
        nodes: vec![make_node("client", ComponentKind::Client, 100)],
        connections: vec![],
        traffic: TrafficConfig {
            start_rps: 10,
            target_rps: 10,
            ramp_seconds: 0,
        },
        seed: 42,
    };

    let mut engine = Engine::new(scenario);
    engine.start();
    engine.pause();
    assert!(!engine.is_running());
    // Run should process no steps when paused
    let steps = engine.run(100);
    assert_eq!(steps, 0, "paused engine should process no steps");
}

#[test]
fn high_packet_loss_still_terminates() {
    let scenario = Scenario {
        name: "packet-loss".into(),
        nodes: vec![
            make_node("client", ComponentKind::Client, 100),
            make_node("svc", ComponentKind::Service, 100),
        ],
        connections: vec![ConnectionConfig {
            from: "client".into(),
            to: "svc".into(),
            latency_ms: 10,
            packet_loss: 0.99, // 99% packet loss
            bandwidth_rps: 0,
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
    engine.run(50000);

    let m = engine.metrics();
    assert!(m.total_requests > 0);
    let accounted = m.successful + m.failed + m.timed_out + m.dropped;
    assert_eq!(accounted, m.total_requests, "all should terminate");
    assert!(m.failed > 0, "most should fail with 99% packet loss");
}

#[test]
fn diamond_topology_runs_to_completion() {
    // A → B → D
    // A → C → D
    let scenario = Scenario {
        name: "diamond".into(),
        nodes: vec![
            make_node("a", ComponentKind::Client, 100),
            make_node("b", ComponentKind::Service, 100),
            make_node("c", ComponentKind::Service, 100),
            make_node("d", ComponentKind::Database, 100),
        ],
        connections: vec![
            ConnectionConfig {
                from: "a".into(),
                to: "b".into(),
                latency_ms: 10,
                packet_loss: 0.0,
                bandwidth_rps: 0,
            },
            ConnectionConfig {
                from: "a".into(),
                to: "c".into(),
                latency_ms: 10,
                packet_loss: 0.0,
                bandwidth_rps: 0,
            },
            ConnectionConfig {
                from: "b".into(),
                to: "d".into(),
                latency_ms: 5,
                packet_loss: 0.0,
                bandwidth_rps: 0,
            },
            ConnectionConfig {
                from: "c".into(),
                to: "d".into(),
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
    };

    let mut engine = Engine::new(scenario);
    engine.start();
    engine.run(50000);

    let m = engine.metrics();
    assert!(m.total_requests > 0, "should generate requests");
    assert!(m.successful > 0, "some should succeed");
    // All requests should reach terminal state (no deadlock)
    let all_done = engine
        .state()
        .requests
        .values()
        .all(|r| r.phase == RequestPhase::Done);
    assert!(all_done, "all requests should be Done in diamond topology");
}
