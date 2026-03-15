//! simulation-core: the deterministic discrete-event simulation engine.
//!
//! This crate contains the core simulation logic with no platform
//! dependencies. It is compiled to both native (for testing) and
//! WebAssembly (for the browser).

pub mod engine;
pub mod network;
pub mod rng;
pub mod routing;
pub mod scheduler;
pub mod traffic;
pub mod types;

pub use engine::*;
pub use network::*;
pub use rng::*;
pub use routing::*;
pub use scheduler::*;
pub use traffic::*;
pub use types::*;
