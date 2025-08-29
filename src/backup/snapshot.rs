//! Snapshot management for rastOS backups

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::backup::btrfs::{self, BtrfsError, Subvolume};

/// Represents a snapshot in the backup system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Unique identifier for the snapshot
    pub id: String,
    
    /// Name of the subvolume being snapshotted
    pub subvolume: String,
    
    /// Path to the snapshot
    pub path: PathBuf,
    
    /// Whether this is a read-only snapshot
    pub read_only: bool,
    
    /// Parent snapshot ID (for incremental backups)
    pub parent_id: Option<String>,
    
    /// Creation time
    pub created_at: DateTime<Utc>,
    
    /// Size in bytes
    pub size: u64,
    
    /// Optional description or tags
    pub metadata: HashMap<String, String>,
}

impl Snapshot {
    /// Create a new snapshot of a subvolume
    pub async fn create<P: AsRef<Path>>(
        subvolume: P,
        snapshot_dir: P,
        read_only: bool,
        parent: Option<&Snapshot>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<Self> {
        let subvolume = subvolume.as_ref();
        let snapshot_dir = snapshot_dir.as_ref();
        
        // Create snapshot directory if it doesn't exist
        if !snapshot_dir.exists() {
            fs::create_dir_all(snapshot_dir).await?;
        }
        
        // Generate a unique snapshot ID
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let snapshot_name = format!(
            "{}_{}",
            subvolume
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("subvol"),
            timestamp
        );
        
        let snapshot_path = snapshot_dir.join(&snapshot_name);
        
        // Create the BTRFS snapshot
        let btrfs_snapshot = if let Some(parent_snapshot) = parent {
            btrfs::Subvolume::create_snapshot(
                &parent_snapshot.path,
                &snapshot_path,
                read_only,
            )
            .await
            .context("Failed to create incremental snapshot")?
        } else {
            btrfs::Subvolume::create_snapshot(subvolume, &snapshot_path, read_only)
                .await
                .context("Failed to create full snapshot")?
        };
        
        // Create the snapshot metadata
        let snapshot = Self {
            id: format!("snap_{}", uuid::Uuid::new_v4()),
            subvolume: subvolume.to_string_lossy().into_owned(),
            path: snapshot_path,
            read_only: btrfs_snapshot.read_only,
            parent_id: parent.map(|p| p.id.clone()),
            created_at: btrfs_snapshot.created_at,
            size: btrfs_snapshot.size,
            metadata: metadata.unwrap_or_default(),
        };
        
        Ok(snapshot)
    }
    
    /// Delete this snapshot
    pub async fn delete(&self) -> Result<()> {
        if self.path.exists() {
            btrfs::Subvolume::delete(&self.path).await?;
        }
        Ok(())
    }
    
    /// Send this snapshot to a file or stream
    pub async fn send<P: AsRef<Path>>(&self, output: P) -> Result<()> {
        let subvol = btrfs::Subvolume::from_path(&self.path).await?;
        subvol.send(Some(output)).await?;
        Ok(())
    }
    
    /// Restore this snapshot to a target path
    pub async fn restore<P: AsRef<Path>>(&self, target: P) -> Result<()> {
        // If target exists, it must be a subvolume
        if target.as_ref().exists() {
            if !btrfs::Subvolume::is_subvolume(&target).await? {
                return Err(anyhow::anyhow!("Target exists and is not a subvolume"));
            }
            
            // Delete the target subvolume
            btrfs::Subvolume::delete(&target).await?;
        }
        
        // Create a new snapshot from our snapshot
        let restored = btrfs::Subvolume::create_snapshot(&self.path, &target, false).await?;
        
        // If the original was read-only, make the restored one read-write
        if self.read_only && restored.read_only {
            // We would need to remount with -o remount,rw or use btrfs property set
            // This is a simplification - in a real implementation, we'd handle this properly
            log::warn!("Restored a read-only snapshot as read-write - ensure proper permissions are set");
        }
        
        Ok(())
    }
}

/// Manages snapshots for backup purposes
pub struct SnapshotManager {
    /// Directory where snapshots are stored
    snapshot_dir: PathBuf,
    
    /// Whether to create read-only snapshots by default
    read_only: bool,
    
    /// Metadata to include in new snapshots
    default_metadata: HashMap<String, String>,
}

impl SnapshotManager {
    /// Create a new SnapshotManager
    pub fn new<P: Into<PathBuf>>(snapshot_dir: P) -> Self {
        Self {
            snapshot_dir: snapshot_dir.into(),
            read_only: true,
            default_metadata: HashMap::new(),
        }
    }
    
    /// Set whether snapshots should be read-only
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    
    /// Add default metadata to all new snapshots
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_metadata.insert(key.into(), value.into());
        self
    }
    
