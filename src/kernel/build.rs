use std::path::{Path, PathBuf};
use std::process::Command;

use indicatif::{ProgressBar, ProgressStyle};
use log::debug;

use super::error::KernelError;
use crate::kernel::KernelProfile;

/// Builder for compiling Linux kernels
#[derive(Debug)]
pub struct KernelBuilder {
    source_dir: PathBuf,
    build_dir: PathBuf,
    install_dir: PathBuf,
    config_path: Option<PathBuf>,
    profile: KernelProfile,
    jobs: usize,
}

impl KernelBuilder {
    /// Create a new kernel builder
    pub fn new<P: AsRef<Path>>(source_dir: P) -> Self {
        let source_dir = source_dir.as_ref().to_path_buf();
        let build_dir = source_dir.join("build");
        let install_dir = source_dir.join("install");

        Self {
            source_dir,
            build_dir,
            install_dir,
            config_path: None,
            profile: KernelProfile::default(),
            jobs: num_cpus::get(),
        }
    }

    /// Set custom config file path
    pub fn with_config<P: AsRef<Path>>(mut self, config_path: P) -> Self {
        self.config_path = Some(config_path.as_ref().to_path_buf());
        self
    }

    /// Set build profile
    pub fn with_profile(mut self, profile: KernelProfile) -> Self {
        self.profile = profile;
        self
    }

    /// Set number of parallel jobs
    pub fn with_jobs(mut self, jobs: usize) -> Self {
        self.jobs = jobs;
        self
    }

    /// Build the kernel
    pub async fn build(&self) -> Result<(), KernelError> {
        self.prepare_build_dir()?;
        self.configure()?;
        self.compile()?;
        self.install()?;
        Ok(())
    }

    fn prepare_build_dir(&self) -> Result<(), KernelError> {
        if !self.source_dir.exists() {
            return Err(KernelError::MissingFile(self.source_dir.clone()));
        }

        std::fs::create_dir_all(&self.build_dir)?;
        std::fs::create_dir_all(&self.install_dir)?;

        Ok(())
    }

    fn configure(&self) -> Result<(), KernelError> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner:.blue} {msg}")
                .unwrap(),
        );
        pb.set_message("Configuring kernel...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        // Copy config if provided, otherwise use default
        if let Some(config_path) = &self.config_path {
            std::fs::copy(
                config_path,
                self.source_dir.join(".config"),
            )?;
        } else {
            // Generate default config based on profile
            let config = match self.profile {
                KernelProfile::ContainerHost => include_str!("../../configs/linux-container.config"),
                _ => include_str!("../../configs/linux-container.config"), // TODO: Add other profiles
            };
            std::fs::write(self.source_dir.join(".config"), config)?;
        }

        // Run olddefconfig to set defaults for new options
        self.run_command(
            "make",
            &[
                "-C",
                self.source_dir.to_str().unwrap(),
                "O=build",
                "olddefconfig",
            ],
        )?;

        pb.finish_with_message("✓ Configuration complete");
        Ok(())
    }

    fn compile(&self) -> Result<(), KernelError> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"])
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Compiling kernel...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        // Build the kernel
        self.run_command(
            "make",
            &[
                "-C",
                self.source_dir.to_str().unwrap(),
                "O=build",
                &format!("-j{}", self.jobs),
                "all",
            ],
        )?;

        pb.finish_with_message("✓ Kernel compiled successfully");
        Ok(())
    }

    fn install(&self) -> Result<(), KernelError> {
        let pb = ProgressBar::new_spinner();
        pb.set_message("Installing kernel...");

        // Install kernel modules
        self.run_command(
            "make",
            &[
                "-C",
                self.source_dir.to_str().unwrap(),
                "O=build",
                &format!("INSTALL_MOD_PATH={}", self.install_dir.display()),
                "modules_install",
            ],
        )?;

        // Install kernel image
        self.run_command(
            "make",
            &[
                "-C",
                self.source_dir.to_str().unwrap(),
                "O=build",
                &format!("INSTALL_PATH={}/boot", self.install_dir.display()),
                "install",
            ],
        )?;

        pb.finish_with_message("✓ Kernel installed successfully");
        Ok(())
    }

    fn run_command(&self, program: &str, args: &[&str]) -> Result<(), KernelError> {
        debug!("Running: {} {}", program, args.join(" "));
        
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|e| KernelError::Io(e))?;

        if !output.status.success() {
            return Err(KernelError::command_error(program, &output));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_kernel_builder_creation() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary directory for testing
        let temp_dir = tempdir()?;
        let test_path = temp_dir.path().join("linux-test");
        fs::create_dir_all(&test_path)?;

        // Test builder creation
        let builder = KernelBuilder::new(&test_path);
        assert_eq!(builder.source_dir, test_path);
        assert_eq!(builder.jobs, num_cpus::get());

        // Cleanup
        temp_dir.close()?;
        Ok(())
    }
}
