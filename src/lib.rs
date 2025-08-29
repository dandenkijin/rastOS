//! rastOS - A modern, safe Linux distribution built in Rust
//!
//! This library provides the core functionality for the rastOS system,
//! including container management, package management, system installation,
//! and snapshot management.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
//#![forbid(unsafe_code)]

/// Authentication and authorization module
/// 
/// Provides a unified interface for API key authentication across different services.
pub mod auth {
    pub use crate::auth_internal::*;
}

mod auth_internal {
    //! Internal implementation of authentication functionality
    //! 
    //! This module is re-exported by the parent `auth` module.
    
    /// API key management and validation
    pub mod api_key;
    
    /// Configuration for authentication
    pub mod config;
    
    /// Command-line interface for managing API keys
    pub mod cli;
    
    // Re-export the main types for convenience
    pub use api_key::{ApiKey, ApiKeyManager, AuthError};
    pub use config::{ApiKeyConfig, ConfigError};
    pub use cli::{ApiKeyCommand, AddKeyArgs, RemoveKeyArgs, GenerateKeyArgs};
}

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
