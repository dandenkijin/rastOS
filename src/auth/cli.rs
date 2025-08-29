//! Command-line interface for managing API keys

use clap::{Args, Subcommand};
use std::path::PathBuf;

use crate::auth::{
    ApiKey, ApiKeyConfig, ApiKeyManager, AuthError, ConfigError,
};

/// CLI commands for API key management
#[derive(Debug, Subcommand)]
pub enum ApiKeyCommand {
    /// Add a new API key
    Add(AddKeyArgs),
    
    /// List all API keys
    List,
    
    /// Remove an API key
    Remove(RemoveKeyArgs),
    
    /// Generate a new random API key
    Generate(GenerateKeyArgs),
}

/// Arguments for adding an API key
#[derive(Debug, Args)]
pub struct AddKeyArgs {
    /// The service this key is for (e.g., "backup", "llm")
    #[arg(short, long)]
    pub service: String,
    
    /// The API key value
    #[arg(short, long)]
    pub key: Option<String>,
    
    /// Description of the key
    #[arg(short, long)]
    pub description: Option<String>,
    
    /// Expiration date (YYYY-MM-DD)
    #[arg(long)]
    pub expires: Option<String>,
    
    /// Mark this as the primary key for the service
    #[arg(long)]
    pub primary: bool,
    
    /// Environment variable name for this key
    #[arg(long)]
    pub env_var: Option<String>,
    
    /// Path to the API key configuration file
    #[arg(long, default_value = "/etc/rast/auth/keys.toml")]
    pub config: PathBuf,
}

/// Arguments for removing an API key
#[derive(Debug, Args)]
pub struct RemoveKeyArgs {
    /// The service to remove the key from
    #[arg(short, long)]
    pub service: String,
    
    /// The API key to remove
    #[arg(short, long)]
    pub key: Option<String>,
    
    /// Remove all keys for the service
    #[arg(long)]
    pub all: bool,
    
    /// Path to the API key configuration file
    #[arg(long, default_value = "/etc/rast/auth/keys.toml")]
    pub config: PathBuf,
}

/// Arguments for generating a new API key
#[derive(Debug, Args)]
pub struct GenerateKeyArgs {
    /// The service this key is for
    #[arg(short, long)]
    pub service: String,
    
    /// Key length in bytes
    #[arg(short, long, default_value = "32")]
    pub length: usize,
    
    /// Save the generated key to the configuration
    #[arg(long)]
    pub save: bool,
    
    /// Path to the API key configuration file
    #[arg(long, default_value = "/etc/rast/auth/keys.toml")]
    pub config: PathBuf,
}

/// Handle API key management commands
pub async fn handle_api_key_command(cmd: ApiKeyCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        ApiKeyCommand::Add(args) => handle_add_key(args).await?,
        ApiKeyCommand::List => handle_list_keys().await?,
        ApiKeyCommand::Remove(args) => handle_remove_key(args).await?,
        ApiKeyCommand::Generate(args) => handle_generate_key(args).await?,
    }
    
    Ok(())
}

/// Handle adding a new API key
async fn handle_add_key(args: AddKeyArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Load or create config
    let mut config = if args.config.exists() {
        ApiKeyConfig::from_file(&args.config)?
    } else {
        // Create parent directory if it doesn't exist
        if let Some(parent) = args.config.parent() {
            std::fs::create_dir_all(parent)?;
        }
        ApiKeyConfig::default()
    };
    
    // Get or generate the key
    let key = if let Some(key) = args.key {
        key
    } else {
        // Generate a random key if none provided
        use rand::RngCore;
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        base64::encode_config(&key, base64::URL_SAFE_NO_PAD)
    };
    
    // Parse expiration date if provided
    let expires_at = if let Some(expires) = &args.expires {
        use chrono::NaiveDate;
        let date = NaiveDate::parse_from_str(expires, "%Y-%m-%d")?;
        Some(date.and_hms_opt(0, 0, 0).unwrap().timestamp())
    } else {
        None
    };
    
    // Add the key to the service
    let service_entry = config.keys.entry(args.service.clone()).or_default();
    
    if args.primary || service_entry.primary.is_none() {
        // Set as primary key
        if let Some(old_primary) = service_entry.primary.take() {
            // Move old primary to additional keys
            service_entry.additional.push(old_primary);
        }
        service_entry.primary = Some(key.clone());
    } else {
        // Add as additional key
        service_entry.additional.push(key.clone());
    }
    
    // Set environment variable if specified
    if let Some(env_var) = args.env_var {
        service_entry.env_var = Some(env_var);
    }
    
    // Save the configuration
    config.save_to_file(&args.config)?;
    
    println!("Added {} key for service '{}'", 
        if args.primary { "primary" } else { "additional" }, 
        args.service
    );
    
    if let Some(expires_at) = expires_at {
        use chrono::NaiveDateTime;
        let dt = NaiveDateTime::from_timestamp_opt(expires_at, 0).unwrap();
        println!("Key expires at: {}", dt.format("%Y-%m-%d %H:%M:%S"));
    }
    
    Ok(())
}

