//! rastOS Backup Utility
//! 
//! Command-line interface for managing rastOS backups.

use anyhow::Result;
use clap::Parser;
use rastos::backup::cli::BackupCli;
use std::process;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse command line arguments
    let cli = BackupCli::parse();

    // Execute the command
    if let Err(e) = cli.execute().await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
