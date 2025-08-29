//! rastOS Backup System
//! 
//! Provides a unified interface for backing up BTRFS snapshots to various cloud storage providers.

#![deny(missing_docs)]
#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]

use std::path::PathBuf;
use thiserror::Error;

//! Backup management for rastOS

pub mod btrfs;
pub mod cli;
pub mod config;
pub mod encryption;
pub mod providers;
pub mod snapshot;
pub mod storage;
pub mod tests;

/// Result type for backup operations
pub type Result<T> = std::result::Result<T, BackupError>;

/// Error type for backup operations
#[derive(Error, Debug)]
pub enum BackupError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Storage provider error
    #[error("Storage error: {0}")]
    Storage(#[from] object_store::Error),
    
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Encryption error
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    /// Snapshot error
    #[error("Snapshot error: {0}")]
    Snapshot(String),
    
    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

/// Represents a backup in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    /// Unique identifier for the backup
    pub id: String,
    
    /// Name of the backup
    pub name: String,
    
    /// Description of the backup
    pub description: Option<String>,
    
    /// Path to the subvolume being backed up
    pub subvolume_path: PathBuf,
    
    /// Path to the snapshot used for this backup
    pub snapshot_path: Option<PathBuf>,
    
    /// Size of the backup in bytes
    pub size: u64,
    
    /// When the backup was created
    pub created_at: chrono::DateTime<Utc>,
    
    /// When the backup was last modified
    pub updated_at: chrono::DateTime<Utc>,
    
    /// Metadata for the backup
    pub metadata: std::collections::HashMap<String, String>,
    
    /// Whether this is an incremental backup
    pub is_incremental: bool,
    
    /// ID of the parent backup (for incremental backups)
    pub parent_id: Option<String>,
    
    /// IDs of child backups (for incremental backups)
    pub child_ids: Vec<String>,
}

/// Manages backup operations
pub struct BackupManager {
    /// Backup configuration
    config: config::BackupConfig,
    
    /// Storage backend for backups
    storage: Box<dyn storage::StorageBackend>,
    
    /// Snapshot manager for BTRFS snapshots
    snapshot_manager: snapshot::SnapshotManager,
    
    /// Directory for temporary files
    temp_dir: PathBuf,
}

impl BackupManager {
    /// Create a new BackupManager with the given configuration
    pub async fn new(config: config::BackupConfig) -> Result<Self> {
        // Create storage backend
        let storage = storage::StorageBackendFactory::create(&config.storage).await?;
        
        // Create snapshot manager
        let snapshot_dir = config
            .storage
            .local
            .as_ref()
            .map(|c| c.path.clone())
            .unwrap_or_else(|| "/var/lib/rast/backups/snapshots".into());
            
        let snapshot_manager = snapshot::SnapshotManager::new(snapshot_dir);
        
        // Create temp directory
        let temp_dir = std::env::temp_dir()
            .join("rast-backup")
            .join(Uuid::new_v4().to_string());
            
        tokio::fs::create_dir_all(&temp_dir).await?;
        
        Ok(Self {
            config,
            storage,
            snapshot_manager,
            temp_dir,
        })
    }
    
    /// Get the storage backend
    pub fn storage(&self) -> &dyn storage::StorageBackend {
        self.storage.as_ref()
    }
    
    /// Get the snapshot manager
    pub fn snapshot_manager(&self) -> &snapshot::SnapshotManager {
        &self.snapshot_manager
    }