/// Handle listing API keys
async fn handle_list_keys() -> Result<(), Box<dyn std::error::Error>> {
    // For now, just list the keys in memory
    // In a real implementation, we would load from the config file
    println!("Listing API keys is not yet implemented");
    Ok(())
}

/// Handle removing an API key
async fn handle_remove_key(args: RemoveKeyArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    if !args.config.exists() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "API key configuration file not found",
        )));
    }
    
    let mut config = ApiKeyConfig::from_file(&args.config)?;
    
    // Remove the key(s)
    if let Some(keys) = config.keys.get_mut(&args.service) {
        if args.all {
            // Remove all keys for the service
            config.keys.remove(&args.service);
            println!("Removed all keys for service '{}'", args.service);
        } else if let Some(key) = &args.key {
            // Remove a specific key
            if keys.primary.as_ref() == Some(key) {
                keys.primary = None;
                println!("Removed primary key for service '{}'", args.service);
            } else if let Some(pos) = keys.additional.iter().position(|k| k == key) {
                keys.additional.remove(pos);
                println!("Removed additional key for service '{}'", args.service);
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Key not found for service '{}'", args.service),
                )));
            }
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Either --key or --all must be specified",
            )));
        }
        
        // Save the updated config
        config.save_to_file(&args.config)?;
    } else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No keys found for service '{}'", args.service),
        )));
    }
    
    Ok(())
}

/// Handle generating a new API key
async fn handle_generate_key(args: GenerateKeyArgs) -> Result<(), Box<dyn std::error::Error>> {
    use rand::RngCore;
    
    // Generate a random key
    let mut key = vec![0u8; args.length];
    rand::thread_rng().fill_bytes(&mut key);
    let key = base64::encode_config(&key, base64::URL_SAFE_NO_PAD);
    
    println!("Generated API key: {}", key);
    
    if args.save {
        // Add the key to the config
        let add_args = AddKeyArgs {
            service: args.service,
            key: Some(key),
            description: Some("Auto-generated key".to_string()),
            expires: None,
            primary: false,
            env_var: None,
            config: args.config,
        };
        
        handle_add_key(add_args).await?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_generate_key() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("keys.toml");
        
        // Test generating a key without saving
        let gen_args = GenerateKeyArgs {
            service: "test-service".to_string(),
            length: 32,
            save: false,
            config: config_path.clone(),
        };
        
        handle_generate_key(gen_args).await.unwrap();
        
        // Config file should not exist since we didn't save
        assert!(!config_path.exists());
        
        // Test generating and saving a key
        let gen_args = GenerateKeyArgs {
            service: "test-service".to_string(),
            length: 32,
            save: true,
            config: config_path.clone(),
        };
        
        handle_generate_key(gen_args).await.unwrap();
        
        // Config file should exist now
        assert!(config_path.exists());
        
        // Load the config and verify the key was saved
        let config = ApiKeyConfig::from_file(&config_path).unwrap();
        assert!(config.keys.contains_key("test-service"));
        assert!(config.keys.get("test-service").unwrap().primary.is_some());
    }
    
    #[tokio::test]
    async fn test_add_and_remove_key() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("keys.toml");
        
        // Add a key
        let add_args = AddKeyArgs {
            service: "test-service".to_string(),
            key: Some("test-key".to_string()),
            description: Some("Test key".to_string()),
            expires: None,
            primary: true,
            env_var: Some("TEST_API_KEY".to_string()),
            config: config_path.clone(),
        };
        
        handle_add_key(add_args).await.unwrap();
        
        // Load the config and verify the key was added
        let config = ApiKeyConfig::from_file(&config_path).unwrap();
        let service_keys = config.keys.get("test-service").unwrap();
        assert_eq!(service_keys.primary.as_ref().unwrap(), "test-key");
        assert_eq!(service_keys.env_var.as_ref().unwrap(), "TEST_API_KEY");
        
        // Remove the key
        let remove_args = RemoveKeyArgs {
            service: "test-service".to_string(),
            key: Some("test-key".to_string()),
            all: false,
            config: config_path.clone(),
        };
        
        handle_remove_key(remove_args).await.unwrap();
        
        // Verify the key was removed
        let config = ApiKeyConfig::from_file(&config_path).unwrap();
        assert!(config.keys.get("test-service").is_none());
    }
}
