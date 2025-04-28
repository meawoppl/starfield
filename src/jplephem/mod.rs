//! JPL Ephemeris module for high-precision planetary positions
//!
//! This module provides functionality for reading and interpreting JPL Development 
//! Ephemerides (DE) files, which contain high-precision position and velocity data
//! for solar system bodies.
//!
//! # Overview
//!
//! JPL Ephemerides are distributed as binary SPK (Spacecraft Planet Kernel) files
//! in the SPICE format. This module provides Rust implementations of the file readers
//! and algorithms necessary to extract planetary positions from these files.
//!
//! # Main Components
//!
//! - `daf`: Double Array File format reader (underlying format of SPK files)
//! - `spk`: Spacecraft Planet Kernel format reader
//! - `pck`: Planetary Constants Kernel format reader (for rotation data)
//! - `names`: Mappings between celestial body names and ID numbers
//! - Error types for proper error handling

pub mod errors;
pub mod daf;
pub mod spk;
pub mod pck;
pub mod names;
pub mod calendar;

#[cfg(test)]
mod tests;

// Re-export primary types for convenience
pub use self::errors::JplephemError;
pub use self::spk::SPK;
pub use self::pck::PCK;