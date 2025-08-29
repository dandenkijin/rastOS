//! Package management for rastOS

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use std::fs;
use std::path::PathBuf;

/// Error type for package management operations
#[derive(Error, Debug)]
pub enum PackageError {
    /// An I/O error occurred during package operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Failed to parse package list or configuration
    #[error("Failed to parse package list: {0}")]
    ParseError(String),
    
    /// A package operation failed to complete successfully
    #[error("Package operation failed: {0}")]
    OperationFailed(String),
}

/// Package specification with version constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSpec {
    /// Package name
    pub name: String,
    
    /// Optional version constraint (e.g., ">=1.2.3")
    pub version: Option<String>,
    
    /// Optional source (e.g., "official", "aur")
    pub source: Option<String>,
    
    /// Optional installation options
    pub options: Option<Vec<String>>,
}

/// Package list specification
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageList {
    /// List of packages to install
    pub packages: Vec<PackageSpec>,
    
    /// Optional pre-installation commands
    pub pre_install: Option<Vec<String>>,
    
    /// Optional post-installation commands
    pub post_install: Option<Vec<String>>,
}

/// Manages system packages
pub struct PackageManager {
    /// The base path for package management operations
    #[allow(dead_code)]
    base_path: PathBuf,
    
    /// Whether to show verbose output
    verbose: bool,
}

impl PackageManager {
    /// Create a new package manager instance
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
            verbose: false,
        }
    }
    
    /// Enable verbose output
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    /// Install packages from a declarative package list file
    pub fn install_from_file<P: AsRef<Path>>(&self, path: P) -> Result<(), PackageError> {
        let content = fs::read_to_string(&path)?;
        let pkg_list: PackageList = toml::from_str(&content)
            .map_err(|e| PackageError::ParseError(e.to_string()))?;
        
        self.install_list(&pkg_list)
    }
    
    /// Install packages from a PackageList
    pub fn install_list(&self, pkg_list: &PackageList) -> Result<(), PackageError> {
        if let Some(cmds) = &pkg_list.pre_install {
            self.run_commands(cmds, "pre-install")?;
        }
        
        // Group packages by source for batch processing
        let mut official_pkgs = Vec::new();
        let mut aur_pkgs = Vec::new();
        
        for pkg in &pkg_list.packages {
            match pkg.source.as_deref() {
                Some("aur") | None => aur_pkgs.push(pkg),
                _ => official_pkgs.push(pkg),
            }
        }
        
        // Install official packages
        if !official_pkgs.is_empty() {
            self.install_official_packages(&official_pkgs)?;
        }
        
        // Install AUR packages
        if !aur_pkgs.is_empty() {
            self.install_aur_packages(&aur_pkgs)?;
        }
        
        if let Some(cmds) = &pkg_list.post_install {
            self.run_commands(cmds, "post-install")?;
        }
        
        Ok(())
    }
    
    /// Install official repository packages
    fn install_official_packages(&self, packages: &[&PackageSpec]) -> Result<(), PackageError> {
        if self.verbose {
            println!("Installing {} official packages...", packages.len());
        }
        
        // Convert package specs to pacman format
        let pkg_args: Vec<String> = packages.iter()
            .map(|p| {
                if let Some(ver) = &p.version {
                    format!("{} {}", p.name, ver)
                } else {
                    p.name.clone()
                }
            })
            .collect();
        
        // Execute pacman command
        self.run_command("pacman", &["-S", "--noconfirm", "--needed", &pkg_args.join(" ")])?;
        
        Ok(())
    }
    
    /// Install AUR packages
    fn install_aur_packages(&self, packages: &[&PackageSpec]) -> Result<(), PackageError> {
        if self.verbose {
            println!("Installing {} AUR packages...", packages.len());
        }
        
        // Convert package specs to AUR helper format
        let pkg_args: Vec<String> = packages.iter()
            .map(|p| {
                if let Some(ver) = &p.version {
                    format!("{}@{}", p.name, ver)
                } else {
                    p.name.clone()
                }
            })
            .collect();
        
        // Use paru as AUR helper
        self.run_command("paru", &["-S", "--noconfirm", "--needed", &pkg_args.join(" ")])?;
        
        Ok(())
    }
    
    /// Run system commands with error handling
    fn run_commands(&self, commands: &[String], context: &str) -> Result<(), PackageError> {
        for cmd in commands {
            if self.verbose {
                println!("Running {} command: {}", context, cmd);
            }
            
            self.run_command("sh", &["-c", cmd])
                .map_err(|e| PackageError::OperationFailed(
                    format!("{} command failed: {} - {}", context, cmd, e)
                ))?;
        }
        Ok(())
    }
    
    /// Execute a system command
    fn run_command(&self, cmd: &str, args: &[&str]) -> Result<(), PackageError> {
        let output = std::process::Command::new(cmd)
            .args(args)
            .output()
            .map_err(|e| PackageError::Io(e))?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PackageError::OperationFailed(
                format!("Command '{}' failed: {}", cmd, stderr)
            ));
        }
        
        if self.verbose {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                println!("{} {}\n{}", cmd, args.join(" "), stdout);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_package_manager_creation() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let test_path = temp_dir.path().to_str().unwrap();
        let _pm = PackageManager::new(test_path);
        temp_dir.close()?;
        Ok(())
    }
}
