//! rastOS - A modern, safe Linux distribution built in Rust
//!
//! This library provides the core functionality for the rastOS system,
//! including container management, package management, system installation,
//! and snapshot management.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
//#![forbid(unsafe_code)]

/// OCI (Open Container Initiative) runtime implementation
pub mod oci;

// Other core modules
pub mod installer;
pub mod kernel;
pub mod package;
pub mod snapshot;
pub mod system;

// Re-export commonly used types
pub use oci::*;

/// Type alias for the standard result type with our error type
pub type Result<T> = std::result::Result<T, oci::ContainerError>;
