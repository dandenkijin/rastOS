//! Snapshot management for rastOS
//!
//! This module provides types and functions for managing Btrfs snapshots in a tree structure,
//! allowing for efficient tracking of parent-child relationships between snapshots.

use std::collections::{HashMap, HashSet};
// use std::ffi::CString;
use std::path::{Path, PathBuf};
// use std::ptr;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use uuid::Uuid;

// Import the btrfs module
use btrfsutil::error::{BtrfsUtilError, LibError};
use btrfsutil_sys::*;

// Import local modules
// use crate::fs::btrfs;
/// Re-export BtrfsError for convenience

/// Represents a single system snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Unique identifier for the snapshot
    pub id: Uuid,
    
    /// User-defined name for the snapshot
    pub name: String,
    
    /// Optional description
    pub description: Option<String>,
    
    /// Path to the snapshot in the filesystem
    pub path: PathBuf,
    
    /// Whether the snapshot is read-only
    pub read_only: bool,
    
    /// When the snapshot was created
    pub created_at: DateTime<Utc>,
    
    /// Optional parent snapshot ID
    pub parent_id: Option<Uuid>,
    
    /// IDs of child snapshots
    pub children_ids: Vec<Uuid>,
    
    /// System version or release this snapshot is based on
    pub system_version: Option<String>,
    
    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, String>,
}

