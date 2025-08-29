//! Error types for kernel building operations

use std::{io, path::PathBuf};
use thiserror::Error;

/// Errors that can occur during kernel building
#[derive(Error, Debug)]
pub enum KernelError {
    /// I/O error during file operations
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Command execution failed
    ///
    /// This error occurs when a subprocess command fails to execute successfully.
    /// It includes the command that was run, the exit code, and any error output.
    #[error("Command '{command}' failed with code {code}: {message}")]
    CommandError {
        /// The full command string that was executed
        command: String,
        
        /// The exit status code returned by the command
        ///
        /// A non-zero exit code typically indicates failure.
        code: i32,
        
        /// The error output (stderr) from the command
        ///
        /// This contains any error messages or diagnostic information
        /// that was output by the command to stderr.
        message: String,
    },

    /// Missing required file or directory
    #[error("Missing required file or directory: {0}")]
    MissingFile(PathBuf),

    /// Invalid kernel configuration
    #[error("Invalid kernel configuration: {0}")]
    InvalidConfig(String),

    /// Build process failed
    #[error("Build failed: {0}")]
    BuildFailed(String),

    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
}

impl KernelError {
    /// Create a new command error
    pub fn command_error<S: Into<String>>(command: S, output: &std::process::Output) -> Self {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Self::CommandError {
            command: command.into(),
            code: output.status.code().unwrap_or(-1),
            message,
        }
    }
}
