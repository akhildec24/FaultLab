//! WebAssembly bindings for the simulation engine.
//!
//! This crate exposes a clean JavaScript interface to the Rust simulation
//! engine via wasm-bindgen. It deliberately hides internal Rust structures.

use simulation_core::{Engine, Metrics, Scenario, TrafficConfig};
use wasm_bindgen::prelude::*;

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
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            engine: Engine::new(Scenario {
                name: "empty".into(),
                nodes: vec![],
                connections: vec![],
                traffic: TrafficConfig {
                    start_rps: 0,
                    target_rps: 0,
                    ramp_seconds: 0,
                },
                seed: 0,
            }),
        }
    }

    pub fn load_scenario(&mut self, json: &str) -> Result<(), JsValue> {
        let scenario: Scenario = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Invalid scenario: {}", e)))?;
        self.engine = Engine::load(scenario);
        Ok(())
    }

    pub fn start(&mut self) {
        self.engine.start();
    }

    pub fn pause(&mut self) {
        self.engine.pause();
    }

    pub fn reset(&mut self) {
        self.engine.reset();
    }

    pub fn step(&mut self) -> bool {
        self.engine.step()
    }

    pub fn is_running(&self) -> bool {
        self.engine.is_running()
    }

    pub fn current_time(&self) -> u64 {
        self.engine.now().millis()
    }

    pub fn get_metrics(&self) -> String {
        let metrics: &Metrics = self.engine.metrics();
        serde_json::to_string(metrics).unwrap_or_else(|_| "{}".into())
    }
}
