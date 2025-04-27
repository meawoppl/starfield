//! Python-Rust bridge for testing against the Python Skyfield implementation
//!
//! This module is only included when the `python-tests` feature is enabled and
//! provides utilities for comparing Rust calculations with the reference Python
//! implementation.

pub mod bridge;
pub mod helpers;

// Re-export main components
pub use bridge::PyRustBridge;
pub use helpers::{BridgeError, PythonResult};
