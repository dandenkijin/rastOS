//! BTRFS snapshot management for rastOS backups

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error type for BTRFS operations
#[derive(Error, Debug)]
pub enum BtrfsError {
    #[error("BTRFS command failed: {0}")]
    CommandFailed(String),
    
    #[error("Invalid subvolume path: {0}")]
    InvalidSubvolume(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Path is not a BTRFS subvolume: {0:?}")]
    NotASubvolume(PathBuf),
    
    #[error("Snapshot already exists: {0:?}")]
    SnapshotExists(PathBuf),
}

/// Represents a BTRFS subvolume or snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subvolume {
    /// Path to the subvolume
    pub path: PathBuf,
    
    /// Whether this is a read-only snapshot
    pub read_only: bool,
    
    /// Parent subvolume (for snapshots)
    pub parent: Option<PathBuf>,
    
    /// Creation time
    pub created_at: DateTime<Utc>,
    
    /// Size in bytes
    pub size: u64,
}

impl Subvolume {
    /// Create a new read-write subvolume
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        // Run btrfs subvolume create
        let output = Command::new("btrfs")
            .args(["subvolume", "create", path.as_os_str().to_str().unwrap()])
            .output()?;
            
        if !output.status.success() {
            return Err(BtrfsError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            )
            .into());
        }
        
        // Get subvolume info
        Self::from_path(path)
    }
    
    /// Create a read-only snapshot of an existing subvolume
    pub fn create_snapshot<P: AsRef<Path>>(
        source: P,
        dest: P,
        read_only: bool,
    ) -> Result<Self> {
        let source = source.as_ref();
        let dest = dest.as_ref();
        
        // Check if source is a subvolume
        if !Self::is_subvolume(source)? {
            return Err(BtrfsError::NotASubvolume(source.to_path_buf()).into());
        }
        
        // Check if destination exists
        if dest.exists() {
            return Err(BtrfsError::SnapshotExists(dest.to_path_buf()).into());
        }
        
        // Create parent directories if they don't exist
        if let Some(parent) = dest.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        // Build the btrfs subvolume snapshot command
        let mut cmd = Command::new("btrfs");
        cmd.arg("subvolume");
        
        if read_only {
            cmd.arg("snapshot");
        } else {
            cmd.arg("snapshot");
        }
        
        cmd.arg("-r");
        cmd.arg(source.as_os_str());
        cmd.arg(dest.as_os_str());
        
        // Execute the command
        let output = cmd.output()?;
        
        if !output.status.success() {
            return Err(BtrfsError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            )
            .into());
        }
        
        // Get the created snapshot info
        Self::from_path(dest)
    }
    
    /// Delete a subvolume or snapshot
    pub fn delete<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        
        // Run btrfs subvolume delete
        let output = Command::new("btrfs")
            .args(["subvolume", "delete", path.as_os_str().to_str().unwrap()])
            .output()?;
            
        if !output.status.success() {
            return Err(BtrfsError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            )
            .into());
        }
        
        Ok(())
    }
    
    /// List all subvolumes under a given path
    pub fn list_subvolumes<P: AsRef<Path>>(path: P) -> Result<Vec<Self>> {
        let path = path.as_ref();
        let output = Command::new("btrfs")
            .args(["subvolume", "list", "-p", path.to_str().unwrap()])
            .output()?;
            
        if !output.status.success() {
            return Err(BtrfsError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            )
            .into());
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut subvolumes = Vec::new();
        
        for line in output_str.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                // Format: ID gen top level path parent_uuid received_uuid uuid path
                if let (Some(id), Some(parent_id), Some(path_str)) = (parts[1].parse::<u64>().ok(), parts[3].parse::<u64>().ok(), parts.get(9)) {
                    if id != 5 { // Skip the root subvolume (ID 5)
                        let path = PathBuf::from(path_str.trim_start_matches("./"));
                        if let Ok(subvol) = Self::from_path(&path) {
                            subvolumes.push(subvol);
                        }
                    }
                }
            }
        }
        
        Ok(subvolumes)
    }
    
    /// Check if a path is a BTRFS subvolume
    pub fn is_subvolume<P: AsRef<Path>>(path: P) -> Result<bool> {
        let path = path.as_ref();
        let output = Command::new("btrfs")
            .args(["subvolume", "show", path.to_str().unwrap()])
            .output()?;
            
        Ok(output.status.success())
    }
    
    /// Get subvolume information from a path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        // Get subvolume info
        let output = Command::new("btrfs")
            .args(["subvolume", "show", path.to_str().unwrap()])
            .output()?;
            
        if !output.status.success() {
            return Err(BtrfsError::NotASubvolume(path.to_path_buf()).into());
        }
        
        // Parse the output to get subvolume properties
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut read_only = false;
        
        for line in output_str.lines() {
            if line.contains("Flags:") && line.contains("readonly") {
                read_only = true;
                break;
            }
        }
        
        // Get file metadata for size and creation time
        let metadata = std::fs::metadata(path)?;
        let created_at = metadata.created()?;
        let created_at: DateTime<Utc> = created_at.into();
        
        // Get size using du (more accurate for subvolumes)
        let du_output = Command::new("du")
            .args(["-bs", path.to_str().unwrap()])
            .output()?;
            
        let size = if du_output.status.success() {
            let output_str = String::from_utf8_lossy(&du_output.stdout);
            output_str
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0)
        } else {
            0
        };
        
        Ok(Self {
            path: path.to_path_buf(),
            read_only,
            parent: None, // Would need additional logic to determine parent
            created_at,
            size,
        })
    }
    
    /// Create a read-only snapshot of this subvolume
    pub fn snapshot<P: AsRef<Path>>(&self, dest: P) -> Result<Self> {
        Self::create_snapshot(&self.path, dest, true)
    }
    
    /// Send this subvolume to a file or stream
    pub fn send<P: AsRef<Path>>(&self, output: Option<P>) -> Result<()> {
        let mut cmd = Command::new("btrfs");
        cmd.arg("send");
        
        // Add parent if this is an incremental snapshot
        if let Some(parent) = &self.parent {
            cmd.arg("-p").arg(parent);
        }
        
        cmd.arg(&self.path);
        
        // Redirect output if specified
        if let Some(output_path) = output {
            use std::fs::File;
            use std::os::unix::io::FromRawFd;
            
            let file = File::create(output_path)?;
            let stdout = unsafe { std::process::Stdio::from_raw_fd(file.into_raw_fd()) };
            cmd.stdout(stdout);
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            return Err(BtrfsError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            )
            .into());
        }
        
        Ok(())
    }
    
    /// Receive a subvolume from a file or stream
    pub fn receive<P: AsRef<Path>>(input: P, dest: P) -> Result<Self> {
        let dest = dest.as_ref();
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = dest.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        let input_file = std::fs::File::open(input)?;
        let input_fd = unsafe { std::os::unix::io::AsRawFd::as_raw_fd(&input_file) };
        
        let output = Command::new("btrfs")
            .arg("receive")
            .arg(dest.parent().unwrap_or_else(|| Path::new("/")))
            .stdin(unsafe { std::process::Stdio::from_raw_fd(input_fd) })
            .output()?;
            
        if !output.status.success() {
            return Err(BtrfsError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            )
            .into());
        }
        
        Self::from_path(dest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_subvolume_operations() {
        // Skip tests if not running as root or not on BTRFS
        if !nix::unistd::Uid::effective().is_root() {
            eprintln!("Skipping BTRFS tests - requires root privileges");
            return;
        }
        
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();
        
        // Test subvolume creation
        let subvol_path = base_path.join("test_subvol");
        let subvol = Subvolume::create(&subvol_path).unwrap();
        assert!(subvol_path.exists());
        assert!(!subvol.read_only);
        
        // Test snapshot creation
        let snapshot_path = base_path.join("test_snapshot");
        let snapshot = Subvolume::create_snapshot(&subvol_path, &snapshot_path, true).unwrap();
        assert!(snapshot_path.exists());
        assert!(snapshot.read_only);
        
        // Test listing subvolumes
        let subvolumes = Subvolume::list_subvolumes(base_path).unwrap();
        assert!(subvolumes.len() >= 2); // At least our two test volumes
        
        // Test sending/receiving
        let send_file = base_path.join("snapshot.btrfs");
        snapshot.send(Some(&send_file)).unwrap();
        assert!(send_file.exists());
        
        let restore_path = base_path.join("restored_snapshot");
        Subvolume::receive(&send_file, &restore_path).unwrap();
        assert!(restore_path.exists());
        
        // Cleanup
        Subvolume::delete(&subvol_path).unwrap();
        Subvolume::delete(&snapshot_path).unwrap();
        Subvolume::delete(&restore_path).unwrap();
    }
}