    /// Create a new backup of a subvolume
    pub async fn create_backup<P: AsRef<Path>>(
        &self,
        subvolume: P,
        name: Option<&str>,
        description: Option<&str>,
        incremental: bool,
        parent_backup: Option<&Backup>,
    ) -> Result<Backup> {
        let subvolume = subvolume.as_ref();
        
        // Create a snapshot first
        let snapshot = if let Some(parent) = parent_backup {
            // For incremental backups, we need the parent snapshot
            let parent_snapshot = self
                .snapshot_manager
                .find_snapshot(&parent.id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Parent snapshot not found"))?;
                
            self.snapshot_manager
                .create_incremental_snapshot(subvolume, &parent_snapshot, description)
                .await?
        } else {
            // Full backup
            self.snapshot_manager
                .create_snapshot(subvolume, description)
                .await?
        };
        
        // Create a temporary file for the backup
        let backup_file = self.temp_dir.join(format!("{}.btrfs", Uuid::new_v4()));
        
        // Send the snapshot to a file
        snapshot.send(&backup_file).await?;
        
        // Upload the backup file to storage
        let backup_id = Uuid::new_v4().to_string();
        let backup_path = format!("backups/{}/{}.btrfs", &backup_id[..2], &backup_id);
        
        self.storage
            .upload_file(&backup_file, &backup_path)
            .await?;
        
        // Get file size
        let size = tokio::fs::metadata(&backup_file).await?.len();
        
        // Create backup metadata
        let now = Utc::now();
        let backup = Backup {
            id: backup_id,
            name: name.unwrap_or_else(|| "Unnamed Backup").to_string(),
            description: description.map(|s| s.to_string()),
            subvolume_path: subvolume.to_path_buf(),
            snapshot_path: Some(snapshot.path.clone()),
            size,
            created_at: now,
            updated_at: now,
            metadata: snapshot.metadata,
            is_incremental: incremental,
            parent_id: parent_backup.map(|b| b.id.clone()),
            child_ids: Vec::new(),
        };
        
        // Save backup metadata
        self.save_backup_metadata(&backup).await?;
        
        // Clean up temporary files
        tokio::fs::remove_file(backup_file).await.ok();
        
        Ok(backup)
    }
    
    /// Restore a backup to a target path
    pub async fn restore_backup<P: AsRef<Path>>(
        &self,
        backup_id: &str,
        target: Option<P>,
    ) -> Result<()> {
        // Get backup metadata
        let backup = self.get_backup(backup_id).await?;
        
        // Determine target path
        let target_path = match target {
            Some(path) => path.as_ref().to_path_buf(),
            None => backup.subvolume_path.clone(),
        };
        
        // Download the backup file
        let backup_path = format!("backups/{}/{}.btrfs", &backup_id[..2], backup_id);
        let temp_file = self.temp_dir.join(format!("restore-{}.btrfs", backup_id));
        
        self.storage
            .download_file(&backup_path, &temp_file)
            .await?;
        
        // Restore the snapshot
        btrfs::Subvolume::receive(&temp_file, &target_path).await?;
        
        // Clean up
        tokio::fs::remove_file(temp_file).await.ok();
        
        Ok(())
    }
    
    /// List all backups
    pub async fn list_backups(&self) -> Result<Vec<Backup>> {
        // List all metadata files in the backup storage
        let mut backups = Vec::new();
        
        for entry in self.storage.list("backups/").await? {
            if entry.ends_with("/metadata.json") {
                if let Ok(metadata) = self.storage.read_to_string(&entry).await {
                    if let Ok(backup) = serde_json::from_str::<Backup>(&metadata) {
                        backups.push(backup);
                    }
                }
            }
        }
        
        // Sort by creation date (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(backups)
    }
    
    /// Get a specific backup by ID
    pub async fn get_backup(&self, backup_id: &str) -> Result<Backup> {
        let metadata_path = format!("backups/{}/{}/metadata.json", &backup_id[..2], backup_id);
        let metadata = self.storage.read_to_string(&metadata_path).await?;
        serde_json::from_str(&metadata).map_err(Into::into)
    }
    
    /// Verify a backup's integrity
    pub async fn verify_backup(&self, backup_id: &str) -> Result<bool> {
        // For now, just check if the backup exists and has valid metadata
        match self.get_backup(backup_id).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Delete a backup
    pub async fn delete_backup(&self, backup_id: &str) -> Result<()> {
        // Get backup metadata first
        let backup = self.get_backup(backup_id).await?;
        
        // Delete the backup file
        let backup_path = format!("backups/{}/{}.btrfs", &backup_id[..2], backup_id);
        self.storage.delete(&backup_path).await?;
        
        // Delete the metadata
        let metadata_path = format!("backups/{}/{}/metadata.json", &backup_id[..2], backup_id);
        self.storage.delete(&metadata_path).await?;
        
        // Delete the snapshot if it exists
        if let Some(snapshot_path) = backup.snapshot_path {
            if let Ok(snapshot) = btrfs::Subvolume::from_path(&snapshot_path).await {
                snapshot.delete().await.ok();
            }
        }
        
        Ok(())
    }
    
    /// Save backup metadata to storage
    async fn save_backup_metadata(&self, backup: &Backup) -> Result<()> {
        let metadata = serde_json::to_string_pretty(backup)?;
        let metadata_path = format!(
            "backups/{}/{}/metadata.json",
            &backup.id[..2],
            backup.id
        );
        
        self.storage
            .write(&metadata_path, metadata.into_bytes())
            .await?;
            
        Ok(())
    }
}

#[async_trait]
pub trait BackupOperation {
    async fn execute(&self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_backup_manager() {
        // TODO: Add comprehensive tests
    }
}