    /// Create a new snapshot of a subvolume
    pub async fn create_snapshot<P: AsRef<Path>>(
        &self,
        subvolume: P,
        description: Option<&str>,
    ) -> Result<Snapshot> {
        let mut metadata = self.default_metadata.clone();
        
        if let Some(desc) = description {
            metadata.insert("description".to_string(), desc.to_string());
        }
        
        Snapshot::create(
            subvolume,
            &self.snapshot_dir,
            self.read_only,
            None, // No parent for now
            Some(metadata),
        )
        .await
    }
    
    /// Create an incremental snapshot based on a parent
    pub async fn create_incremental_snapshot<P: AsRef<Path>>(
        &self,
        subvolume: P,
        parent: &Snapshot,
        description: Option<&str>,
    ) -> Result<Snapshot> {
        let mut metadata = self.default_metadata.clone();
        
        if let Some(desc) = description {
            metadata.insert("description".to_string(), desc.to_string());
        }
        
        Snapshot::create(
            subvolume,
            &self.snapshot_dir,
            self.read_only,
            Some(parent),
            Some(metadata),
        )
        .await
    }
    
    /// List all snapshots for a subvolume
    pub async fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        let mut snapshots = Vec::new();
        
        // List all subvolumes in the snapshot directory
        let subvols = btrfs::Subvolume::list_subvolumes(&self.snapshot_dir).await?;
        
        for subvol in subvols {
            // Try to load metadata from .snapinfo file if it exists
            let mut metadata_path = subvol.path.clone();
            metadata_path.push(".snapinfo");
            
            let metadata = if metadata_path.exists() {
                let metadata_str = fs::read_to_string(&metadata_path).await?;
                serde_json::from_str(&metadata_str).unwrap_or_default()
            } else {
                HashMap::new()
            };
            
            let snapshot = Snapshot {
                id: metadata
                    .get("id")
                    .cloned()
                    .unwrap_or_else(|| format!("snap_{}", uuid::Uuid::new_v4())),
                subvolume: metadata
                    .get("subvolume")
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string()),
                path: subvol.path,
                read_only: subvol.read_only,
                parent_id: metadata.get("parent_id").cloned(),
                created_at: subvol.created_at,
                size: subvol.size,
                metadata,
            };
            
            snapshots.push(snapshot);
        }
        
        Ok(snapshots)
    }
    
    /// Find a snapshot by ID
    pub async fn find_snapshot(&self, id: &str) -> Result<Option<Snapshot>> {
        let snapshots = self.list_snapshots().await?;
        Ok(snapshots.into_iter().find(|s| s.id == id))
    }
    
    /// Delete a snapshot by ID
    pub async fn delete_snapshot(&self, id: &str) -> Result<()> {
        if let Some(snapshot) = self.find_snapshot(id).await? {
            snapshot.delete().await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Snapshot not found: {}", id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_snapshot_management() {
        // Skip tests if not running as root or not on BTRFS
        if !nix::unistd::Uid::effective().is_root() {
            eprintln!("Skipping BTRFS snapshot tests - requires root privileges");
            return;
        }
        
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();
        
        // Create a test subvolume
        let subvol_path = base_path.join("test_subvol");
        btrfs::Subvolume::create(&subvol_path).unwrap();
        
        // Create a file in the subvolume
        let test_file = subvol_path.join("test.txt");
        tokio::fs::write(&test_file, "test content").await.unwrap();
        
        // Create a snapshot manager
        let snapshot_dir = base_path.join("snapshots");
        let manager = SnapshotManager::new(&snapshot_dir)
            .with_metadata("test", "true");
        
        // Create a snapshot
        let snapshot = manager.create_snapshot(&subvol_path, Some("test snapshot"))
            .await
            .unwrap();
            
        assert!(snapshot.path.exists());
        assert!(snapshot.read_only);
        assert_eq!(snapshot.metadata.get("description").unwrap(), "test snapshot");
        assert_eq!(snapshot.metadata.get("test").unwrap(), "true");
        
        // List snapshots
        let snapshots = manager.list_snapshots().await.unwrap();
        assert!(!snapshots.is_empty());
        
        // Create an incremental snapshot
        let incremental = manager.create_incremental_snapshot(&subvol_path, &snapshot, None)
            .await
            .unwrap();
            
        assert!(incremental.path.exists());
        assert_eq!(incremental.parent_id, Some(snapshot.id));
        
        // Cleanup
        manager.delete_snapshot(&snapshot.id).await.unwrap();
        manager.delete_snapshot(&incremental.id).await.unwrap();
        btrfs::Subvolume::delete(&subvol_path).unwrap();
    }
}
