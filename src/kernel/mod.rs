//! Kernel building and management module

mod build;
pub mod config;
mod error;

pub use build::KernelBuilder;
pub use config::{KernelConfig, KernelProfile};
pub use error::KernelError;

/// Re-export commonly used types
pub type Result<T> = std::result::Result<T, KernelError>;
