#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::all)]

//! Simple CLI tool for building Linux kernels

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::LevelFilter;
use rastos::kernel::{KernelBuilder, KernelProfile};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to Linux kernel source directory
    #[arg(short, long)]
    source: PathBuf,

    /// Output directory for built kernel
    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    /// Number of parallel jobs
    #[arg(short, long, default_value_t = num_cpus::get())]
    jobs: usize,

    /// Build profile
    #[arg(short, long, value_enum, default_value_t = KernelProfile::ContainerHost)]
    profile: KernelProfile,

    /// Path to custom kernel config (optional)
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Enable debug output
    #[arg(short, long)]
    debug: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the kernel
    Build,
    /// Clean build directory
    Clean,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    TermLogger::init(
        log_level,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    // Create kernel builder
    let mut builder = KernelBuilder::new(&cli.source)
        .with_jobs(cli.jobs)
        .with_profile(cli.profile);

    if let Some(config_path) = cli.config {
        builder = builder.with_config(config_path);
    }

    match cli.command {
        Commands::Build => {
            println!("Building kernel from source: {}", cli.source.display());
            println!("Using profile: {:?}", cli.profile);
            println!("Parallel jobs: {}", cli.jobs);

            builder.build().await?;
            println!("\n✓ Kernel build completed successfully!");
        }
        Commands::Clean => {
            println!("Cleaning build directory...");
            std::fs::remove_dir_all(cli.source.join("build"))?;
            println!("✓ Build directory cleaned");
        }
    }

    Ok(())
}
