use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand::{rngs::OsRng, RngCore};
use zeroize::Zeroizing;

/// Provides encryption key material for the application.
///
/// Implementations must be thread-safe (`Send + Sync`) and provide
/// 32-byte keys suitable for ChaCha20-Poly1305 encryption.
pub trait KeyProvider: Send + Sync {
    /// Retrieves the current encryption key material.
    ///
    /// # Errors
    ///
    /// Returns an error if the key cannot be retrieved or is invalid.
    fn key_material(&self) -> Result<KeyMaterial>;
}

/// Securely holds encryption key bytes with automatic zeroing on drop.
///
/// This type wraps key material in a `Zeroizing` container to ensure
/// sensitive data is cleared from memory when no longer needed.
#[derive(Debug)]
pub struct KeyMaterial {
    bytes: Zeroizing<Vec<u8>>,
}

impl Clone for KeyMaterial {
    fn clone(&self) -> Self {
        Self {
            bytes: Zeroizing::new(self.bytes.to_vec()),
        }
    }
}

impl KeyMaterial {
    /// Creates new key material from raw bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Raw key bytes (must be at least 32 bytes)
    ///
    /// # Errors
    ///
    /// Returns an error if the provided bytes are less than 32 bytes in length.
    pub fn new(bytes: Vec<u8>) -> Result<Self> {
        if bytes.len() < 32 {
            return Err(anyhow!("Key material must be at least 32 bytes"));
        }
        Ok(Self {
            bytes: Zeroizing::new(bytes),
        })
    }

    /// Extracts exactly 32 bytes for use as a ChaCha20-Poly1305 key.
    ///
    /// # Errors
    ///
    /// This function is infallible in practice since construction validates length.
    pub fn to_key_bytes(&self) -> Result<[u8; 32]> {
        let mut key = [0u8; 32];
        key.copy_from_slice(&self.bytes[..32]);
        Ok(key)
    }
}

/// ChaCha20-Poly1305 envelope encryption with per-record nonces.
///
/// This type provides authenticated encryption for sensitive data before
/// storage. Each encryption operation uses a randomly generated 12-byte nonce,
/// which is prepended to the ciphertext.
///
/// # Security Properties
///
/// - **Confidentiality**: ChaCha20 stream cipher prevents unauthorized reading
/// - **Authenticity**: Poly1305 MAC prevents tampering
/// - **Unique Nonces**: Each `seal()` call generates a fresh nonce from `OsRng`
/// - **Key Zeroization**: Key material is automatically cleared from memory
///
/// # Wire Format
///
/// ```text
/// [12-byte nonce][ciphertext + 16-byte auth tag]
/// ```
pub struct EnvelopeEncryption<P: KeyProvider + 'static> {
    key_provider: Arc<P>,
}

impl<P: KeyProvider + 'static> EnvelopeEncryption<P> {
    /// Creates a new envelope encryption instance with the given key provider.
    pub fn new(key_provider: Arc<P>) -> Self {
        Self { key_provider }
    }

    /// Encrypts plaintext and returns `[nonce || ciphertext]`.
    ///
    /// # Arguments
    ///
    /// * `plaintext` - Data to encrypt
    ///
    /// # Returns
    ///
    /// A vector containing the 12-byte nonce followed by the authenticated ciphertext.
    ///
    /// # Errors
    ///
    /// Returns an error if key retrieval or encryption fails.
    pub fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let key_bytes = self.key_provider.key_material()?.to_key_bytes()?;
        let key = Key::from(key_bytes);
        let cipher = ChaCha20Poly1305::new(&key);
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);
        let mut ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("encryption failed: {e}"))?;

        let mut output = nonce_bytes.to_vec();
        output.append(&mut ciphertext);
        Ok(output)
    }

    /// Decrypts a payload created by `seal()`.
    ///
    /// # Arguments
    ///
    /// * `payload` - The `[nonce || ciphertext]` vector from `seal()`
    ///
    /// # Returns
    ///
    /// The original plaintext if authentication succeeds.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The payload is too short (< 13 bytes)
    /// - Key retrieval fails
    /// - Authentication/decryption fails (tampering detected)
    pub fn open(&self, payload: &[u8]) -> Result<Vec<u8>> {
        if payload.len() < 13 {
            return Err(anyhow!("ciphertext too short"));
        }

        let (nonce_bytes, ciphertext) = payload.split_at(12);
        let key_bytes = self.key_provider.key_material()?.to_key_bytes()?;
        let key = Key::from(key_bytes);
        let cipher = ChaCha20Poly1305::new(&key);
        let mut nonce_arr = [0u8; 12];
        nonce_arr.copy_from_slice(nonce_bytes);
        let nonce = Nonce::from(nonce_arr);

        cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("decryption failed: {e}"))
    }
}

/// Key provider that reads hex-encoded keys from environment variables.
///
/// Useful for server deployments where keys are injected via environment.
pub struct EnvKeyProvider {
    env_key: String,
}

impl EnvKeyProvider {
    /// Creates a provider that reads from the specified environment variable.
    ///
    /// # Arguments
    ///
    /// * `var` - Name of the environment variable containing hex-encoded key bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the environment variable is not set.
    pub fn from_env(var: &str) -> Result<Self> {
        let value = std::env::var(var)
            .map_err(|_| anyhow!("Environment variable {var} missing for encryption key"))?;
        Ok(Self { env_key: value })
    }
}

