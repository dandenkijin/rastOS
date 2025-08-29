//! Container management for rastOS
//!
//! This module provides container management functionality that adheres to the
//! Open Container Initiative (OCI) Runtime Specification.

use oci_spec::runtime::{LinuxBuilder, ProcessBuilder, Spec, SpecBuilder, State};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during container operations
#[derive(Debug, Error)]
pub enum ContainerError {
    #[error("Container not found: {0}")]
    NotFound(String),
    
    #[error("Container already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Runtime error: {0}")]
    Runtime(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("OCI spec error: {0}")]
    OciSpec(#[from] oci_spec::OciSpecError),
}

/// Result type for container operations
pub type Result<T> = std::result::Result<T, ContainerError>;

/// Represents a container instance
#[derive(Debug)]
pub struct Container {
    /// Container ID
    id: String,
    /// Path to the container bundle
    bundle: PathBuf,
    /// OCI runtime specification
    spec: Spec,
    /// Container state
    state: State,
}

impl Container {
    /// Create a new container instance
    pub fn new(id: &str, bundle: &Path) -> Result<Self> {
        let config_path = bundle.join("config.json");
        let spec = Spec::load(&config_path)?;
        
        Ok(Self {
            id: id.to_string(),
            bundle: bundle.to_path_buf(),
            spec,
            state: State::default(),
        })
    }
    
    /// Start the container
    pub fn start(&mut self) -> Result<()> {
        // TODO: Implement container startup logic
        // 1. Create namespaces
        // 2. Set up cgroups
        // 3. Set up rootfs
        // 4. Start the container process
        
        self.state.status = oci_spec::runtime::Status::Running;
        Ok(())
    }
    
    /// Stop the container
    pub fn stop(&mut self) -> Result<()> {
        // TODO: Implement container stop logic
        self.state.status = oci_spec::runtime::Status::Stopped;
        Ok(())
    }
    
    /// Get the container's current state
    pub fn state(&self) -> &State {
        &self.state
    }
    
    /// Get the container's OCI runtime specification
    pub fn spec(&self) -> &Spec {
        &self.spec
    }
}

/// Builder for creating container specifications
#[derive(Default)]
pub struct ContainerBuilder {
    id: String,
    root: Option<PathBuf>,
    process: Option<ProcessBuilder>,
    linux: Option<LinuxBuilder>,
}

impl ContainerBuilder {
    /// Create a new container builder
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            ..Default::default()
        }
    }

    /// Set the root filesystem path
    pub fn root(mut self, path: &Path) -> Self {
        self.root = Some(path.to_path_buf());
        self
    }

    /// Set the process configuration
    pub fn process(mut self, process: ProcessBuilder) -> Self {
        self.process = Some(process);
        self
    }

    /// Set the Linux-specific configuration
    pub fn linux(mut self, linux: LinuxBuilder) -> Self {
        self.linux = Some(linux);
        self
    }

    /// Build the container specification
    pub fn build(self) -> Result<Spec> {
        let process = self.process.ok_or_else(|| ContainerError::InvalidConfig("Process configuration is required".to_string()))?;
        let linux = self.linux.ok_or_else(|| ContainerError::InvalidConfig("Linux configuration is required".to_string()))?;
        
        let spec = SpecBuilder::default()
            .process(process)
            .linux(linux)
            .root(self.root.map(|r| oci_spec::runtime::RootBuilder::default().path(r).build().unwrap()))
            .build()
            .map_err(|e| ContainerError::InvalidConfig(e.to_string()))?;
            
        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use oci_spec::runtime::{ProcessBuilder, LinuxBuilder};

    #[test]
    fn test_container_creation() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let bundle = temp_dir.path();
        
        // Create a minimal OCI config
        let config_path = bundle.join("config.json");
        let spec = SpecBuilder::default()
            .process(
                ProcessBuilder::default()
                    .cwd("/")
                    .args(vec!["/bin/sh".to_string()])
                    .build()?
            )
            .linux(
                LinuxBuilder::default()
                    .build()?
            )
            .build()?;
            
        spec.save(config_path)?;
        
        // Test container creation
        let container = Container::new("test-container", bundle)?;
        assert_eq!(container.id, "test-container");
        
        // Test state management
        let mut container = container;
        container.start()?;
        assert_eq!(container.state().status, oci_spec::runtime::Status::Running);
        
        container.stop()?;
        assert_eq!(container.state().status, oci_spec::runtime::Status::Stopped);
        
        Ok(())
    }
    
    #[test]
    fn test_container_builder() -> Result<()> {
        let process = ProcessBuilder::default()
            .cwd("/")
            .args(vec!["/bin/sh".to_string()])
            .build()?;
            
        let linux = LinuxBuilder::default().build()?;
        
        let spec = ContainerBuilder::new("test-builder")
            .process(process)
            .linux(linux)
            .build()?;
            
        assert!(spec.process().is_some());
        assert!(spec.linux().is_some());
        
        Ok(())
    }
}