impl Snapshot {
    /// Create a new snapshot with the given name and path
    pub fn new<P: AsRef<Path>>(name: &str, path: P, parent: Option<&Snapshot>) -> Self {
        Snapshot {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: None,
            path: path.as_ref().to_path_buf(),
            read_only: true, // Snapshots are read-only by default
            created_at: Utc::now(),
            parent_id: parent.map(|p| p.id),
            children_ids: Vec::new(),
            system_version: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Set the snapshot description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    /// Set the system version
    pub fn with_system_version(mut self, version: &str) -> Self {
        self.system_version = Some(version.to_string());
        self
    }
    
    /// Add metadata key-value pair
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Represents a tree of snapshots with parent-child relationships
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SnapshotTree {
    /// All snapshots in the tree, indexed by ID
    snapshots: HashMap<Uuid, Snapshot>,
    
    /// The root snapshot IDs (snapshots with no parent)
    roots: HashSet<Uuid>,
}

/// Errors that can occur when working with the snapshot tree
#[derive(Debug, Error)]
pub enum SnapshotTreeError {
    /// The specified snapshot ID was not found in the tree
    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(Uuid),
    
    /// The provided path is invalid or inaccessible
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    /// An invalid parent-child relationship was attempted
    #[error("Invalid parent-child relationship")]
    InvalidRelationship,
    
    /// A circular reference was detected in the snapshot tree
    #[error("Circular reference detected")]
    CircularReference,
    
    /// An error occurred in the Btrfs filesystem operations
    #[error(transparent)]
    BtrfsError(#[from] BtrfsUtilError),
    
    /// An I/O error occurred
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl SnapshotTree {
    /// Create a new, empty snapshot tree
    pub fn new() -> Self {
        Self {
            snapshots: HashMap::new(),
            roots: HashSet::new(),
        }
    }
    
    /// Add a new snapshot to the tree
    pub fn add_snapshot(&mut self, snapshot: Snapshot) -> Result<(), SnapshotTreeError> {
        let snapshot_id = snapshot.id;
        
        // Check for duplicate ID
        if self.snapshots.contains_key(&snapshot_id) {
            return Err(SnapshotTreeError::InvalidRelationship);
        }
        
        // If this snapshot has a parent, update the parent's children list
        if let Some(parent_id) = snapshot.parent_id {
            if let Some(parent) = self.snapshots.get_mut(&parent_id) {
                parent.children_ids.push(snapshot_id);
            } else {
                return Err(SnapshotTreeError::SnapshotNotFound(parent_id));
            }
            
            // This is not a root node
            if self.roots.contains(&snapshot_id) {
                self.roots.remove(&snapshot_id);
            }
        } else {
            // This is a root node
            self.roots.insert(snapshot_id);
        }
        
        // Add the snapshot to our collection
        self.snapshots.insert(snapshot_id, snapshot);
        
        Ok(())
    }
    
    /// Get a snapshot by ID
    pub fn get_snapshot(&self, id: &Uuid) -> Option<&Snapshot> {
        self.snapshots.get(id)
    }
    
    /// Get a mutable reference to a snapshot by ID
    pub fn get_snapshot_mut(&mut self, id: &Uuid) -> Option<&mut Snapshot> {
        self.snapshots.get_mut(id)
    }
    
    /// Get all root snapshots (snapshots with no parent)
    pub fn get_roots(&self) -> Vec<&Snapshot> {
        self.roots.iter()
            .filter_map(|id| self.snapshots.get(id))
            .collect()
    }
    
    /// Get all snapshots in the tree
    pub fn get_all_snapshots(&self) -> Vec<&Snapshot> {
        self.snapshots.values().collect()
    }
    
    /// Get the children of a snapshot
    pub fn get_children(&self, parent_id: &Uuid) -> Vec<&Snapshot> {
        self.snapshots.get(parent_id)
            .map(|parent| {
                parent.children_ids.iter()
                    .filter_map(|id| self.snapshots.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get the parent of a snapshot
    pub fn get_parent(&self, child_id: &Uuid) -> Option<&Snapshot> {
        self.snapshots.get(child_id)
            .and_then(|child| child.parent_id.as_ref())
            .and_then(|parent_id| self.snapshots.get(parent_id))
    }
    
    /// Get the full path from a root to a specific snapshot
    pub fn get_path_to_snapshot(&self, id: &Uuid) -> Option<Vec<&Snapshot>> {
        let mut path = Vec::new();
        let mut current_id = *id;
        
        // Walk up the tree until we find a root or a cycle
        let mut visited = HashSet::new();
        
        while let Some(snapshot) = self.snapshots.get(&current_id) {
            // Check for cycles
            if !visited.insert(current_id) {
                return None; // Cycle detected
            }
            
            path.push(snapshot);
            
            match snapshot.parent_id {
                Some(pid) => current_id = pid,
                None => break, // Reached a root
            }
        }
        
        path.reverse(); // So it's from root to the target
        Some(path)
    }
    
    /// Remove a snapshot from the tree
    pub fn remove_snapshot(&mut self, id: &Uuid) -> Result<Snapshot, SnapshotTreeError> {
        // Check if the snapshot exists
        let snapshot = match self.snapshots.get(id) {
            Some(s) => s,
            None => return Err(SnapshotTreeError::SnapshotNotFound(*id)),
        };
        
        // Can't remove a snapshot that has children
        if !snapshot.children_ids.is_empty() {
            return Err(SnapshotTreeError::InvalidRelationship);
        }
        
        // Remove from parent's children list
        if let Some(pid) = snapshot.parent_id {
            // We need to clone the parent_id to avoid holding a mutable reference
            // while we have an immutable one
            if let Some(parent) = self.snapshots.get_mut(&pid) {
                parent.children_ids.retain(|child_id| child_id != id);
            }
        } else {
            // This was a root node
            self.roots.remove(id);
        }
        
        // Remove the snapshot
        Ok(self.snapshots.remove(id).unwrap())
    }
    
    /// Create a new snapshot from an existing one
    pub fn create_snapshot(
        &mut self,
        source_id: &Uuid,
        name: &str,
        dest_path: &Path,
        read_only: bool,
    ) -> Result<Uuid, SnapshotTreeError> {
        // Get the source snapshot
        let source = self.get_snapshot(source_id)
            .ok_or(SnapshotTreeError::SnapshotNotFound(*source_id))?;
        
        // Create the actual Btrfs snapshot
        let source_str = source.path.to_str()
            .ok_or_else(|| SnapshotTreeError::InvalidPath("Invalid source path".to_string()))?;
        let dest_str = dest_path.to_str()
            .ok_or_else(|| SnapshotTreeError::InvalidPath("Invalid destination path".to_string()))?;
            
        let source_cstr = std::ffi::CString::new(source_str)
            .map_err(|e| SnapshotTreeError::InvalidPath(e.to_string()))?;
        let dest_cstr = std::ffi::CString::new(dest_str)
            .map_err(|e| SnapshotTreeError::InvalidPath(e.to_string()))?;
        
        // Convert the FFI result to a proper error
        let result = unsafe {
            btrfs_util_create_snapshot(
                source_cstr.as_ptr(),
                dest_cstr.as_ptr(),
                if read_only { 1 } else { 0 } as i32,
                std::ptr::null_mut(), // flags
                std::ptr::null_mut(), // reserved
            )
        };
        
        if result != 0 {
            // Convert the error code to a BtrfsUtilError
            match LibError::try_from(result as u32) {
                Ok(lib_error) => {
                    log::error!("Btrfs error: {}", lib_error);
                    return Err(SnapshotTreeError::BtrfsError(lib_error.into()));
                },
                Err(e) => {
                    log::error!("Failed to convert Btrfs error code: {}", e);
                    return Err(SnapshotTreeError::BtrfsError(e.into()));
                }
            };
        }
        
        // Create the new snapshot object
        let mut new_snapshot = Snapshot::new(name, dest_path, Some(source));
        new_snapshot.read_only = read_only;
        
        // Copy metadata from parent
        new_snapshot.system_version = source.system_version.clone();
        new_snapshot.metadata = source.metadata.clone();
        
        // Add to the tree
        let new_id = new_snapshot.id;
        self.add_snapshot(new_snapshot)?;
        
        Ok(new_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_snapshot_tree() {
        let mut tree = SnapshotTree::new();
        
        // Create a root snapshot
        let root = Snapshot::new("root", "/snapshots/root", None);
        let root_id = root.id;
        tree.add_snapshot(root).unwrap();
        
        // Create a child snapshot from the root
        let child = Snapshot::new("child", "/snapshots/child", tree.get_snapshot(&root_id))
            .with_description("Test child snapshot")
            .with_system_version("1.0.0");
            
        let child_id = child.id;
        tree.add_snapshot(child).unwrap();
        
        // Verify the tree structure
        assert_eq!(tree.get_roots().len(), 1);
        assert_eq!(tree.get_children(&root_id).len(), 1);
        assert_eq!(tree.get_parent(&child_id).unwrap().id, root_id);
        
        // Test path finding
        let path = tree.get_path_to_snapshot(&child_id).unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].id, root_id);
        assert_eq!(path[1].id, child_id);
        
        // Test removal
        tree.remove_snapshot(&child_id).unwrap();
        assert!(tree.get_snapshot(&child_id).is_none());
        assert!(tree.get_children(&root_id).is_empty());
    }
    
    /// Test the creation of a snapshot and its addition to the snapshot tree.
    ///
    /// This test verifies that:
    /// 1. A root snapshot can be created and added to the tree
    /// 2. A child snapshot can be created from the root
    /// 3. The child snapshot is properly registered in the tree
    /// 4. The parent-child relationship is correctly maintained
    #[test]
    fn test_create_snapshot() {
        let mut tree = SnapshotTree::new();
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        
        // Create a mock root directory
        let root_path = temp_dir.path().join("root");
        std::fs::create_dir_all(&root_path).expect("Failed to create root dir");
        
        // Create a root snapshot
        let root = Snapshot::new("root", &root_path, None)
            .with_description("Test root snapshot")
            .with_system_version("1.0.0");
            
        let root_id = root.id;
        tree.add_snapshot(root).unwrap();
        
        // Create a mock child directory
        let child_path = temp_dir.path().join("child");
        std::fs::create_dir_all(&child_path).expect("Failed to create child dir");
        
        // Add the child to the tree
        let child = Snapshot::new("child", &child_path, Some(&tree.get_snapshot(&root_id).unwrap()));
        let child_id = child.id;
        tree.add_snapshot(child).unwrap();
        
        // Verify the snapshot was added to the tree
        assert!(tree.get_snapshot(&child_id).is_some());
        assert_eq!(tree.get_parent(&child_id).unwrap().id, root_id);
    }
}