impl KeyProvider for EnvKeyProvider {
    fn key_material(&self) -> Result<KeyMaterial> {
        KeyMaterial::new(hex::decode(&self.env_key).context("Invalid hex in env key")?)
    }
}

/// Key provider that holds a static key in memory.
///
/// This is the simplest provider and is suitable for desktop applications
/// where the key is loaded from disk at startup.
///
/// # Security Note
///
/// The key is zeroized on drop, but remains in memory for the lifetime
/// of this provider instance.
pub struct StaticKeyProvider {
    key: KeyMaterial,
}

impl StaticKeyProvider {
    /// Creates a new static key provider from raw bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Raw key bytes (must be at least 32 bytes)
    ///
    /// # Errors
    ///
    /// Returns an error if the key material is invalid (< 32 bytes).
    pub fn new(bytes: Vec<u8>) -> Result<Self> {
        Ok(Self {
            key: KeyMaterial::new(bytes)?,
        })
    }
}

impl KeyProvider for StaticKeyProvider {
    fn key_material(&self) -> Result<KeyMaterial> {
        Ok(self.key.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_material_rejects_short_keys() {
        let result = KeyMaterial::new(vec![1u8; 16]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least 32 bytes"));
    }

    #[test]
    fn key_material_accepts_exactly_32_bytes() {
        let result = KeyMaterial::new(vec![42u8; 32]);
        assert!(result.is_ok());
    }

    #[test]
    fn key_material_accepts_more_than_32_bytes() {
        let result = KeyMaterial::new(vec![42u8; 64]);
        assert!(result.is_ok());
        let key_bytes = result.unwrap().to_key_bytes().unwrap();
        assert_eq!(key_bytes.len(), 32);
    }

    #[test]
    fn envelope_encryption_round_trip_empty_plaintext() {
        let provider = Arc::new(StaticKeyProvider::new(vec![7u8; 32]).unwrap());
        let encryption = EnvelopeEncryption::new(provider);

        let plaintext = b"";
        let sealed = encryption.seal(plaintext).unwrap();
        let opened = encryption.open(&sealed).unwrap();

        assert_eq!(opened, plaintext);
    }

    #[test]
    fn envelope_encryption_round_trip_large_plaintext() {
        let provider = Arc::new(StaticKeyProvider::new(vec![7u8; 32]).unwrap());
        let encryption = EnvelopeEncryption::new(provider);

        let plaintext = vec![42u8; 1_000_000]; // 1MB
        let sealed = encryption.seal(&plaintext).unwrap();
        let opened = encryption.open(&sealed).unwrap();

        assert_eq!(opened, plaintext);
    }

    #[test]
    fn envelope_encryption_detects_tampering() {
        let provider = Arc::new(StaticKeyProvider::new(vec![7u8; 32]).unwrap());
        let encryption = EnvelopeEncryption::new(provider);

        let plaintext = b"sensitive data";
        let mut sealed = encryption.seal(plaintext).unwrap();

        // Tamper with the ciphertext
        if let Some(byte) = sealed.last_mut() {
            *byte ^= 0xFF;
        }

        let result = encryption.open(&sealed);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("decryption failed"));
    }

    #[test]
    fn envelope_encryption_rejects_short_payload() {
        let provider = Arc::new(StaticKeyProvider::new(vec![7u8; 32]).unwrap());
        let encryption = EnvelopeEncryption::new(provider);

        let short_payload = vec![0u8; 12]; // Only nonce, no ciphertext
        let result = encryption.open(&short_payload);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too short"));
    }

    #[test]
    fn envelope_encryption_different_keys_fail_to_decrypt() {
        let provider1 = Arc::new(StaticKeyProvider::new(vec![1u8; 32]).unwrap());
        let provider2 = Arc::new(StaticKeyProvider::new(vec![2u8; 32]).unwrap());

        let encryption1 = EnvelopeEncryption::new(provider1);
        let encryption2 = EnvelopeEncryption::new(provider2);

        let plaintext = b"secret message";
        let sealed = encryption1.seal(plaintext).unwrap();

        // Try to decrypt with wrong key
        let result = encryption2.open(&sealed);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("decryption failed"));
    }

    #[test]
    fn envelope_encryption_unique_nonces_per_call() {
        let provider = Arc::new(StaticKeyProvider::new(vec![42u8; 32]).unwrap());
        let encryption = EnvelopeEncryption::new(provider);

        let plaintext = b"same message";
        let sealed1 = encryption.seal(plaintext).unwrap();
        let sealed2 = encryption.seal(plaintext).unwrap();

        // Even with same plaintext, sealed versions should differ (different nonces)
        assert_ne!(sealed1, sealed2);

        // But both should decrypt correctly
        assert_eq!(encryption.open(&sealed1).unwrap(), plaintext);
        assert_eq!(encryption.open(&sealed2).unwrap(), plaintext);
    }

    #[test]
    fn static_key_provider_clones_key_material() {
        let provider = StaticKeyProvider::new(vec![99u8; 32]).unwrap();
        let key1 = provider.key_material().unwrap();
        let key2 = provider.key_material().unwrap();

        // Should produce equivalent keys
        assert_eq!(key1.to_key_bytes().unwrap(), key2.to_key_bytes().unwrap());
    }
}
