//! Authentication and authorization module for rastOS
//! 
//! Provides a unified interface for API key authentication across different services.

use std::collections::HashMap;
use std::env;
use std::sync::RwLock;
use thiserror::Error;

/// Error type for authentication operations
#[derive(Error, Debug)]
pub enum AuthError {
    /// Missing API key
    #[error("API key is required but not provided")]
    MissingApiKey,
    
    /// Invalid API key
    #[error("Invalid API key")]
    InvalidApiKey,
    
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Other errors
    #[error("Authentication error: {0}")]
    Other(String),
}

/// Result type for authentication operations
pub type Result<T> = std::result::Result<T, AuthError>;

/// Represents an API key with associated metadata
#[derive(Debug, Clone)]
pub struct ApiKey {
    /// The actual key value
    pub key: String,
    /// Service this key is for (e.g., "backup", "llm")
    pub service: String,
    /// Optional description
    pub description: Option<String>,
    /// Optional expiration timestamp (UNIX timestamp)
    pub expires_at: Option<i64>,
}

/// Manages API keys for different services
#[derive(Default)]
pub struct ApiKeyManager {
    keys: RwLock<HashMap<String, ApiKey>>,
}

impl ApiKeyManager {
    /// Create a new ApiKeyManager
    pub fn new() -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
        }
    }
    
    /// Add a new API key
    pub fn add_key(&self, key: ApiKey) -> Result<()> {
        let mut keys = self.keys.write().map_err(|e| AuthError::Other(e.to_string()))?;
        keys.insert(key.key.clone(), key);
        Ok(())
    }
    
    /// Remove an API key
    pub fn remove_key(&self, key: &str) -> Result<()> {
        let mut keys = self.keys.write().map_err(|e| AuthError::Other(e.to_string()))?;
        keys.remove(key);
        Ok(())
    }
    
    /// Validate an API key for a specific service
    pub fn validate_key(&self, key: &str, service: &str) -> Result<()> {
        let keys = self.keys.read().map_err(|e| AuthError::Other(e.to_string()))?;
        
        if let Some(api_key) = keys.get(key) {
            // Check if key is for the correct service
            if api_key.service != service {
                return Err(AuthError::InvalidApiKey);
            }
            
            // Check if key has expired
            if let Some(expires_at) = api_key.expires_at {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| AuthError::Other("System time is before UNIX_EPOCH".to_string()))?
                    .as_secs() as i64;
                
                if now > expires_at {
                    return Err(AuthError::Other("API key has expired".to_string()));
                }
            }
            
            Ok(())
        } else {
            Err(AuthError::InvalidApiKey)
        }
    }
}

/// Get an API key from environment variables
/// 
/// # Arguments
/// * `env_var` - The environment variable name to look for
/// * `service` - The service this key is for (for validation)
/// * `key_manager` - The ApiKeyManager to validate against
/// 
/// # Returns
/// The API key if found and valid, or an error
pub fn get_api_key_from_env(
    env_var: &str, 
    service: &str, 
    key_manager: &ApiKeyManager
) -> Result<String> {
    let key = env::var(env_var)
        .map_err(|_| AuthError::MissingApiKey)?;
    
    key_manager.validate_key(&key, service)?;
    
    Ok(key)
}

/// Get an API key from command line arguments or environment variables
/// 
/// # Arguments
/// * `arg_key` - The API key from command line arguments (if any)
/// * `env_var` - The environment variable name to fall back to
/// * `service` - The service this key is for (for validation)
/// * `key_manager` - The ApiKeyManager to validate against
/// 
/// # Returns
/// The API key if found and valid, or an error
pub fn get_api_key(
    arg_key: Option<String>,
    env_var: &str,
    service: &str,
    key_manager: &ApiKeyManager
) -> Result<String> {
    if let Some(key) = arg_key {
        key_manager.validate_key(&key, service)?;
        return Ok(key);
    }
    
    get_api_key_from_env(env_var, service, key_manager)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_api_key_validation() {
        let manager = ApiKeyManager::new();
        let key = "test-key".to_string();
        
        // Add a test key
        manager.add_key(ApiKey {
            key: key.clone(),
            service: "backup".to_string(),
            description: Some("Test key".to_string()),
            expires_at: None,
        }).unwrap();
        
        // Test valid key
        assert!(manager.validate_key(&key, "backup").is_ok());
        
        // Test invalid service
        assert!(matches!(
            manager.validate_key(&key, "llm"),
            Err(AuthError::InvalidApiKey)
        ));
        
        // Test non-existent key
        assert!(matches!(
            manager.validate_key("invalid-key", "backup"),
            Err(AuthError::InvalidApiKey)
        ));
    }
    
    #[test]
    fn test_expired_key() {
        let manager = ApiKeyManager::new();
        let key = "expired-key".to_string();
        
        // Add an expired key (expired 1 hour ago)
        let one_hour_ago = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64) - 3600;
            
        manager.add_key(ApiKey {
            key: key.clone(),
            service: "backup".to_string(),
            description: Some("Expired key".to_string()),
            expires_at: Some(one_hour_ago),
        }).unwrap();
        
        // Should fail with expired key
        assert!(matches!(
            manager.validate_key(&key, "backup"),
            Err(AuthError::Other(msg)) if msg.contains("expired")
        ));
    }
    
    #[test]
    fn test_get_api_key_from_env() {
        let manager = ApiKeyManager::new();
        let test_key = "test-env-key";
        
        // Add a test key
        manager.add_key(ApiKey {
            key: test_key.to_string(),
            service: "backup".to_string(),
            description: None,
            expires_at: None,
        }).unwrap();
        
        // Set up test environment
        env::set_var("TEST_API_KEY", test_key);
        
        // Test getting key from env
        let key = get_api_key_from_env("TEST_API_KEY", "backup", &manager).unwrap();
        assert_eq!(key, test_key);
        
        // Clean up
        env::remove_var("TEST_API_KEY");
        
        // Test missing env var
        assert!(matches!(
            get_api_key_from_env("NON_EXISTENT_VAR", "backup", &manager),
            Err(AuthError::MissingApiKey)
        ));
    }
    
    #[test]
    fn test_get_api_key() {
        let manager = ApiKeyManager::new();
        let test_key = "test-arg-key";
        
        // Add a test key
        manager.add_key(ApiKey {
            key: test_key.to_string(),
            service: "llm".to_string(),
            description: None,
            expires_at: None,
        }).unwrap();
        
        // Test getting key from argument
        let key = get_api_key(
            Some(test_key.to_string()),
            "LLM_API_KEY",
            "llm",
            &manager
        ).unwrap();
        assert_eq!(key, test_key);
        
        // Test getting key from env when arg is None
        env::set_var("LLM_API_KEY", test_key);
        let key = get_api_key(None, "LLM_API_KEY", "llm", &manager).unwrap();
        assert_eq!(key, test_key);
        env::remove_var("LLM_API_KEY");
        
        // Test missing key
        assert!(matches!(
            get_api_key(None, "LLM_API_KEY", "llm", &manager),
            Err(AuthError::MissingApiKey)
        ));
    }
}
