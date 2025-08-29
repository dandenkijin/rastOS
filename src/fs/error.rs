//! Error types and utilities for file system operations
//!
//! This module defines the error types used throughout the file system operations.
//! It provides a rich error type ([`FsError`]) that can represent all possible
//! file system related errors with detailed context.
//!
//! # Examples
//!
//! ```
//! use rastos::fs::FsError;
//! use std::path::Path;
//!
//! // Create a custom error
//! let not_found = FsError::not_found("/path/to/nonexistent/file.txt");
//! assert!(matches!(not_found, FsError::NotFound(_)));
//!
//! // Convert from std::io::Error
//! let io_error = std::io::Error::new(
//!     std::io::ErrorKind::PermissionDenied,
//!     "permission denied"
//! );
//! let fs_error: FsError = io_error.into();
//! assert!(matches!(fs_error, FsError::PermissionDenied(_)));
//! ```

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during file system operations
#[derive(Debug, Error)]
pub enum FsError {
    /// I/O error from the standard library
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// File or directory not found
    #[error("Not found: {0}")]
    NotFound(PathBuf),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    /// File already exists
    #[error("File already exists: {0}")]
    AlreadyExists(PathBuf),

    /// Invalid path or path operation
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Operation not supported
    #[error("Operation not supported: {0}")]
    NotSupported(String),

    /// Directory not empty
    #[error("Directory not empty: {0}")]
    DirectoryNotEmpty(PathBuf),

    /// Other errors
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl FsError {
    /// Create a new not found error
    pub fn not_found<P: Into<PathBuf>>(path: P) -> Self {
        Self::NotFound(path.into())
    }

    /// Create a new permission denied error
    pub fn permission_denied<P: Into<PathBuf>>(path: P) -> Self {
        Self::PermissionDenied(path.into())
    }

    /// Create a new already exists error
    pub fn already_exists<P: Into<PathBuf>>(path: P) -> Self {
        Self::AlreadyExists(path.into())
    }

    /// Create a new invalid path error
    pub fn invalid_path<S: Into<String>>(msg: S) -> Self {
        Self::InvalidPath(msg.into())
    }

    /// Create a new not supported error
    pub fn not_supported<S: Into<String>>(msg: S) -> Self {
        Self::NotSupported(msg.into())
    }

    /// Create a new directory not empty error
    pub fn directory_not_empty<P: Into<PathBuf>>(path: P) -> Self {
        Self::DirectoryNotEmpty(path.into())
    }
}

impl From<FsError> for std::io::Error {
    fn from(err: FsError) -> Self {
        match err {
            FsError::Io(e) => e,
            _ => std::io::Error::new(std::io::ErrorKind::Other, err),
        }
    }
}
