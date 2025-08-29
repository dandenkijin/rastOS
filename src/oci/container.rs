//! Container management for OCI runtime

use super::*;
use std::path::{Path, PathBuf};
use oci_spec::runtime::{Spec, SpecBuilder, LinuxBuilder, ProcessBuilder, RootBuilder};

/// Represents an OCI container instance
#[derive(Debug)]
pub struct Container {
    /// Container ID
    id: String,
    /// Path to the container bundle
    bundle: PathBuf,
    /// OCI runtime specification
    spec: Spec,
    /// Container status
    status: ContainerStatus,
}

/// Represents the status of a container
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerStatus {
    Created,
    Running,
    Stopped,
    Paused,
    Error,
}

impl Default for ContainerStatus {
    fn default() -> Self {
        Self::Created
    }
}

impl Container {
    /// Create a new container instance
    pub fn new(id: &str, bundle: &Path) -> Result<Self> {
        let config_path = bundle.join("config.json");
        let spec = Spec::load(config_path)?;
        
        Ok(Self {
            id: id.to_string(),
            bundle: bundle.to_path_buf(),
            spec,
            status: ContainerStatus::Created,
        })
    }
    
    /// Start the container
    pub fn start(&mut self) -> Result<()> {
        // TODO: Implement container startup logic
        // 1. Create namespaces
        // 2. Set up cgroups
        // 3. Set up rootfs
        // 4. Start the container process
        
        self.status = ContainerStatus::Running;
        Ok(())
    }
    
    /// Stop the container
    pub fn stop(&mut self) -> Result<()> {
        // TODO: Implement container stop logic
        self.status = ContainerStatus::Stopped;
        Ok(())
    }
    
    /// Get the container's current status
    pub fn status(&self) -> ContainerStatus {
        self.status
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
        let process = self.process.ok_or_else(|| 
            ContainerError::InvalidConfig("Process configuration is required".to_string())
        )?;
        
        let linux = self.linux.ok_or_else(|| 
            ContainerError::InvalidConfig("Linux configuration is required".to_string())
        )?;
        
        let mut spec_builder = SpecBuilder::default()
            .process(process.build()?)
            .linux(linux.build()?);
            
        if let Some(root) = self.root {
            spec_builder = spec_builder.root(RootBuilder::default()
                .path(root)
                .build()?);
        }
        
        let spec = spec_builder.build()?;
        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use oci_spec::runtime::ProcessBuilder as SpecProcessBuilder;
    use oci_spec::runtime::LinuxBuilder as SpecLinuxBuilder;

    #[test]
    fn test_container_lifecycle() -> Result<()> {
        let temp_dir = tempdir()?;
        let bundle = temp_dir.path();
        
        // Create a minimal OCI config
        let config_path = bundle.join("config.json");
        let process = SpecProcessBuilder::default()
            .cwd("/")
            .args(vec!["/bin/sh".to_string()])
            .build()?;
            
        let linux = SpecLinuxBuilder::default().build()?;
            
        let spec = SpecBuilder::default()
            .process(process)
            .linux(linux)
            .build()?;
            
        spec.save(config_path)?;
        
        // Test container creation
        let container = Container::new("test-container", bundle)?;
        assert_eq!(container.id, "test-container");
        
        // Test status management
        let mut container = container;
        container.start()?;
        assert_eq!(container.status(), ContainerStatus::Running);
        
        container.stop()?;
        assert_eq!(container.status(), ContainerStatus::Stopped);
        
        Ok(())
    }
    
    #[test]
    fn test_container_builder() -> Result<()> {
        let process_builder = ProcessBuilder::default()
            .cwd("/")
            .args(vec!["/bin/sh".to_string()]);
            
        let linux_builder = LinuxBuilder::default();
        
        let spec = ContainerBuilder::new("test-builder")
            .process(process_builder)
            .linux(linux_builder)
            .build()?;
            
        assert!(spec.process().is_some());
        assert!(spec.linux().is_some());
        
        Ok(())
    }
}
