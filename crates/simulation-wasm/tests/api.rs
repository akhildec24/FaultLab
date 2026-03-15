//! WASM API surface tests.
//!
//! These run in a browser via wasm-bindgen-test.
//! Run with: `wasm-pack test --browser`

use simulation_wasm::Simulation;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn new_simulation_is_not_running() {
    let sim = Simulation::new();
    assert!(!sim.is_running());
    assert_eq!(sim.current_time(), 0);
}

#[wasm_bindgen_test]
fn load_valid_scenario() {
    let mut sim = Simulation::new();
    let json = r#"{
        "name": "test",
        "nodes": [
            {"id": "client", "kind": "client", "name": "Client", "capacity": 100, "latency_ms": 5, "error_rate": 0.0, "timeout_ms": 5000, "retry_policy": {"strategy": "immediate", "max_retries": 3, "jitter": 0.0}},
            {"id": "svc", "kind": "service", "name": "Service", "capacity": 50, "latency_ms": 20, "error_rate": 0.0, "timeout_ms": 1000, "retry_policy": {"strategy": "immediate", "max_retries": 3, "jitter": 0.0}}
        ],
        "connections": [
            {"from": "client", "to": "svc", "latency_ms": 10, "packet_loss": 0.0, "bandwidth_rps": 0}
        ],
        "traffic": {"start_rps": 5, "target_rps": 5, "ramp_seconds": 0},
        "seed": 42
    }"#;
    sim.load_scenario(json).expect("should load");
    assert!(!sim.is_running());
}

#[wasm_bindgen_test]
fn load_invalid_scenario_returns_error() {
    let mut sim = Simulation::new();
    let result = sim.load_scenario("not valid json");
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn start_then_step_processes_events() {
    let mut sim = Simulation::new();
    let json = r#"{
        "name": "test",
        "nodes": [
            {"id": "client", "kind": "client", "name": "Client", "capacity": 100, "latency_ms": 5, "error_rate": 0.0, "timeout_ms": 5000, "retry_policy": {"strategy": "immediate", "max_retries": 3, "jitter": 0.0}}
        ],
        "connections": [],
        "traffic": {"start_rps": 5, "target_rps": 5, "ramp_seconds": 0},
        "seed": 42
    }"#;
    sim.load_scenario(json).unwrap();
    sim.start();
    assert!(sim.is_running());
    let processed = sim.step();
    assert!(processed, "should process at least one event");
}

#[wasm_bindgen_test]
fn run_processes_multiple_events() {
    let mut sim = Simulation::new();
    let json = r#"{
        "name": "test",
        "nodes": [
            {"id": "client", "kind": "client", "name": "Client", "capacity": 100, "latency_ms": 5, "error_rate": 0.0, "timeout_ms": 5000, "retry_policy": {"strategy": "immediate", "max_retries": 3, "jitter": 0.0}}
        ],
        "connections": [],
        "traffic": {"start_rps": 5, "target_rps": 5, "ramp_seconds": 0},
        "seed": 42
    }"#;
    sim.load_scenario(json).unwrap();
    sim.start();
    let steps = sim.run(100);
    assert!(steps > 0, "should process multiple events");
}

#[wasm_bindgen_test]
fn get_metrics_returns_valid_json() {
    let mut sim = Simulation::new();
    sim.start();
    let metrics_json = sim.get_metrics();
    let parsed: serde_json::Value = serde_json::from_str(&metrics_json).unwrap();
    assert!(parsed.is_object());
    assert!(parsed.get("total_requests").is_some());
}

#[wasm_bindgen_test]
fn get_state_returns_valid_json() {
    let sim = Simulation::new();
    let state_json = sim.get_state();
    let parsed: serde_json::Value = serde_json::from_str(&state_json).unwrap();
    assert!(parsed.is_object());
}

#[wasm_bindgen_test]
fn get_recent_events_returns_array() {
    let mut sim = Simulation::new();
    let json = r#"{
        "name": "test",
        "nodes": [
            {"id": "client", "kind": "client", "name": "Client", "capacity": 100, "latency_ms": 5, "error_rate": 0.0, "timeout_ms": 5000, "retry_policy": {"strategy": "immediate", "max_retries": 3, "jitter": 0.0}}
        ],
        "connections": [],
        "traffic": {"start_rps": 5, "target_rps": 5, "ramp_seconds": 0},
        "seed": 42
    }"#;
    sim.load_scenario(json).unwrap();
    sim.start();
    sim.run(10);
    let events_json = sim.get_recent_events();
    let parsed: serde_json::Value = serde_json::from_str(&events_json).unwrap();
    assert!(parsed.is_array());
    assert!(parsed.as_array().unwrap().len() > 0, "should have events");
}

#[wasm_bindgen_test]
fn reset_clears_state() {
    let mut sim = Simulation::new();
    let json = r#"{
        "name": "test",
        "nodes": [
            {"id": "client", "kind": "client", "name": "Client", "capacity": 100, "latency_ms": 5, "error_rate": 0.0, "timeout_ms": 5000, "retry_policy": {"strategy": "immediate", "max_retries": 3, "jitter": 0.0}}
        ],
        "connections": [],
        "traffic": {"start_rps": 5, "target_rps": 5, "ramp_seconds": 0},
        "seed": 42
    }"#;
    sim.load_scenario(json).unwrap();
    sim.start();
    sim.run(10);
    assert!(sim.current_time() > 0);
    sim.reset();
    assert_eq!(sim.current_time(), 0);
    assert!(!sim.is_running());
}

#[wasm_bindgen_test]
fn pause_then_run_does_nothing() {
    let mut sim = Simulation::new();
    let json = r#"{
        "name": "test",
        "nodes": [
            {"id": "client", "kind": "client", "name": "Client", "capacity": 100, "latency_ms": 5, "error_rate": 0.0, "timeout_ms": 5000, "retry_policy": {"strategy": "immediate", "max_retries": 3, "jitter": 0.0}}
        ],
        "connections": [],
        "traffic": {"start_rps": 5, "target_rps": 5, "ramp_seconds": 0},
        "seed": 42
    }"#;
    sim.load_scenario(json).unwrap();
    sim.start();
    sim.pause();
    assert!(!sim.is_running());
    let steps = sim.run(100);
    assert_eq!(steps, 0, "paused engine should not process events");
}

#[wasm_bindgen_test]
fn pending_events_nonzero_after_start() {
    let mut sim = Simulation::new();
    let json = r#"{
        "name": "test",
        "nodes": [
            {"id": "client", "kind": "client", "name": "Client", "capacity": 100, "latency_ms": 5, "error_rate": 0.0, "timeout_ms": 5000, "retry_policy": {"strategy": "immediate", "max_retries": 3, "jitter": 0.0}}
        ],
        "connections": [],
        "traffic": {"start_rps": 5, "target_rps": 5, "ramp_seconds": 0},
        "seed": 42
    }"#;
    sim.load_scenario(json).unwrap();
    sim.start();
    assert!(sim.pending_events() > 0, "should have scheduled events");
}
