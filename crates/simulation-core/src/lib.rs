//! simulation-core: the deterministic discrete-event simulation engine.
//!
//! This crate contains the core simulation logic with no platform
//! dependencies. It is compiled to both native (for testing) and
//! WebAssembly (for the browser).

pub mod engine;
pub mod scheduler;
pub mod types;

pub use engine::*;
pub use scheduler::*;
pub use types::*;
