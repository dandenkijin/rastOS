//! File operations for rastOS
//!
//! This module provides high-level file operations including copying, moving, deleting,
//! reading, and writing files. All operations are designed to be safe, atomic where possible,
//! and provide detailed error information.
//!
//! # Features
//!
//! - **Atomic Operations**: Where possible, operations are performed atomically
//! - **Cross-Platform**: Consistent behavior across different operating systems
//! - **Detailed Errors**: Rich error information including paths and operation context
//! - **Automatic Cleanup**: Uses RAII patterns to ensure resources are properly cleaned up
//!
//! # Examples
//!
//! ```no_run
//! use rastos::fs;
//! use std::path::Path;
//!
//! fn main() -> Result<(), fs::FsError> {
//!     // Write data to a file
//!     fs::write("data.txt", "Hello, world!")?;
//!     
//!     // Read data from a file
//!     let content = fs::read_to_string("data.txt")?;
//!     println!("Read: {}", content);
//!     
//!     // Copy a file
//!     fs::copy_file("data.txt", "data_copy.txt")?;
//!     
//!     // Move a file
//!     fs::move_file("data_copy.txt", "moved_data.txt")?;
//!     
//!     // Delete a file
//!     fs::delete_file("moved_data.txt")?;
//!     
//!     // Clean up
//!     fs::delete_file("data.txt")?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Atomicity
//!
//! - `write`: Atomic on most platforms when the target file doesn't exist
//! - `move_file`: Atomic on the same filesystem, falls back to copy+delete across filesystems
//! - `copy_file`: Not guaranteed to be atomic
//! - `delete_file`: Atomic on all platforms

use std::fs;
use std::path::{Path, PathBuf};
use std::io::{self, Read, Write};

use super::{FsError, Result};
use super::metadata::metadata;

/// Copy a file from source to destination
pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<u64> {
    let from = from.as_ref();
    let to = to.as_ref();
    
    // Check if source exists
    if !from.exists() {
        return Err(FsError::not_found(from));
    }
    
    // Check if source is a file
    if !from.is_file() {
        return Err(FsError::invalid_path("Source is not a file"));
    }
    
    // If destination is a directory, copy into it with the same filename
    let dest_path = if to.is_dir() {
        let file_name = from.file_name()
            .ok_or_else(|| FsError::invalid_path("Invalid source filename"))?;
        to.join(file_name)
    } else {
        to.to_path_buf()
    };
    
    // Create parent directories if they don't exist
    if let Some(parent) = dest_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    
    // Perform the copy
    fs::copy(from, &dest_path)
        .map_err(|e| {
            if e.kind() == io::ErrorKind::PermissionDenied {
                FsError::permission_denied(&dest_path)
            } else {
                e.into()
            }
        })
}

/// Move a file or directory from source to destination
pub fn move_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    
    // Check if source exists
    if !from.exists() {
        return Err(FsError::not_found(from));
    }
    
    // Handle directory vs file moves
    if from.is_dir() {
        // For directories, we need to handle the move differently
        // First try a simple rename, which is atomic
        match fs::rename(from, to) {
            Ok(_) => return Ok(()),
            Err(_) => {
                // If rename fails (cross-device move), fall back to copy + delete
                copy_dir_all(from, to)?;
                fs::remove_dir_all(from)?;
            }
        }
    } else {
        // For files, try a simple rename first
        match fs::rename(from, to) {
            Ok(_) => return Ok(()),
            Err(_) => {
                // If rename fails (cross-device move), fall back to copy + delete
                copy_file(from, to)?;
                fs::remove_file(from)?;
            }
        }
    }
    
    Ok(())
}

/// Delete a file
pub fn delete_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    
    if !path.exists() {
        return Err(FsError::not_found(path));
    }
    
    if path.is_dir() {
        return Err(FsError::invalid_path("Path is a directory, use remove_dir instead"));
    }
    
    fs::remove_file(path).map_err(Into::into)
}

/// Copy a directory and all its contents recursively
fn copy_dir_all<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    
    // Create the destination directory
    fs::create_dir_all(to)?;
    
    // Copy each entry in the directory
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = to.join(entry.file_name());
        
        if file_type.is_dir() {
            copy_dir_all(entry.path(), target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    
    Ok(())
}

/// Read the entire contents of a file into a string
pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = fs::File::open(path.as_ref())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Write a string to a file, creating it if it doesn't exist
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    let path = path.as_ref();
    
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    
    fs::write(path, contents).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_file_operations() -> Result<()> {
        let dir = tempdir()?;
        let src_file = dir.path().join("test.txt");
        let dest_file = dir.path().join("test_copy.txt");
        
        // Test write and read
        let test_content = "Hello, world!";
        write(&src_file, test_content)?;
        assert_eq!(read_to_string(&src_file)?, test_content);
        
        // Test copy
        let bytes_copied = copy_file(&src_file, &dest_file)?;
        assert_eq!(bytes_copied, test_content.len() as u64);
        assert_eq!(read_to_string(&dest_file)?, test_content);
        
        // Test move
        let moved_file = dir.path().join("test_moved.txt");
        move_file(&dest_file, &moved_file)?;
        assert!(!dest_file.exists());
        assert_eq!(read_to_string(&moved_file)?, test_content);
        
        // Test delete
        delete_file(&moved_file)?;
        assert!(!moved_file.exists());
        
        Ok(())
    }
    
    #[test]
    fn test_directory_operations() -> Result<()> {
        let dir = tempdir()?;
        let src_dir = dir.path().join("src");
        let dest_dir = dir.path().join("dest");
        
        // Create a directory with a file
        fs::create_dir(&src_dir)?;
        let src_file = src_dir.join("test.txt");
        write(&src_file, "test")?;
        
        // Test moving a directory
        move_file(&src_dir, &dest_dir)?;
        assert!(!src_dir.exists());
        assert!(dest_dir.exists());
        assert!(dest_dir.join("test.txt").exists());
        
        Ok(())
    }
}
