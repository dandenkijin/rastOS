//! Encryption module for secure backup storage

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use bytes::{Bytes, BytesMut};
use std::path::Path;

/// Size of the nonce in bytes (96 bits for AES-GCM)
const NONCE_SIZE: usize = 12;

/// Encrypts data using AES-256-GCM
pub fn encrypt_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    // Generate a random nonce
    let nonce = Nonce::from_slice(&{
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        nonce
    });

    // Create cipher instance
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| anyhow!(e))?;

    // Encrypt the data
    let mut ciphertext = cipher.encrypt(nonce, data).map_err(|e| anyhow!(e))?;

    // Prepend the nonce to the ciphertext
    let mut result = nonce.to_vec();
    result.append(&mut ciphertext);

    Ok(result)
}

/// Decrypts data using AES-256-GCM
pub fn decrypt_data(encrypted: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if encrypted.len() < NONCE_SIZE {
        return Err(anyhow!("Encrypted data too short"));
    }

    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Create cipher instance
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| anyhow!(e))?;

    // Decrypt the data
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!(e))?;

    Ok(plaintext)
}

/// Trait for encryption providers
#[async_trait::async_trait]
pub trait EncryptionProvider: Send + Sync + std::fmt::Debug {
    /// Encrypt data
    async fn encrypt(&self, data: Bytes) -> Result<Bytes>;

    /// Decrypt data
    async fn decrypt(&self, data: Bytes) -> Result<Bytes>;
}

/// No-op encryption provider for when encryption is disabled
#[derive(Debug, Clone)]
pub struct NoOpEncryption;

#[async_trait::async_trait]
impl EncryptionProvider for NoOpEncryption {
    async fn encrypt(&self, data: Bytes) -> Result<Bytes> {
        Ok(data)
    }

    async fn decrypt(&self, data: Bytes) -> Result<Bytes> {
        Ok(data)
    }
}

/// AES-256-GCM encryption provider
#[derive(Debug, Clone)]
pub struct AesGcmEncryption {
    key: [u8; 32],
}

impl AesGcmEncryption {
    /// Create a new AES-256-GCM encryption provider
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Generate a new random key
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Load key from file
    pub async fn load_key(path: &Path) -> Result<Self> {
        let key = tokio::fs::read(path).await?;
        if key.len() != 32 {
            return Err(anyhow!("Invalid key length"));
        }
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key);
        Ok(Self { key: key_array })
    }

    /// Save key to file
    pub async fn save_key(&self, path: &Path) -> Result<()> {
        tokio::fs::write(path, &self.key).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl EncryptionProvider for AesGcmEncryption {
    async fn encrypt(&self, data: Bytes) -> Result<Bytes> {
        // For small data, use a single buffer to avoid allocations
        if data.len() < 1024 {
            return Ok(Bytes::from(encrypt_data(&data, &self.key)?));
        }

        // For larger data, process in chunks
        let mut encrypted = BytesMut::new();
        let chunk_size = 64 * 1024; // 64KB chunks
        let mut pos = 0;

        while pos < data.len() {
            let end = std::cmp::min(pos + chunk_size, data.len());
            let chunk = &data[pos..end];
            encrypted.extend_from_slice(&encrypt_data(chunk, &self.key)?);
            pos = end;
        }

        Ok(encrypted.freeze())
    }

    async fn decrypt(&self, data: Bytes) -> Result<Bytes> {
        // For small data, use a single buffer to avoid allocations
        if data.len() < 1024 + NONCE_SIZE {
            return Ok(Bytes::from(decrypt_data(&data, &self.key)?));
        }

        // For larger data, process in chunks
        let mut decrypted = BytesMut::new();
        let chunk_size = 64 * 1024 + NONCE_SIZE; // Account for nonce in each chunk
        let mut pos = 0;

        while pos < data.len() {
            let end = std::cmp::min(pos + chunk_size, data.len());
            let chunk = &data[pos..end];
            decrypted.extend_from_slice(&decrypt_data(chunk, &self.key)?);
            pos = end;
        }

        Ok(decrypted.freeze())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_encryption_decryption() {
        let key = b"0123456789abcdef0123456789abcdef"; // 32 bytes
        let data = b"Hello, world! This is a test.";

        // Encrypt
        let encrypted = encrypt_data(data, key).unwrap();
        assert_ne!(encrypted, data);
        assert!(encrypted.len() > data.len());

        // Decrypt
        let decrypted = decrypt_data(&encrypted, key).unwrap();
        assert_eq!(decrypted, data);
    }

    #[tokio::test]
    async fn test_encryption_provider() {
        let key = AesGcmEncryption::generate_key();
        let provider = AesGcmEncryption::new(key);
        let data = Bytes::from("Test data for encryption");

        // Test encryption/decryption
        let encrypted = provider.encrypt(data.clone()).await.unwrap();
        assert_ne!(encrypted, data);

        let decrypted = provider.decrypt(encrypted).await.unwrap();
        assert_eq!(decrypted, data);
    }
}
