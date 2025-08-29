//! Configuration for the backup system

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Storage provider configuration
    pub storage: StorageConfig,
    
    /// Encryption settings
    #[serde(default)]
    pub encryption: EncryptionConfig,
    
    /// Retention policy
    #[serde(default)]
    pub retention: RetentionPolicy,
    
    /// Performance settings
    #[serde(default)]
    pub performance: PerformanceSettings,
}

/// Storage provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StorageConfig {
    /// Local filesystem storage
    Local {
        /// Path to store backups
        path: PathBuf,
    },
    
    /// S3-compatible storage
    S3 {
        /// Bucket name
        bucket: String,
        
        /// Region
        region: String,
        
        /// Endpoint URL (for non-AWS S3)
        endpoint: Option<String>,
        
        /// Access key
        access_key_id: String,
        
        /// Secret access key
        secret_access_key: String,
    },
    
    // Add other storage providers as needed
}

/// Encryption configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable encryption
    pub enabled: bool,
    
    /// Path to encryption key
    pub key_path: Option<PathBuf>,
    
    /// Encryption algorithm
    pub algorithm: String,
}

/// Retention policy for backups
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Keep all backups for this many days
    pub keep_daily: Option<u32>,
    
    /// Keep weekly backups for this many weeks
    pub keep_weekly: Option<u32>,
    
    /// Keep monthly backups for this many months
    pub keep_monthly: Option<u32>,
    
    /// Keep yearly backups for this many years
    pub keep_yearly: Option<u32>,
}

/// Performance-related settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Maximum number of parallel uploads
    pub max_parallel_uploads: usize,
    
    /// Chunk size for uploads (in bytes)
    pub chunk_size: usize,
    
    /// Enable compression
    pub compression: bool,
    
    /// Compression level (1-22)
    pub compression_level: u32,
    
    /// Maximum upload bandwidth (bytes/second)
    pub max_bandwidth: Option<u64>,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            storage: StorageConfig::Local {
                path: "/var/lib/rast/backups".into(),
            },
            encryption: Default::default(),
            retention: Default::default(),
            performance: PerformanceSettings {
                max_parallel_uploads: 4,
                chunk_size: 8 * 1024 * 1024, // 8MB
                compression: true,
                compression_level: 3,
                max_bandwidth: None,
            },
        }
    }
}
