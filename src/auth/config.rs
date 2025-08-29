//! Configuration for API key authentication

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

use super::{ApiKey, ApiKeyManager, AuthError};

/// Error type for API key configuration
#[derive(Error, Debug)]
pub enum ConfigError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// TOML deserialization error
    #[error("TOML deserialization error: {0}")]
    Toml(#[from] toml::de::Error),
    
    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),
}

/// Result type for configuration operations
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Configuration for API keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// Default environment variable prefix
    #[serde(default = "default_env_prefix")]
    pub env_prefix: String,
    
    /// API keys by service
    #[serde(default)]
    pub keys: HashMap<String, ServiceKeys>,
}

/// API keys for a specific service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceKeys {
    /// The primary API key for this service
    pub primary: Option<String>,
    
    /// Additional API keys for this service
    #[serde(default)]
    pub additional: Vec<String>,
    
    /// Environment variable to override the API key
    pub env_var: Option<String>,
}

fn default_env_prefix() -> String {
    "RAST".to_string()
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            env_prefix: default_env_prefix(),
            keys: HashMap::new(),
        }
    }
}

impl ApiKeyConfig {
    /// Load API key configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save API key configuration to a TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    /// Get the environment variable name for a service
    pub fn env_var_for_service(&self, service: &str) -> String {
        format!("{}_API_KEY", self.env_prefix.to_uppercase())
    }
    
    /// Get the API key for a service, checking environment variables first
    pub fn get_key(&self, service: &str) -> Option<String> {
        // Check environment variable first
        if let Some(env_var) = self.keys.get(service).and_then(|s| s.env_var.as_ref()) {
            if let Ok(key) = std::env::var(env_var) {
                return Some(key);
            }
        }
        
        // Fall back to config file
        self.keys
            .get(service)
            .and_then(|s| s.primary.clone())
    }
    
    /// Add all keys to an ApiKeyManager
    pub fn add_to_manager(&self, manager: &ApiKeyManager) -> Result<()> {
        for (service, keys) in &self.keys {
            if let Some(key) = &keys.primary {
                manager.add_key(ApiKey {
                    key: key.clone(),
                    service: service.clone(),
                    description: Some("Primary key from config".to_string()),
                    expires_at: None,
                })?;
            }
            
            for (i, key) in keys.additional.iter().enumerate() {
                manager.add_key(ApiKey {
                    key: key.clone(),
                    service: service.clone(),
                    description: Some(format!("Additional key #{} from config", i + 1)),
                    expires_at: None,
                })?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_load_config() {
        let mut config = ApiKeyConfig::default();
        
        // Add some test keys
        let mut service_keys = ServiceKeys {
            primary: Some("test-primary-key".to_string()),
            additional: vec![
                "test-additional-1".to_string(),
                "test-additional-2".to_string(),
            ],
            env_var: Some("TEST_API_KEY".to_string()),
        };
        
        config.keys.insert("test-service".to_string(), service_keys);
        
        // Write to a temporary file
        let mut file = NamedTempFile::new().unwrap();
        let config_str = toml::to_string_pretty(&config).unwrap();
        write!(file, "{}", config_str).unwrap();
        
        // Load it back
        let loaded = ApiKeyConfig::from_file(file.path()).unwrap();
        
        // Verify the loaded config
        assert_eq!(loaded.env_prefix, "RAST");
        assert!(loaded.keys.contains_key("test-service"));
        let keys = loaded.keys.get("test-service").unwrap();
        assert_eq!(keys.primary.as_ref().unwrap(), "test-primary-key");
        assert_eq!(keys.additional.len(), 2);
        assert_eq!(keys.env_var.as_ref().unwrap(), "TEST_API_KEY");
    }
    
    #[test]
    fn test_get_key() {
        let mut config = ApiKeyConfig::default();
        
        // Add a test service with an environment variable
        let service_keys = ServiceKeys {
            primary: Some("test-primary-key".to_string()),
            additional: vec!["test-additional-1".to_string()],
            env_var: Some("TEST_API_KEY".to_string()),
        };
        
        config.keys.insert("test-service".to_string(), service_keys);
        
        // Test getting the primary key
        assert_eq!(
            config.get_key("test-service").unwrap(),
            "test-primary-key"
        );
        
        // Test getting a non-existent service
        assert!(config.get_key("non-existent").is_none());
    }
    
    #[test]
    fn test_env_var_override() {
        let mut config = ApiKeyConfig::default();
        
        // Add a test service with an environment variable
        let service_keys = ServiceKeys {
            primary: Some("test-primary-key".to_string()),
            additional: vec!["test-additional-1".to_string()],
            env_var: Some("TEST_API_KEY".to_string()),
        };
        
        config.keys.insert("test-service".to_string(), service_keys);
        
        // Set the environment variable
        std::env::set_var("TEST_API_KEY", "env-var-key");
        
        // Should get the key from the environment variable
        assert_eq!(
            config.get_key("test-service").unwrap(),
            "env-var-key"
        );
        
        // Clean up
        std::env::remove_var("TEST_API_KEY");
    }
    
    #[test]
    fn test_add_to_manager() {
        let mut config = ApiKeyConfig::default();
        
        // Add a test service
        let service_keys = ServiceKeys {
            primary: Some("test-primary-key".to_string()),
            additional: vec!["test-additional-1".to_string()],
            env_var: None,
        };
        
        config.keys.insert("test-service".to_string(), service_keys);
        
        // Add to manager
        let manager = ApiKeyManager::new();
        config.add_to_manager(&manager).unwrap();
        
        // Verify keys were added
        assert!(manager.validate_key("test-primary-key", "test-service").is_ok());
        assert!(manager.validate_key("test-additional-1", "test-service").is_ok());
        
        // Verify invalid key
        assert!(manager.validate_key("invalid-key", "test-service").is_err());
    }
}
