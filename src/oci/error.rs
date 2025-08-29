//! Error types for OCI container operations

use std::fmt;
use thiserror::Error;

/// Errors that can occur during container operations
#[derive(Debug, Error)]
pub enum ContainerError {
    /// Container not found
    #[error("Container not found: {0}")]
    NotFound(String),
    
    /// Container already exists
    #[error("Container already exists: {0}")]
    AlreadyExists(String),
    
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    /// Runtime error
    #[error("Runtime error: {0}")]
    Runtime(String),
    
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// OCI spec error
    #[error("OCI spec error: {0}")]
    OciSpec(#[from] oci_spec::OciSpecError),
    
    /// Other errors
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Implement std::convert::From for std::fmt::Error to ContainerError
impl From<fmt::Error> for ContainerError {
    fn from(err: fmt::Error) -> Self {
        ContainerError::Runtime(format!("Format error: {}", err))
    }
}
