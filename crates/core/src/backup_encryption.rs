//! Password-based encryption for backup files.
//!
//! This module provides password-based encryption for backup JSON data using:
//! - Argon2id for key derivation from passwords
//! - ChaCha20-Poly1305 for authenticated encryption
//! - Random salts and nonces for each backup
//!
//! The encrypted backup format is:
//! ```json
//! {
//!   "version": 1,
//!   "encrypted": true,
//!   "salt": "base64-encoded-salt",
//!   "nonce": "base64-encoded-nonce",
//!   "ciphertext": "base64-encoded-encrypted-data"
//! }
//! ```

use anyhow::{anyhow, Context, Result};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

const BACKUP_ENCRYPTION_VERSION: u32 = 1;
const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

/// Encrypted backup container
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedBackup {
    /// Format version for future compatibility
    pub version: u32,
    /// Indicates this is an encrypted backup
    pub encrypted: bool,
    /// Base64-encoded salt for key derivation
    pub salt: String,
    /// Base64-encoded nonce for encryption
    pub nonce: String,
    /// Base64-encoded encrypted backup data
    pub ciphertext: String,
}

/// Encrypts backup JSON data with a password.
///
/// # Arguments
///
/// * `backup_json` - The plaintext backup JSON string
/// * `password` - User-provided password
///
/// # Returns
///
/// A JSON string containing the encrypted backup
///
/// # Errors
///
/// Returns an error if:
/// - Key derivation fails
/// - Encryption fails
/// - Serialization fails
pub fn encrypt_backup(backup_json: &str, password: &str) -> Result<String> {
    // Generate random salt for key derivation
    let mut salt_bytes = vec![0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt_bytes);

    // Derive encryption key from password using Argon2id
    let argon2 = Argon2::default();
    let salt_string = SaltString::encode_b64(&salt_bytes)
        .map_err(|e| anyhow!("Failed to encode salt: {}", e))?;

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| anyhow!("Failed to derive key from password: {}", e))?;

    // Extract the 32-byte key from the hash
    let key_bytes = password_hash
        .hash
        .ok_or_else(|| anyhow!("No hash output from Argon2"))?;

    let key = Zeroizing::new(key_bytes.as_bytes().to_vec());

    if key.len() < 32 {
        return Err(anyhow!("Derived key too short (< 32 bytes)"));
    }

    // Create cipher
    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&key[..32]);
    let cipher = ChaCha20Poly1305::new((&key_array).into());

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the backup data
    let ciphertext = cipher
        .encrypt(nonce, backup_json.as_bytes())
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    // Package into encrypted backup format
    let encrypted_backup = EncryptedBackup {
        version: BACKUP_ENCRYPTION_VERSION,
        encrypted: true,
        salt: BASE64.encode(&salt_bytes),
        nonce: BASE64.encode(&nonce_bytes),
        ciphertext: BASE64.encode(&ciphertext),
    };

    // Serialize to JSON
    serde_json::to_string(&encrypted_backup).context("Failed to serialize encrypted backup")
}

/// Decrypts an encrypted backup with a password.
///
/// # Arguments
///
/// * `encrypted_json` - The encrypted backup JSON string
/// * `password` - User-provided password
///
/// # Returns
///
/// The decrypted plaintext backup JSON
///
/// # Errors
///
/// Returns an error if:
/// - Parsing fails
/// - Password is incorrect
/// - Decryption fails
/// - Format is invalid
pub fn decrypt_backup(encrypted_json: &str, password: &str) -> Result<String> {
    // Parse encrypted backup
    let encrypted: EncryptedBackup =
        serde_json::from_str(encrypted_json).context("Failed to parse encrypted backup")?;

    // Verify version
    if encrypted.version != BACKUP_ENCRYPTION_VERSION {
        return Err(anyhow!(
            "Unsupported backup encryption version: {}",
            encrypted.version
        ));
    }

    if !encrypted.encrypted {
        return Err(anyhow!("Backup is not marked as encrypted"));
    }

    // Decode salt, nonce, and ciphertext
    let salt_bytes = BASE64
        .decode(&encrypted.salt)
        .context("Failed to decode salt")?;

    let nonce_bytes = BASE64
        .decode(&encrypted.nonce)
        .context("Failed to decode nonce")?;

    let ciphertext = BASE64
        .decode(&encrypted.ciphertext)
        .context("Failed to decode ciphertext")?;

    // Derive key from password using the stored salt
    let argon2 = Argon2::default();
    let salt_string = SaltString::encode_b64(&salt_bytes)
        .map_err(|e| anyhow!("Failed to encode salt: {}", e))?;

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| anyhow!("Failed to derive key from password: {}", e))?;

    let key_bytes = password_hash
        .hash
        .ok_or_else(|| anyhow!("No hash output from Argon2"))?;

    let key = Zeroizing::new(key_bytes.as_bytes().to_vec());

    if key.len() < 32 {
        return Err(anyhow!("Derived key too short (< 32 bytes)"));
    }

    // Create cipher
    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&key[..32]);
    let cipher = ChaCha20Poly1305::new((&key_array).into());

    // Decrypt
    let nonce = Nonce::from_slice(&nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| anyhow!("Decryption failed - incorrect password or corrupted data"))?;

    // Convert to string
    String::from_utf8(plaintext).context("Decrypted data is not valid UTF-8")
}

