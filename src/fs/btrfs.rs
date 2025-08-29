//! Btrfs subvolume and snapshot operations
//!
//! This module provides functions for working with Btrfs subvolumes and snapshots
//! using the `btrfsutil-rs` crate.

use std::path::{Path, PathBuf};
use thiserror::Error;
use btrfsutil_rs::{BtrfsUtil, SubvolumeInfo, BtrfsUtilError};

use crate::fs::FsError;

/// Errors that can occur during Btrfs operations
#[derive(Debug, Error)]
pub enum BtrfsError {
    #[error("Btrfs operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Invalid subvolume path: {0}")]
    InvalidPath(String),
    
    #[error("Btrfs error: {0}")]
    BtrfsUtil(#[from] BtrfsUtilError),
    
    #[error("Subvolume not found: {0}")]
    SubvolumeNotFound(PathBuf),
    
    #[error("Path is not a valid UTF-8 string")]
    InvalidUtf8,
    
    #[error(transparent)]
    Io(#[from] std::io::Error),
    
    #[error("Invalid subvolume ID: {0}")]
    InvalidSubvolumeId(u64),
    
    #[error(transparent)]
    Btrfs(#[from] BtrfsCrateError),
    
    #[error(transparent)]
    Fs(#[from] FsError),
}

/// Result type for Btrfs operations
pub type Result<T> = std::result::Result<T, BtrfsError>;

/// Check if a path is a Btrfs subvolume
pub fn is_subvolume<P: AsRef<Path>>(path: P) -> bool {
    BtrfsUtil::is_subvolume(path).unwrap_or(false)
}

/// Create a new Btrfs subvolume
pub fn create_subvolume<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(BtrfsError::Io)?;
    }
    
    BtrfsUtil::create_subvolume(path, false, None)
        .map_err(|e| BtrfsError::OperationFailed(format!(
            "Failed to create subvolume at {}: {}", 
            path.display(), 
            e
        )))?;
    
    Ok(())
}

/// Create a read-only snapshot of a subvolume
pub fn create_snapshot<S: AsRef<Path>, D: AsRef<Path>>(
    source: S,
    dest: D,
    read_only: bool,
) -> Result<()> {
    let source = source.as_ref();
    let dest = dest.as_ref();
    
    // Create parent directories if they don't exist
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(BtrfsError::Io)?;
    }
    
    BtrfsUtil::create_snapshot(source, dest, read_only, None)
        .map_err(|e| BtrfsError::OperationFailed(format!(
            "Failed to create snapshot from {} to {}: {}", 
            source.display(), 
            dest.display(), 
            e
        )))?;
    
    Ok(())
}

/// Delete a subvolume or snapshot
pub fn delete_subvolume<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    
    if !is_subvolume(path) {
        return Err(BtrfsError::SubvolumeNotFound(path.to_path_buf()));
    }
    
    BtrfsUtil::delete_subvolume(path, None)
        .map_err(|e| BtrfsError::OperationFailed(format!(
            "Failed to delete subvolume at {}: {}", 
            path.display(), 
            e
        )))
}

/// List all subvolumes under a given path
pub fn list_subvolumes<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>> {
    let subvols = BtrfsUtil::list_subvolumes(path, true, None)
        .map_err(|e| BtrfsError::OperationFailed(e.to_string()))?;
    
    Ok(subvols.into_iter()
        .map(|(_, path)| PathBuf::from(path))
        .collect())
}

/// Get information about a subvolume
pub fn get_subvolume_info<P: AsRef<Path>>(path: P) -> Result<SubvolumeInfo> {
    let path = path.as_ref();
    BtrfsUtil::subvolume_info(path, None)
        .map_err(|e| BtrfsError::OperationFailed(format!(
            "Failed to get subvolume info for {}: {}", 
            path.display(), 
            e
        )))
}

/// Set a subvolume as read-only or read-write
pub fn set_subvolume_readonly<P: AsRef<Path>>(path: P, read_only: bool) -> Result<()> {
    let path = path.as_ref();
    
    if !is_subvolume(path) {
        return Err(BtrfsError::SubvolumeNotFound(path.to_path_buf()));
    }
    
    BtrfsUtil::set_subvolume_read_only(path, read_only, None)
        .map_err(|e| BtrfsError::OperationFailed(format!(
            "Failed to set read-only flag for {}: {}", 
            path.display(), 
            e
        )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_create_and_delete_subvolume() {
        let temp_dir = tempdir().unwrap();
        let subvol_path = temp_dir.path().join("test_subvol");
        
        // Create a new subvolume
        create_subvolume(&subvol_path).unwrap();
        assert!(is_subvolume(&subvol_path));
        
        // Clean up
        delete_subvolume(&subvol_path).unwrap();
        assert!(!is_subvolume(&subvol_path));
    }
    
    #[test]
    fn test_create_snapshot() {
        let temp_dir = tempdir().unwrap();
        let subvol_path = temp_dir.path().join("test_subvol");
        let snapshot_path = temp_dir.path().join("test_snapshot");
        
        // Create a subvolume and a snapshot
        create_subvolume(&subvol_path).unwrap();
        create_snapshot(&subvol_path, &snapshot_path, false).unwrap();
        
        // Both should be valid subvolumes
        assert!(is_subvolume(&subvol_path));
        assert!(is_subvolume(&snapshot_path));
        
        // Clean up
        delete_subvolume(&snapshot_path).unwrap();
        delete_subvolume(&subvol_path).unwrap();
    }
    
    #[test]
    fn test_set_subvolume_readonly() {
        let temp_dir = tempdir().unwrap();
        let subvol_path = temp_dir.path().join("test_subvol");
        
        // Create a new subvolume
        create_subvolume(&subvol_path).unwrap();
        
        // Set to read-only
        set_subvolume_readonly(&subvol_path, true).unwrap();
        
        // Verify it's read-only
        let info = get_subvolume_info(&subvol_path).unwrap();
        assert!(info.readonly);
        
        // Set back to read-write
        set_subvolume_readonly(&subvol_path, false).unwrap();
        
        // Verify it's read-write
        let info = get_subvolume_info(&subvol_path).unwrap();
        assert!(!info.readonly);
        
        // Clean up
        delete_subvolume(&subvol_path).unwrap();
    }
}
