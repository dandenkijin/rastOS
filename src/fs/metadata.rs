//! File metadata operations for rastOS

use std::fs::{self, Metadata as StdMetadata};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::SystemTime;

use super::{FsError, Result};

/// Extended file metadata information
#[derive(Debug, Clone)]
pub struct Metadata {
    path: String,
    inner: StdMetadata,
}

impl Metadata {
    /// Create a new Metadata instance from std::fs::Metadata
    pub fn from_std<P: AsRef<Path>>(path: P, metadata: StdMetadata) -> Self {
        Self {
            path: path.as_ref().to_string_lossy().to_string(),
            inner: metadata,
        }
    }

    /// Get the file size in bytes
    pub fn len(&self) -> u64 {
        self.inner.len()
    }

    /// Check if the file is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if the path is a directory
    pub fn is_dir(&self) -> bool {
        self.inner.is_dir()
    }

    /// Check if the path is a file
    pub fn is_file(&self) -> bool {
        self.inner.is_file()
    }

    /// Check if the path is a symbolic link
    pub fn is_symlink(&self) -> bool {
        self.inner.file_type().is_symlink()
    }

    /// Get the file's permissions
    pub fn permissions(&self) -> std::fs::Permissions {
        self.inner.permissions()
    }

    /// Get the file's modification time
    pub fn modified(&self) -> Result<SystemTime> {
        self.inner.modified().map_err(Into::into)
    }

    /// Get the file's last access time
    pub fn accessed(&self) -> Result<SystemTime> {
        self.inner.accessed().map_err(Into::into)
    }

    /// Get the file's creation time
    pub fn created(&self) -> Result<SystemTime> {
        self.inner.created().map_err(Into::into)
    }

    /// Get the file's inode number (Unix only)
    pub fn ino(&self) -> u64 {
        self.inner.ino()
    }

    /// Get the number of hard links to this file
    pub fn nlink(&self) -> u64 {
        self.inner.nlink()
    }

    /// Get the user ID of the file's owner (Unix only)
    pub fn uid(&self) -> u32 {
        self.inner.uid()
    }

    /// Get the group ID of the file (Unix only)
    pub fn gid(&self) -> u32 {
        self.inner.gid()
    }

    /// Get the file's mode (permissions + file type) (Unix only)
    pub fn mode(&self) -> u32 {
        self.inner.mode()
    }
}

/// Get metadata for a file or directory
pub fn metadata<P: AsRef<Path>>(path: P) -> Result<Metadata> {
    let path = path.as_ref();
    let metadata = fs::metadata(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => FsError::not_found(path),
        std::io::ErrorKind::PermissionDenied => FsError::permission_denied(path),
        _ => e.into(),
    })?;
    
    Ok(Metadata::from_std(path, metadata))
}

/// Check if a path exists
pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// Check if a path is a file
pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

/// Check if a path is a directory
pub fn is_dir<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_metadata() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test.txt");
        File::create(&file_path)?;

        let meta = metadata(&file_path)?;
        assert!(meta.is_file());
        assert!(!meta.is_dir());
        assert!(!meta.is_symlink());
        assert_eq!(meta.len(), 0);
        assert!(meta.is_empty());

        // Test file times
        assert!(meta.created().is_ok());
        assert!(meta.accessed().is_ok());
        assert!(meta.modified().is_ok());

        // Test Unix-specific metadata
        assert_ne!(meta.ino(), 0);
        assert_eq!(meta.nlink(), 1);
        assert_ne!(meta.uid(), 0); // Should have some user ID
        assert_ne!(meta.gid(), 0); // Should have some group ID
        assert_ne!(meta.mode(), 0); // Should have some permissions

        Ok(())
    }

    #[test]
    fn test_metadata_nonexistent() {
        let result = metadata("/nonexistent/path");
        assert!(matches!(result, Err(FsError::NotFound(_))));
    }
}