/// Checks if a backup JSON string is encrypted.
///
/// # Arguments
///
/// * `backup_json` - The backup JSON string to check
///
/// # Returns
///
/// `true` if the backup is encrypted, `false` otherwise
pub fn is_encrypted_backup(backup_json: &str) -> bool {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(backup_json) {
        value
            .get("encrypted")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_round_trip() {
        let original_data = r#"{"test": "data", "number": 42}"#;
        let password = "super-secret-password-123";

        // Encrypt
        let encrypted = encrypt_backup(original_data, password).expect("encryption failed");

        // Verify it's encrypted
        assert!(is_encrypted_backup(&encrypted));

        // Decrypt
        let decrypted = decrypt_backup(&encrypted, password).expect("decryption failed");

        // Verify data matches
        assert_eq!(original_data, decrypted);
    }

    #[test]
    fn wrong_password_fails() {
        let original_data = r#"{"test": "data"}"#;
        let password = "correct-password";
        let wrong_password = "wrong-password";

        let encrypted = encrypt_backup(original_data, password).expect("encryption failed");

        // Should fail with wrong password
        let result = decrypt_backup(&encrypted, wrong_password);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Decryption failed"));
    }

    #[test]
    fn is_encrypted_backup_detects_encrypted() {
        let encrypted_json = r#"{"version": 1, "encrypted": true, "salt": "...", "nonce": "...", "ciphertext": "..."}"#;
        assert!(is_encrypted_backup(encrypted_json));
    }

    #[test]
    fn is_encrypted_backup_detects_unencrypted() {
        let unencrypted_json = r#"{"metadata": {}, "protocols": []}"#;
        assert!(!is_encrypted_backup(unencrypted_json));
    }

    #[test]
    fn different_salts_produce_different_ciphertexts() {
        let data = "same data";
        let password = "same password";

        let encrypted1 = encrypt_backup(data, password).expect("encryption 1 failed");
        let encrypted2 = encrypt_backup(data, password).expect("encryption 2 failed");

        // Different salts/nonces should produce different ciphertexts
        assert_ne!(encrypted1, encrypted2);

        // Both should decrypt successfully
        assert_eq!(data, decrypt_backup(&encrypted1, password).unwrap());
        assert_eq!(data, decrypt_backup(&encrypted2, password).unwrap());
    }

    #[test]
    fn empty_password_works() {
        let data = "test data";
        let password = "";

        let encrypted = encrypt_backup(data, password).expect("encryption failed");
        let decrypted = decrypt_backup(&encrypted, password).expect("decryption failed");

        assert_eq!(data, decrypted);
    }

    #[test]
    fn long_data_encryption() {
        let data = "x".repeat(1024 * 100); // 100KB of data
        let password = "test-password";

        let encrypted = encrypt_backup(&data, password).expect("encryption failed");
        let decrypted = decrypt_backup(&encrypted, password).expect("decryption failed");

        assert_eq!(data, decrypted);
    }

    #[test]
    fn unicode_data_encryption() {
        let data = r#"{"message": "Hello 世界 🌍", "emoji": "🔒🔑"}"#;
        let password = "пароль密码";

        let encrypted = encrypt_backup(data, password).expect("encryption failed");
        let decrypted = decrypt_backup(&encrypted, password).expect("decryption failed");

        assert_eq!(data, decrypted);
    }
}
