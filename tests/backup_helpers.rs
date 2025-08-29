//! Test helpers for backup system

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tempfile::tempdir;
use tokio::fs;

use rastos::backup::{
    config::{BackupConfig, StorageConfig},
    storage::{LocalStorageConfig, StorageBackend},
    BackupManager, SnapshotManager,
};

/// A test environment for backup tests
pub struct TestEnvironment {
    /// Temporary directory that will be cleaned up when dropped
    _temp_dir: tempfile::TempDir,
    /// Backup configuration
    pub config: BackupConfig,
    /// Snapshot manager for the test
    pub snapshot_manager: SnapshotManager,
    /// Storage backend for the test
    pub storage: Arc<dyn StorageBackend>,
}

impl TestEnvironment {
    /// Create a new test environment with a temporary directory
    pub async fn new() -> Result<Self> {
        let temp_dir = tempdir()?;
        
        // Create a test configuration
        let config = BackupConfig {
            storage: StorageConfig {
                local: Some(LocalStorageConfig {
                    path: temp_dir.path().join("backups"),
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        
        // Set up snapshot directory
        let snapshot_dir = temp_dir.path().join("snapshots");
        fs::create_dir_all(&snapshot_dir).await?;
        let snapshot_manager = SnapshotManager::new(snapshot_dir);
        
        // Set up storage
        let storage = Arc::new(LocalStorage::new(temp_dir.path().join("storage")));
        
        Ok(Self {
            _temp_dir: temp_dir,
            config,
            snapshot_manager,
            storage,
        })
    }
    
    /// Create a backup manager for testing
    pub async fn create_backup_manager(&self) -> Result<BackupManager> {
        BackupManager::new(self.config.clone()).await
    }
    
    /// Create a test subvolume with some files
    pub async fn create_test_subvolume(&self, name: &str) -> Result<PathBuf> {
        let subvol_path = self._temp_dir.path().join(name);
        fs::create_dir_all(&subvol_path).await?;
        
        // Add some test files
        fs::write(subvol_path.join("test.txt"), "test content").await?;
        fs::create_dir(subvol_path.join("subdir")).await?;
        fs::write(subvol_path.join("subdir/file.txt"), "nested content").await?;
        
        Ok(subvol_path)
    }
}

/// A simple in-memory storage backend for testing
#[derive(Clone)]
struct LocalStorage {
    base_path: std::path::PathBuf,
}

impl LocalStorage {
    fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }
    
    fn full_path(&self, path: &str) -> std::path::PathBuf {
        self.base_path.join(path)
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    async fn upload_file(&self, source: &std::path::Path, dest: &str) -> Result<()> {
        let dest_path = self.full_path(dest);
        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::copy(source, dest_path).await?;
        Ok(())
    }

    async fn download_file(&self, source: &str, dest: &std::path::Path) -> Result<()> {
        let source_path = self.full_path(source);
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::copy(source_path, dest).await?;
        Ok(())
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let prefix_path = self.full_path(prefix);
        let mut files = Vec::new();
        
        if let Ok(mut entries) = tokio::fs::read_dir(prefix_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(rel_path) = entry.path().strip_prefix(&self.base_path) {
                    if let Some(path_str) = rel_path.to_str() {
                        files.push(path_str.to_string());
                    }
                }
            }
        }
        
        Ok(files)
    }

    async fn read_to_string(&self, path: &str) -> Result<String> {
        let path = self.full_path(path);
        tokio::fs::read_to_string(path).await.map_err(Into::into)
    }

    async fn write(&self, path: &str, content: Vec<u8>) -> Result<()> {
        let path = self.full_path(path);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let path = self.full_path(path);
        if path.is_dir() {
            tokio::fs::remove_dir_all(path).await?;
        } else if path.exists() {
            tokio::fs::remove_file(path).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[tokio::test]
    async fn test_test_environment() -> Result<()> {
        let env = TestEnvironment::new().await?;
        let subvol_path = env.create_test_subvolume("test_subvol").await?;
        
        // Verify test files were created
        assert!(subvol_path.join("test.txt").exists());
        assert!(subvol_path.join("subdir/file.txt").exists());
        
        // Test backup manager creation
        let _manager = env.create_backup_manager().await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_local_storage() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let storage = LocalStorage::new(temp_dir.path());
        
        // Test write and read
        let test_data = b"test data".to_vec();
        storage.write("test/file.txt", test_data.clone()).await?;
        
        let read_data = tokio::fs::read(temp_dir.path().join("test/file.txt")).await?;
        assert_eq!(read_data, test_data);
        
        // Test list
        let files = storage.list("test").await?;
        assert!(files.iter().any(|f| f == "test/file.txt"));
        
        // Test delete
        storage.delete("test/file.txt").await?;
        assert!(!temp_dir.path().join("test/file.txt").exists());
        
        Ok(())
    }
}
