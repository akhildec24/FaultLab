//! WebAssembly bindings for the simulation engine.
//!
//! This crate exposes a clean JavaScript interface to the Rust simulation
//! engine via wasm-bindgen. It deliberately hides internal Rust structures
//! and returns JSON strings for complex data.
//!
//! API:
//!   - `new()` — create a simulation with an empty scenario
//!   - `loadScenario(json)` — load a scenario from JSON string
//!   - `start()` — begin processing events
//!   - `pause()` — stop processing (can resume with `start`)
//!   - `reset()` — clear all state, keep scenario
//!   - `step()` — process one event, returns `true` if an event was processed
//!   - `run(maxSteps)` — process up to `maxSteps` events, returns count
//!   - `isRunning()` — whether the engine is currently running
//!   - `currentTime()` — virtual time in milliseconds
//!   - `getMetrics()` — JSON string of current metrics
//!   - `getState()` — JSON string of full simulation state
//!   - `getRecentEvents()` — JSON string of recent events (drains buffer)
//!   - `pendingEvents()` — number of events in the scheduler queue

use simulation_core::{Engine, Event, Metrics, SimulationState};
use wasm_bindgen::prelude::*;

/// Initialise the panic hook for better error messages in the browser.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct Simulation {
    engine: Engine,
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl Simulation {
    /// Create a new simulation with an empty scenario.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            engine: Engine::new(simulation_core::Scenario {
                name: "empty".into(),
                nodes: vec![],
                connections: vec![],
                traffic: simulation_core::TrafficConfig {
                    start_rps: 0,
                    target_rps: 0,
                    ramp_seconds: 0,
                },
                seed: 0,
            }),
        }
    }

    /// Load a scenario from a JSON string.
    pub fn load_scenario(&mut self, json: &str) -> Result<(), JsValue> {
        let scenario: simulation_core::Scenario = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Invalid scenario: {}", e)))?;
        self.engine = Engine::load(scenario);
        Ok(())
    }

    /// Start the simulation (schedules traffic if not already scheduled).
    pub fn start(&mut self) {
        self.engine.start();
    }

    /// Pause the simulation.
    pub fn pause(&mut self) {
        self.engine.pause();
    }

    /// Reset the simulation to its initial state.
    pub fn reset(&mut self) {
        self.engine.reset();
    }

    /// Process one event. Returns `true` if an event was processed.
    pub fn step(&mut self) -> bool {
        self.engine.step()
    }

    /// Process up to `max_steps` events. Returns the number of steps executed.
    pub fn run(&mut self, max_steps: usize) -> usize {
        self.engine.run(max_steps)
    }

    /// Whether the engine is currently running.
    pub fn is_running(&self) -> bool {
        self.engine.is_running()
    }

    /// Current virtual time in milliseconds.
    pub fn current_time(&self) -> u64 {
        self.engine.now().millis()
    }

    /// Current metrics as a JSON string.
    pub fn get_metrics(&self) -> String {
        let metrics: &Metrics = self.engine.metrics();
        serde_json::to_string(metrics).unwrap_or_else(|_| "{}".into())
    }

    /// Full simulation state as a JSON string (requests, nodes, network, metrics).
    pub fn get_state(&self) -> String {
        let state: &SimulationState = self.engine.state();
        serde_json::to_string(state).unwrap_or_else(|_| "{}".into())
    }

    /// Drain and return recent events as a JSON array string.
    /// Each event is `{"time": <ms>, "event": {...}}`.
    pub fn get_recent_events(&mut self) -> String {
        let events = self.engine.drain_recent_events();
        let serializable: Vec<RecentEvent> = events
            .into_iter()
            .map(|(time, event)| RecentEvent {
                time: time.millis(),
                event,
            })
            .collect();
        serde_json::to_string(&serializable).unwrap_or_else(|_| "[]".into())
    }

    /// Number of pending events in the scheduler queue.
    pub fn pending_events(&self) -> usize {
        self.engine.pending()
    }

    /// Inject a failure mid-simulation. Takes a JSON string representing
    /// a `FailureInjection` enum variant.
    pub fn inject_failure(&mut self, json: &str) -> Result<(), JsValue> {
        let failure: simulation_core::FailureInjection = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Invalid failure injection: {}", e)))?;
        self.engine.inject_failure(&failure);
        Ok(())
    }
}

/// Wrapper for serialising recent events.
#[derive(serde::Serialize)]
struct RecentEvent {
    time: u64,
    event: Event,
}
