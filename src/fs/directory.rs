//! Directory operations for rastOS
//!
//! This module provides a safe and ergonomic interface for working with directories.
//! It includes functionality for creating, listing, and removing directories, as well as
//! recursive operations and directory traversal.
//!
//! # Features
//!
//! - **Directory Management**: Create, list, and remove directories
//! - **Recursive Operations**: Recursively create and remove directory trees
//! - **Directory Traversal**: Iterate over directory contents
//! - **Error Handling**: Detailed error information for all operations
//!
//! # Examples
//!
//! ```no_run
//! use rastos::fs;
//! use std::path::Path;
//!
//! fn main() -> Result<(), fs::FsError> {
//!     // Create a new directory
//!     fs::create_dir("my_dir")?;
//!     
//!     // Create a file in the directory
//!     fs::write("my_dir/file.txt", "Hello")?;
//!     
//!     // List directory contents
//!     for entry in fs::list_dir("my_dir")? {
//!         println!("Found: {}", entry.display());
//!     }
//!     
//!     // Create nested directories
//!     fs::create_dir_all("my_dir/nested/deep")?;
//!     
//!     // Remove the directory and all its contents
//!     fs::remove_dir_all("my_dir")?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Directory Traversal
//!
//! The `list_dir` function returns a vector of `PathBuf`s for each entry in the directory.
//! For more advanced directory traversal, consider using the `walkdir` crate which provides
//! recursive directory iteration with more control over the traversal process.


use std::fs::{self, ReadDir};
use std::path::{Path, PathBuf};

use super::{FsError, Result};

/// Directory operations trait
pub trait DirectoryOps {
    /// List directory contents
    fn list(&self) -> Result<Vec<PathBuf>>;
    
    /// Create a new directory
    fn create(&self) -> Result<()>;
    
    /// Remove an empty directory
    fn remove(&self) -> Result<()>;
    
    /// Remove a directory and all its contents
    fn remove_all(&self) -> Result<()>;
    
    /// Get the directory path
    fn path(&self) -> &Path;
}

/// Standard directory implementation
#[derive(Debug)]
pub struct StdDirectory {
    path: PathBuf,
}

impl StdDirectory {
    /// Open a directory
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(FsError::not_found(path));
        }
        if !path.is_dir() {
            return Err(FsError::invalid_path("Not a directory"));
        }
        Ok(Self {
            path: path.to_path_buf(),
        })
    }
    
    /// Create a new directory
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if path.exists() {
            return Err(FsError::already_exists(path));
        }
        fs::create_dir(path)?;
        Ok(Self {
            path: path.to_path_buf(),
        })
    }
    
    /// Create a directory and all its parents if they don't exist
    pub fn create_all<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(path)?;
        } else if !path.is_dir() {
            return Err(FsError::invalid_path("Path exists but is not a directory"));
        }
        Ok(Self {
            path: path.to_path_buf(),
        })
    }
}

impl DirectoryOps for StdDirectory {
    fn list(&self) -> Result<Vec<PathBuf>> {
        let entries = fs::read_dir(&self.path)?;
        let paths: Result<Vec<_>> = entries
            .map(|entry| {
                let entry = entry?;
                Ok(entry.path())
            })
            .collect();
        paths
    }
    
    fn create(&self) -> Result<()> {
        fs::create_dir(&self.path).map_err(Into::into)
    }
    
    fn remove(&self) -> Result<()> {
        fs::remove_dir(&self.path).map_err(Into::into)
    }
    
    fn remove_all(&self) -> Result<()> {
        fs::remove_dir_all(&self.path).map_err(Into::into)
    }
    
    fn path(&self) -> &Path {
        &self.path
    }
}

/// List directory contents
pub fn list_dir<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>> {
    let dir = StdDirectory::open(path)?;
    dir.list()
}

/// Create a new directory
pub fn create_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    StdDirectory::create(path).map(|_| ())
}

/// Create a directory and all its parents if they don't exist
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    StdDirectory::create_all(path).map(|_| ())
}

/// Remove an empty directory
pub fn remove_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let dir = StdDirectory::open(path)?;
    dir.remove()
}

/// Remove a directory and all its contents
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    let dir = StdDirectory::open(path)?;
    dir.remove_all()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_directory_operations() -> Result<()> {
        let dir = tempdir()?;
        let dir_path = dir.path().join("test_dir");
        
        // Test creating a directory
        create_dir(&dir_path)?;
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
        
        // Test listing directory contents
        let file_path = dir_path.join("test.txt");
        File::create(&file_path)?;
        
        let mut entries = list_dir(&dir_path)?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], file_path);
        
        // Test removing directory with contents
        let result = remove_dir(&dir_path);
        assert!(result.is_err()); // Should fail because directory is not empty
        
        // Test removing directory and contents
        remove_dir_all(&dir_path)?;
        assert!(!dir_path.exists());
        
        // Test creating nested directories
        let nested_path = dir.path().join("a/b/c");
        create_dir_all(&nested_path)?;
        assert!(nested_path.exists());
        
        // Test opening a directory
        let dir = StdDirectory::open(dir.path())?;
        assert_eq!(dir.path(), dir.path());
        
        Ok(())
    }
}
