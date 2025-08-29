//! OCI (Open Container Initiative) Runtime Implementation for rastOS
//!
//! This module provides an implementation of the OCI Runtime Specification,
//! allowing rastOS to run containers in a standards-compliant way.

mod container;
mod error;

// Re-export public interfaces
pub use container::{Container, ContainerBuilder, ContainerStatus};
pub use error::ContainerError;

// Re-export oci_spec types for convenience
pub use oci_spec::runtime::{
    LinuxBuilder, ProcessBuilder, RootBuilder, Spec, SpecBuilder
};

/// Type alias for the standard result type with our error type
pub type Result<T> = std::result::Result<T, ContainerError>;
