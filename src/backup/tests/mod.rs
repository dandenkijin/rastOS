//! Test module for the backup system

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tempfile::tempdir;
use tokio::fs;

use crate::backup::{
    config::{BackupConfig, StorageConfig},
    storage::{LocalStorageConfig, StorageBackend, StorageBackendFactory},
    BackupManager, Snapshot, SnapshotManager,
};

// Mock storage backend for testing
#[derive(Debug, Clone)]
struct MockStorageBackend {
    files: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
}

impl MockStorageBackend {
    fn new() -> Self {
        Self {
            files: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait]
impl StorageBackend for MockStorageBackend {
    async fn upload_file(&self, source: &std::path::Path, dest: &str) -> Result<()> {
        let content = tokio::fs::read(source).await?;
        self.files
            .lock()
            .unwrap()
            .insert(dest.to_string(), content);
        Ok(())
    }

    async fn download_file(&self, source: &str, dest: &std::path::Path) -> Result<()> {
        if let Some(content) = self.files.lock().unwrap().get(source) {
            tokio::fs::write(dest, content).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("File not found"))
        }
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        Ok(self
            .files
            .lock()
            .unwrap()
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }

    async fn read_to_string(&self, path: &str) -> Result<String> {
        self.files
            .lock()
            .unwrap()
            .get(path)
            .map(|v| String::from_utf8_lossy(v).into_owned())
            .ok_or_else(|| anyhow::anyhow!("File not found"))
    }

    async fn write(&self, path: &str, content: Vec<u8>) -> Result<()> {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), content);
        Ok(())
    }

    async fn delete(&self, path: &str) -> Result<()> {
        self.files.lock().unwrap().remove(path);
        Ok(())
    }
}

// Test utilities
struct TestEnvironment {
    _temp_dir: tempfile::TempDir,
    config: BackupConfig,
    snapshot_manager: SnapshotManager,
    storage: Arc<dyn StorageBackend>,
}

impl TestEnvironment {
    async fn new() -> Result<Self> {
        let temp_dir = tempdir()?;
        let storage = Arc::new(MockStorageBackend::new());
        
        let config = BackupConfig {
            storage: StorageConfig {
                local: Some(LocalStorageConfig {
                    path: temp_dir.path().join("backups"),
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        
        let snapshot_dir = temp_dir.path().join("snapshots");
        fs::create_dir_all(&snapshot_dir).await?;
        let snapshot_manager = SnapshotManager::new(snapshot_dir);
        
        Ok(Self {
            _temp_dir: temp_dir,
            config,
            snapshot_manager,
            storage,
        })
    }
    
    async fn create_backup_manager(&self) -> Result<BackupManager> {
        BackupManager::new(self.config.clone()).await
    }
}

#[tokio::test]
async fn test_backup_creation() -> Result<()> {
    let env = TestEnvironment::new().await?;
    let backup_manager = env.create_backup_manager().await?;
    
    // Create a test subvolume
    let subvol_path = env._temp_dir.path().join("test_subvol");
    fs::create_dir_all(&subvol_path).await?;
    
    // Create a test file in the subvolume
    let test_file = subvol_path.join("test.txt");
    fs::write(&test_file, "test content").await?;
    
    // Create a backup
    let backup = backup_manager
        .create_backup(&subvol_path, Some("test_backup"), Some("test description"), false, None)
        .await?;
    
    // Verify backup metadata
    assert_eq!(backup.name, "test_backup");
    assert_eq!(backup.description, Some("test description".to_string()));
    assert!(!backup.is_incremental);
    assert!(backup.size > 0);
    
    // Verify the backup exists in storage
    let backup_path = format!("backups/{}/{}.btrfs", &backup.id[..2], backup.id);
    assert!(backup_manager.storage().list("backups/").await?.contains(&backup_path));
    
    Ok(())
}

#[tokio::test]
async fn test_backup_restore() -> Result<()> {
    let env = TestEnvironment::new().await?;
    let backup_manager = env.create_backup_manager().await?;
    
    // Create a test subvolume with content
    let subvol_path = env._temp_dir.path().join("test_subvol");
    fs::create_dir_all(&subvol_path).await?;
    fs::write(subvol_path.join("test.txt"), "test content").await?;
    
    // Create a backup
    let backup = backup_manager
        .create_backup(&subvol_path, None, None, false, None)
        .await?;
    
    // Restore to a new location
    let restore_path = env._temp_dir.path().join("restored");
    backup_manager.restore_backup(&backup.id, Some(&restore_path)).await?;
    
    // Verify the restored content
    let restored_content = fs::read_to_string(restore_path.join("test.txt")).await?;
    assert_eq!(restored_content, "test content");
    
    Ok(())
}

#[tokio::test]
async fn test_incremental_backup() -> Result<()> {
    let env = TestEnvironment::new().await?;
    let backup_manager = env.create_backup_manager().await?;
    
    // Create initial subvolume and backup
    let subvol_path = env._temp_dir.path().join("test_subvol");
    fs::create_dir_all(&subvol_path).await?;
    fs::write(subvol_path.join("file1.txt"), "initial content").await?;
    
    let full_backup = backup_manager
        .create_backup(&subvol_path, None, None, false, None)
        .await?;
    
    // Modify the subvolume
    fs::write(subvol_path.join("file2.txt"), "new file").await?;
    
    // Create incremental backup
    let incremental = backup_manager
        .create_backup(&subvol_path, None, None, true, Some(&full_backup))
        .await?;
    
    assert!(incremental.is_incremental);
    assert_eq!(incremental.parent_id, Some(full_backup.id));
    
    // Verify both backups exist
    let backups = backup_manager.list_backups().await?;
    assert_eq!(backups.len(), 2);
    
    Ok(())
}

#[tokio::test]
async fn test_backup_deletion() -> Result<()> {
    let env = TestEnvironment::new().await?;
    let backup_manager = env.create_backup_manager().await?;
    
    // Create a test backup
    let subvol_path = env._temp_dir.path().join("test_subvol");
    fs::create_dir_all(&subvol_path).await?;
    
    let backup = backup_manager
        .create_backup(&subvol_path, None, None, false, None)
        .await?;
    
    // Verify the backup exists
    assert!(backup_manager.get_backup(&backup.id).await.is_ok());
    
    // Delete the backup
    backup_manager.delete_backup(&backup.id).await?;
    
    // Verify the backup is gone
    assert!(backup_manager.get_backup(&backup.id).await.is_err());
    
    Ok(())
}
