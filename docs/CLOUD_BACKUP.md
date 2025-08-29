# Cloud Storage Backup Integration

## Overview
This document outlines the implementation of BTRFS snapshot-based cloud storage backups in rastOS using the `object_store` crate. This provides a robust, efficient way to back up system snapshots to various cloud storage providers with a unified interface.

## Supported Backends
- Amazon S3 and S3-compatible storage (MinIO, Ceph, etc.)
- Google Cloud Storage
- Microsoft Azure Blob Storage
- Local filesystem (for testing and local backups)

## BTRFS Integration

### Snapshot Backup Process
1. Create a read-only BTRFS snapshot
2. Stream the snapshot directly to object storage using `btrfs send`
3. Store metadata including:
   - Snapshot tree structure
   - File permissions and attributes
   - Checksums for verification

### Incremental Backups
- Use `btrfs send -p <parent>` for incremental snapshots
- Store parent-child relationships in metadata
- Automatically determine minimal required snapshots for restoration

### Example Backup Flow
```rust
// 1. Create a read-only snapshot
let snapshot = btrfs::create_snapshot("@home", "@snapshots/home_backup_$(date +%s)", true)?;

// 2. Initialize object store
let store = ObjectStoreBuilder::from_config(&config).build()?;

// 3. Stream snapshot to object storage
let backup_id = format!("backups/{}/home", Utc::now().format("%Y%m%d_%H%M%S"));
let mut stream = BtrfsSendStream::new(snapshot.path())?;
store.put(&backup_id, stream).await?;

// 4. Store metadata
let metadata = BackupMetadata {
    snapshot_name: snapshot.name(),
    timestamp: Utc::now(),
    parent_snapshot: last_backup_id,
    size: snapshot.size(),
    checksum: snapshot.checksum(),
};
store.put_metadata(&format!("{}/metadata.json", backup_id), &metadata).await?;
```

## Architecture

### Core Components

#### 1. Backup Manager
- Handles backup/restore operations
- Manages encryption/decryption
- Implements retention policies
- Handles scheduling

#### 2. Storage Backend
- Abstracted through `object_store`
- Provider-specific configuration
- Connection pooling
- Retry logic

#### 3. Configuration
```rust
[backup]
# Required
provider = "s3"  # or "gcs", "azure", "local"
bucket = "rastos-backups"

# Authentication (provider-specific)
access_key_id = "..."
secret_access_key = "..."

# Optional
region = "us-east-1"
endpoint = "https://s3.amazonaws.com"
encryption_key = "path/to/key"

[backup.retention]
keep_last = 7
keep_daily = 30
keep_weekly = 12
keep_monthly = 12

[backup.schedule]
enabled = true
interval = "1d"
time = "02:00"
```

## Implementation Plan

### 1. Dependencies
```toml
[dependencies]
object_store = { version = "0.8", features = ["aws", "gcp", "azure"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
chrono = "0.4"
thiserror = "1.0"
tracing = "0.1"
```

### 2. Core Traits
```rust
#[async_trait]
pub trait BackupEngine {
    async fn create_backup(&self, snapshot: &str) -> Result<BackupId>;
    async fn restore_backup(&self, backup_id: &BackupId) -> Result<()>;
    async fn list_backups(&self) -> Result<Vec<BackupInfo>>;
    async fn delete_backup(&self, backup_id: &BackupId) -> Result<()>;
    async fn verify_backup(&self, backup_id: &BackupId) -> Result<()>;
}
```

### 2. Snapshot-Specific Commands

#### Backup Commands
```bash
# Create and upload a new snapshot backup
rast backup create @home --to s3://my-bucket/backups

# Create incremental backup based on last snapshot
rast backup create @home --incremental --to s3://my-bucket/backups

# List available backups
rast backup list @home --from s3://my-bucket/backups

# Restore a snapshot backup
rast backup restore s3://my-bucket/backups/20230829_120000 @home_restored

# Verify backup integrity
rast backup verify s3://my-bucket/backups/20230829_120000
```

#### Snapshot Management
```bash
# Create and upload a snapshot
rast snapshot create @home --backup s3://my-bucket/backups

# List snapshots with backup status
rast snapshot list --show-backups

# Restore from a backed-up snapshot
rast snapshot restore @home s3://my-bucket/backups/20230829_120000
```

### 3. CLI Commands
```bash
# Create backup
rast backup create [SNAPSHOT]

# List backups
rast backup list

# Restore from backup
rast backup restore <BACKUP_ID>

# Configure backup settings
rast backup config [--provider=PROVIDER] [--bucket=BUCKET] [--key=KEY] [--secret=SECRET]

# Schedule automatic backups
rast backup schedule [--enable|--disable] [--interval=INTERVAL] [--time=TIME]
```

## Security Considerations

### Data Encryption
- Client-side encryption using AES-256-GCM
- Key management using system keyring
- Support for KMS integration

### Authentication
- Environment variables for sensitive data
- IAM roles for cloud providers
- Credential rotation

## Performance Optimizations

### BTRFS-Specific Optimizations
- **Incremental Backups**: Only transfer changed blocks using `btrfs send -p`
- **Streaming**: Directly pipe `btrfs send` output to object storage
- **Parallel Uploads**: Split large snapshots into chunks for parallel upload
- **Compression**: On-the-fly compression using zstd
- **Delta Encoding**: Efficient storage of only changed blocks
- **Bandwidth Throttling**: Configurable rate limiting

### Example Configuration
```toml
[backup]
provider = "s3"
bucket = "rastos-backups"

[btrfs]
# Compression level (1-22, 0 = none)
compression_level = 3

# Split snapshots larger than this (in MB)
chunk_size = 1024

# Maximum upload bandwidth in KB/s (0 = unlimited)
max_bandwidth = 10240

# Number of parallel uploads
parallel_uploads = 4

[retention]
# Keep daily backups for 30 days
keep_daily = 30

# Keep weekly backups for 12 weeks
keep_weekly = 12

# Keep monthly backups for 12 months
keep_monthly = 12
```

## Monitoring and Logging
```rust
#[derive(Debug)]
pub struct BackupEvent {
    pub timestamp: DateTime<Utc>,
    pub operation: String,
    pub status: BackupStatus,
    pub size_bytes: Option<u64>,
    pub duration: Option<Duration>,
    pub error: Option<String>,
}
```

## Testing Strategy
- Unit tests for core functionality
- Integration tests with localstack for S3
- End-to-end tests with actual cloud providers
- Fuzz testing for edge cases

## Future Extensions
- Incremental backups
- Cross-region replication
- Backup verification and validation
- Web UI for management
- Support for additional storage backends

## References
- [object_store crate](https://docs.rs/object_store/latest/object_store/)
- [AWS S3 Best Practices](https://docs.aws.amazon.com/AmazonS3/latest/userguide/best-practices.html)
- [Google Cloud Storage Documentation](https://cloud.google.com/storage/docs)
- [Azure Blob Storage Documentation](https://docs.microsoft.com/en-us/azure/storage/blobs/)
