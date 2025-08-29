//! Utility functions for file system operations
//!
//! This module provides additional utilities for working with the file system,
//! including temporary files/directories and glob pattern matching.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use glob::{glob_with, MatchOptions};
use tempfile::{NamedTempFile, TempDir};

use crate::fs::{FsError, Result};

/// Options for glob pattern matching
#[derive(Debug, Clone)]
pub struct GlobOptions {
    /// Whether to match case-insensitively
    pub case_insensitive: bool,
    /// Whether to match hidden files
    pub match_hidden: bool,
    /// Whether to require a literal separator
    pub require_literal_separator: bool,
}

impl Default for GlobOptions {
    fn default() -> Self {
        Self {
            case_insensitive: false,
            match_hidden: false,
            require_literal_separator: false,
        }
    }
}

/// Find files matching a glob pattern
///
/// # Examples
///
/// ```no_run
/// use rastos::fs::utils::glob;
///
/// // Find all .txt files in the current directory
/// let files = glob("*.txt").unwrap();
/// for file in files {
///     println!("Found: {}", file.display());
/// }
/// ```
pub fn glob(pattern: &str) -> Result<Vec<PathBuf>> {
    glob_with(pattern, &MatchOptions::new())
        .map_err(|e| FsError::invalid_path(e.to_string()))
        .and_then(|entries| {
            entries
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| FsError::io(e, "Failed to read directory entry"))
        })
}

/// Find files matching a glob pattern with custom options
///
/// # Examples
///
/// ```no_run
/// use rastos::fs::utils::{glob_with_options, GlobOptions};
///
/// // Find files case-insensitively
/// let options = GlobOptions {
///     case_insensitive: true,
///     ..Default::default()
/// };
/// let files = glob_with_options("*.{TXT,txt}", options).unwrap();
/// ```
pub fn glob_with_options(pattern: &str, options: GlobOptions) -> Result<Vec<PathBuf>> {
    let mut match_options = MatchOptions::new();
    match_options.case_sensitive = !options.case_insensitive;
    match_options.require_literal_separator = options.require_literal_separator;

    let pattern = if options.match_hidden {
        pattern.to_string()
    } else {
        // Skip hidden files unless explicitly matched
        if !pattern.starts_with("**/") && !pattern.starts_with(".") && !pattern.starts_with("*/") {
            format!("[!.]*/**/{}", pattern)
        } else {
            pattern.to_string()
        }
    };

    glob_with(&pattern, &match_options)
        .map_err(|e| FsError::invalid_path(e.to_string()))
        .and_then(|entries| {
            entries
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| FsError::io(e, "Failed to read directory entry"))
        })
}

/// Create a new temporary file
///
/// The file will be automatically deleted when the returned `NamedTempFile` is dropped.
///
/// # Examples
///
/// ```no_run
/// use rastos::fs::utils::temp_file;
/// use std::io::Write;
///
/// let mut file = temp_file("prefix_", ".txt").unwrap();
/// writeln!(file, "Temporary data").unwrap();
/// let path = file.path().to_owned();
/// 
/// // File is automatically deleted when `file` is dropped
/// ```
pub fn temp_file(prefix: &str, suffix: &str) -> Result<NamedTempFile> {
    let temp_dir = std::env::temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let mut file_name = OsString::new();
    file_name.push(prefix);
    file_name.push(timestamp.to_string());
    file_name.push(suffix);
    
    let path = temp_dir.join(file_name);
    
    NamedTempFile::new_in(temp_dir)
        .map_err(|e| FsError::io(e, "Failed to create temporary file"))
}

/// Create a new temporary directory
///
/// The directory will be automatically deleted when the returned `TempDir` is dropped.
///
/// # Examples
///
/// ```no_run
/// use rastos::fs::utils::temp_dir;
/// use std::fs::File;
///
/// let dir = temp_dir("myprefix_").unwrap();
/// let file_path = dir.path().join("temp.txt");
/// File::create(&file_path).unwrap();
/// 
/// // Directory and all its contents are automatically deleted when `dir` is dropped
/// ```
pub fn temp_dir(prefix: &str) -> Result<TempDir> {
    let temp_dir = std::env::temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let dir_name = format!("{}temp_{}", prefix, timestamp);
    let path = temp_dir.join(dir_name);
    
    TempDir::new_in(temp_dir, &path)
        .map_err(|e| FsError::io(e, "Failed to create temporary directory"))
}

/// Create a temporary file with the given content
///
/// Returns the path to the created file. The file will be automatically deleted when the program exits.
///
/// # Examples
///
/// ```no_run
/// use rastos::fs::utils::create_temp_file;
///
/// let path = create_temp_file("config_", ".toml", b"[settings]\nkey = \"value\"").unwrap();
/// // Use the file...
/// ```
pub fn create_temp_file(prefix: &str, suffix: &str, content: &[u8]) -> Result<PathBuf> {
    let mut temp_file = temp_file(prefix, suffix)?;
    std::io::Write::write_all(&mut temp_file, content)
        .map_err(|e| FsError::io(e, "Failed to write to temporary file"))?;
    
    let path = temp_file.path().to_owned();
    // Convert to a persistent file that will be deleted when the program exits
    temp_file.keep().map_err(|e| FsError::io(e.error, "Failed to persist temporary file"))?;
    
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn test_glob() -> Result<()> {
        // Create test files
        let dir = temp_dir("test_glob")?;
        let file1 = dir.path().join("test1.txt");
        let file2 = dir.path().join("test2.txt");
        let file3 = dir.path().join("data.dat");
        
        File::create(&file1)?;
        File::create(&file2)?;
        File::create(&file3)?;
        
        // Test basic glob
        let pattern = format!("{}/test*.txt", dir.path().display());
        let files = glob(&pattern)?;
        assert_eq!(files.len(), 2);
        
        // Test with options
        let options = GlobOptions {
            case_insensitive: true,
            match_hidden: false,
            require_literal_separator: false,
        };
        
        let pattern = format!("{}/TEST*.TXT", dir.path().display());
        let files = glob_with_options(&pattern, options)?;
        assert_eq!(files.len(), 2);
        
        Ok(())
    }
    
    #[test]
    fn test_temp_file() -> Result<()> {
        let mut file = temp_file("test_", ".tmp")?;
        let path = file.path().to_owned();
        
        // Write to the temp file
        file.write_all(b"test content")?;
        
        // Check the file exists and has the right content
        let content = fs::read_to_string(&path)?;
        assert_eq!(content, "test content");
        
        // File is deleted when `file` is dropped
        drop(file);
        assert!(!path.exists());
        
        Ok(())
    }
    
    #[test]
    fn test_temp_dir() -> Result<()> {
        let dir = temp_dir("test_dir")?;
        let file_path = dir.path().join("test.txt");
        
        // Create a file in the temp directory
        fs::write(&file_path, "test")?;
        assert!(file_path.exists());
        
        // Directory is deleted when `dir` is dropped
        let path = dir.path().to_owned();
        drop(dir);
        assert!(!path.exists());
        
        Ok(())
    }
}
