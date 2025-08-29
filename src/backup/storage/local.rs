//! Local filesystem storage backend

use super::*;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Local filesystem storage backend
#[derive(Debug)]
pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    /// Create a new local storage backend
    pub async fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        
        // Create base directory if it doesn't exist
        if !base_path.exists() {
            fs::create_dir_all(&base_path).await?;
        }
        
        Ok(Self { base_path })
    }
    
    fn resolve_path(&self, path: &Path) -> PathBuf {
        // Prevent directory traversal
        let path = path.components()
            .filter(|c| !matches!(c, std::path::Component::ParentDir))
            .collect::<PathBuf>();
            
        self.base_path.join(path)
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    async fn put(&self, path: &Path, data: bytes::Bytes) -> Result<()> {
        let full_path = self.resolve_path(path);
        
        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await?;
            }
        }
        
        let mut file = fs::File::create(&full_path).await?;
        file.write_all(&data).await?;
        
        Ok(())
    }
    
    async fn get(&self, path: &Path) -> Result<bytes::Bytes> {
        let full_path = self.resolve_path(path);
        let data = fs::read(&full_path).await?;
        Ok(bytes::Bytes::from(data))
    }
    
    async fn list(&self, prefix: Option<&Path>) -> Result<Vec<object_store::path::Path>> {
        let base = if let Some(prefix) = prefix {
            self.resolve_path(prefix)
        } else {
            self.base_path.clone()
        };
        
        let mut paths = Vec::new();
        
        let mut read_dir = fs::read_dir(base).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            if entry.file_type().await?.is_file() {
                if let Some(rel_path) = entry.path().strip_prefix(&self.base_path).ok() {
                    if let Some(path_str) = rel_path.to_str() {
                        paths.push(object_store::path::Path::from(path_str));
                    }
                }
            }
        }
        
        Ok(paths)
    }
    
    async fn delete(&self, path: &Path) -> Result<()> {
        let full_path = self.resolve_path(path);
        if full_path.exists() {
            fs::remove_file(full_path).await?;
        }
        Ok(())
    }
    
    async fn exists(&self, path: &Path) -> bool {
        self.resolve_path(path).exists()
    }
}
