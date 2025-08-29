//! Storage backends for backups

use async_trait::async_trait;
use bytes::Bytes;
use std::path::Path;
use object_store::path::Path as ObjectPath;

use crate::backup::{BackupError, Result};

mod local;
mod s3;

/// Trait for storage backends
#[async_trait]
pub trait StorageBackend: Send + Sync + std::fmt::Debug {
    /// Upload data to the storage backend
    async fn put(&self, path: &Path, data: Bytes) -> Result<()>;
    
    /// Download data from the storage backend
    async fn get(&self, path: &Path) -> Result<Bytes>;
    
    /// List objects with the given prefix
    async fn list(&self, prefix: Option<&Path>) -> Result<Vec<ObjectPath>>;
    
    /// Delete an object
    async fn delete(&self, path: &Path) -> Result<()>;
    
    /// Check if an object exists
    async fn exists(&self, path: &Path) -> bool;
}

/// Create a storage backend from the given configuration
pub async fn create_backend(config: &super::config::BackupConfig) -> Result<Box<dyn StorageBackend>> {
    match &config.storage {
        super::config::StorageConfig::Local { path } => {
            Ok(Box::new(local::LocalStorage::new(path).await?))
        }
        super::config::StorageConfig::S3 { 
            bucket, 
            region, 
            endpoint, 
            access_key_id, 
            secret_access_key 
        } => {
            Ok(Box::new(s3::S3Storage::new(
                bucket,
                region,
                endpoint.as_deref(),
                access_key_id,
                secret_access_key,
            ).await?))
        }
    }
}

// Re-export implementations
pub use local::LocalStorage;
pub use s3::S3Storage;
