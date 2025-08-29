//! Kernel configuration management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use clap::ValueEnum;

/// Represents a kernel configuration profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum KernelProfile {
    /// Container-optimized configuration
    ContainerHost,
    /// Development configuration with debugging
    Development,
    /// Production configuration with security hardening
    Production,
}

impl Default for KernelProfile {
    fn default() -> Self {
        Self::ContainerHost
    }
}

/// Kernel configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelConfig {
    /// Kernel source directory
    pub source_dir: PathBuf,
    /// Build directory
    pub build_dir: PathBuf,
    /// Installation directory
    pub install_dir: PathBuf,
    /// Build profile
    pub profile: KernelProfile,
    /// Number of parallel jobs
    pub jobs: usize,
    /// Additional make arguments
    pub make_args: Vec<String>,
}

impl Default for KernelConfig {
    fn default() -> Self {
        let source_dir = PathBuf::from("/usr/src/linux");
        let build_dir = source_dir.join("build");
        let install_dir = PathBuf::from("/boot");

        Self {
            source_dir,
            build_dir,
            install_dir,
            profile: KernelProfile::default(),
            jobs: num_cpus::get(),
            make_args: Vec::new(),
        }
    }
}

/// Errors that can occur during kernel configuration
#[derive(Error, Debug)]
pub enum ConfigError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    /// Missing required file or directory
    #[error("Missing file or directory: {0}")]
    MissingPath(PathBuf),
}

impl KernelConfig {
    /// Create a new kernel configuration
    pub fn new(source_dir: PathBuf) -> Self {
        let build_dir = source_dir.join("build");
        let install_dir = PathBuf::from("/boot");

        Self {
            source_dir,
            build_dir,
            install_dir,
            ..Default::default()
        }
    }

    /// Set the build profile
    pub fn with_profile(mut self, profile: KernelProfile) -> Self {
        self.profile = profile;
        self
    }

    /// Set the number of parallel jobs
    pub fn with_jobs(mut self, jobs: usize) -> Self {
        self.jobs = jobs;
        self
    }

    /// Set the build directory
    pub fn with_build_dir(mut self, build_dir: PathBuf) -> Self {
        self.build_dir = build_dir;
        self
    }

    /// Set the installation directory
    pub fn with_install_dir(mut self, install_dir: PathBuf) -> Self {
        self.install_dir = install_dir;
        self
    }

    /// Add a make argument
    pub fn add_make_arg<S: Into<String>>(mut self, arg: S) -> Self {
        self.make_args.push(arg.into());
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !self.source_dir.exists() {
            return Err(ConfigError::MissingPath(self.source_dir.clone()));
        }

        // Ensure build and install directories exist or can be created
        for dir in &[&self.build_dir, &self.install_dir] {
            if !dir.exists() {
                std::fs::create_dir_all(dir)?;
            }
        }

        Ok(())
    }
}
