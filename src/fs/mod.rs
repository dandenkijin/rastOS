//! Safe and ergonomic file system operations for rastOS
//!
//! This module provides a high-level, safe API for file system operations with proper error handling
//! and cross-platform support. It wraps the standard library's file system operations with additional
//! safety checks, better error messages, and a more ergonomic interface.
//!
//! # Features
//!
//! - **File Operations**: Create, read, write, copy, move, and delete files
//! - **Directory Operations**: Create, list, and remove directories
//! - **Metadata**: Get and set file metadata (permissions, timestamps, etc.)
//! - **Error Handling**: Comprehensive error types with detailed error messages
//! - **Cross-platform**: Works consistently across different operating systems
//!
//! # Examples
//!
//! ```no_run
//! use rastos::fs;
//! use std::path::Path;
//!
//! fn main() -> Result<(), fs::FsError> {
//!     // Create a new file
//!     fs::write("example.txt", "Hello, world!")?;
//!     
//!     // Read the file
//!     let content = fs::read_to_string("example.txt")?;
//!     println!("File content: {}", content);
//!     
//!     // Create a directory
//!     fs::create_dir("my_dir")?;
//!     
//!     // Copy the file
//!     fs::copy_file("example.txt", "my_dir/example_copy.txt")?;
//!     
//!     // List directory contents
//!     for entry in fs::list_dir("my_dir")? {
//!         println!("Found: {}", entry.display());
//!     }
//!     
//!     // Clean up
//!     fs::delete_file("example.txt")?;
//!     fs::remove_dir_all("my_dir")?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Error Handling
//!
//! All operations return a `Result<T, FsError>` where `FsError` provides detailed information
//! about what went wrong. The error type includes the operation that failed, the path involved,
//! and the underlying system error if any.

mod btrfs;
mod directory;
mod error;
mod file;
mod file_ops;
mod metadata;
mod utils;

pub use error::FsError;
pub use file::FileOps;
pub use file_ops::{copy_file, move_file, delete_file, read_to_string, write};
pub use metadata::Metadata;
pub use btrfs::{
    create_subvolume, create_snapshot, delete_subvolume, list_subvolumes,
    set_subvolume_readonly, is_subvolume, BtrfsError
};
pub use directory::{DirectoryOps, list_dir, create_dir, create_dir_all, remove_dir, remove_dir_all};
pub use utils::{
    glob, glob_with_options, GlobOptions,
    temp_file, temp_dir, create_temp_file,
};

/// Type alias for the standard result type with our error type
pub type Result<T> = std::result::Result<T, FsError>;
