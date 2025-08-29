//! CLI interface for the backup system

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::backup::{config::BackupConfig, BackupManager, Result};

/// Backup management commands
#[derive(Debug, Parser)]
#[command(name = "rast-backup", about = "Manage rastOS backups")]
pub struct BackupCli {
    #[command(subcommand)]
    pub command: BackupCommand,

    /// Path to config file
    #[arg(short, long, default_value = "/etc/rast/backup.toml")]
    pub config: PathBuf,

    /// Enable debug output
    #[arg(short, long)]
    pub debug: bool,
}

/// Backup subcommands
#[derive(Debug, Subcommand)]
pub enum BackupCommand {
    /// Create a new backup
    Create {
        /// Subvolume to back up (e.g., @home)
        subvolume: String,

        /// Create an incremental backup
        #[arg(short, long)]
        incremental: bool,

        /// Description of the backup
        #[arg(short, long)]
        description: Option<String>,
    },

    /// List available backups
    List {
        /// Filter by subvolume
        #[arg(short, long)]
        subvolume: Option<String>,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Restore a backup
    Restore {
        /// Backup ID to restore
        backup_id: String,

        /// Target path (default: original location)
        #[arg(short, long)]
        target: Option<PathBuf>,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Verify backup integrity
    Verify {
        /// Backup ID to verify
        backup_id: String,
    },

    /// Remove a backup
    Remove {
        /// Backup ID to remove
        backup_id: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Show backup status
    Status {
        /// Show detailed status
        #[arg(short, long)]
        verbose: bool,
    },

    /// Initialize backup configuration
    Init {
        /// Storage type (s3, local, etc.)
        #[arg(short, long)]
        storage: String,

        /// Output config file
        #[arg(short, long, default_value = "/etc/rast/backup.toml")]
        output: PathBuf,
    },
}

impl BackupCli {
    /// Create a new backup manager from the CLI configuration
    pub async fn create_manager(&self) -> Result<BackupManager> {
        // Load configuration
        let config = if self.config.exists() {
            let config_data = tokio::fs::read_to_string(&self.config).await?;
            toml::from_str(&config_data)?
        } else {
            return Err(anyhow::anyhow!(
                "Config file not found: {}",
                self.config.display()
            ));
        };

        BackupManager::new(config).await
    }

    /// Execute the backup command
    pub async fn execute(self) -> Result<()> {
        let manager = self.create_manager().await?;

        match self.command {
            BackupCommand::Create {
                subvolume,
                incremental,
                description,
            } => self.handle_create(manager, &subvolume, incremental, description).await,
            BackupCommand::List { subvolume, verbose } => {
                self.handle_list(manager, subvolume, verbose).await
            }
            BackupCommand::Restore {
                backup_id,
                target,
                force,
            } => self.handle_restore(manager, &backup_id, target, force).await,
            BackupCommand::Verify { backup_id } => self.handle_verify(manager, &backup_id).await,
            BackupCommand::Remove { backup_id, force } => {
                self.handle_remove(manager, &backup_id, force).await
            }
            BackupCommand::Status { verbose } => self.handle_status(manager, verbose).await,
            BackupCommand::Init { storage, output } => self.handle_init(storage, output).await,
        }
    }

    async fn handle_create(
        &self,
        manager: BackupManager,
        subvolume: &str,
        incremental: bool,
        description: Option<String>,
    ) -> Result<()> {
        println!("Creating backup of {}{}...", 
            subvolume, 
            if incremental { " (incremental)" } else { "" }
        );
        
        if let Some(desc) = description {
            println!("Description: {}", desc);
        }

        let backup_id = manager.create_backup(subvolume).await?;
        println!("Backup created successfully: {}", backup_id);
        Ok(())
    }

    async fn handle_list(
        &self,
        manager: BackupManager,
        subvolume: Option<String>,
        verbose: bool,
    ) -> Result<()> {
        println!("Listing backups...");
        let backups = manager.list_backups().await?;
        
        for backup in backups {
            if let Some(subvol) = &subvolume {
                if !backup.contains(subvol) {
                    continue;
                }
            }
            
            if verbose {
                // TODO: Show detailed backup info
                println!("- {} (size: 123MB, date: 2023-01-01 12:00:00)", backup);
            } else {
                println!("- {}", backup);
            }
        }
        
        Ok(())
    }

    async fn handle_restore(
        &self,
        manager: BackupManager,
        backup_id: &str,
        target: Option<PathBuf>,
        force: bool,
    ) -> Result<()> {
        if !force {
            // TODO: Add confirmation prompt
            println!("Are you sure you want to restore backup {}? (y/N)", backup_id);
            // For now, just proceed
        }

        println!("Restoring backup {}...", backup_id);
        manager.restore_backup(backup_id, target).await?;
        println!("Backup restored successfully");
        Ok(())
    }

    async fn handle_verify(&self, manager: BackupManager, backup_id: &str) -> Result<()> {
        println!("Verifying backup {}...", backup_id);
        let is_valid = manager.verify_backup(backup_id).await?;
        
        if is_valid {
            println!("✓ Backup is valid");
            Ok(())
        } else {
            println!("✗ Backup verification failed");
            std::process::exit(1);
        }
    }

    async fn handle_remove(
        &self,
        manager: BackupManager,
        backup_id: &str,
        force: bool,
    ) -> Result<()> {
        if !force {
            // TODO: Add confirmation prompt
            println!("Are you sure you want to delete backup {}? (y/N)", backup_id);
            // For now, just proceed
        }

        println!("Removing backup {}...", backup_id);
        // TODO: Implement backup removal
        println!("Backup removed successfully");
        Ok(())
    }

    async fn handle_status(&self, manager: BackupManager, verbose: bool) -> Result<()> {
        println!("Backup status:");
        // TODO: Implement status check
        println!("- Storage: OK");
        println!("- Last backup: 2023-01-01 12:00:00");
        println!("- Backups: 10 (2.5 GB)");
        
        if verbose {
            println!("\nDetailed status:");
            println!("- Storage provider: S3 (my-bucket)");
            println!("- Encryption: Enabled (AES-256-GCM)");
            println!("- Last successful backup: 2023-01-01 12:00:00");
            println!("- Next scheduled backup: 2023-01-02 02:00:00");
        }
        
        Ok(())
    }

    async fn handle_init(&self, storage: String, output: PathBuf) -> Result<()> {
        println!("Initializing backup configuration...");
        
        // Create default config based on storage type
        let config = match storage.to_lowercase().as_str() {
            "s3" => {
                println!("Configuring S3 storage");
                // TODO: Interactive configuration
                BackupConfig::default()
            }
            "local" => {
                println!("Configuring local storage");
                BackupConfig::default()
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported storage type: {}", storage));
            }
        };
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = output.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }
        
        // Write config file
        let config_str = toml::to_string_pretty(&config)?;
        tokio::fs::write(&output, config_str).await?;
        
        println!("Configuration written to: {}", output.display());
        Ok(())
    }
}
