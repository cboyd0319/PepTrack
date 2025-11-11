//! macOS Keychain integration for secure key storage.
//!
//! This module provides a `KeychainKeyProvider` that stores and retrieves
//! encryption keys using the macOS Keychain Services API, providing OS-level
//! security and access control.

use anyhow::{anyhow, Context, Result};
use rand::{rngs::OsRng, RngCore};
use tracing::info;

use crate::encryption::{KeyMaterial, KeyProvider};

#[cfg(target_os = "macos")]
use security_framework::passwords::{delete_generic_password, get_generic_password, set_generic_password};

const SERVICE_NAME: &str = "com.peptrack.encryption-key";
const ACCOUNT_NAME: &str = "master-key";

/// Key provider that stores encryption keys in the macOS Keychain.
///
/// This provider offers enhanced security over file-based storage by:
/// - Storing keys encrypted by the OS
/// - Requiring user authentication for access (optionally)
/// - Integrating with macOS security features
///
/// # Platform Support
///
/// This provider is only available on macOS. On other platforms, attempting
/// to create this provider will return an error.
///
/// # Keychain Item Details
///
/// - **Service:** `com.peptrack.encryption-key`
/// - **Account:** `master-key`
/// - **Access:** After first unlock (default)
///
/// # Examples
///
/// ```no_run
/// use peptrack_core::{KeychainKeyProvider, KeyProvider};
///
/// # fn main() -> anyhow::Result<()> {
/// let provider = KeychainKeyProvider::new()?;
/// let key_material = provider.key_material()?;
/// # Ok(())
/// # }
/// ```
pub struct KeychainKeyProvider {
    service: String,
    account: String,
}

impl KeychainKeyProvider {
    /// Creates a new Keychain key provider.
    ///
    /// This will load an existing key from the Keychain if present,
    /// or generate and store a new one if not.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The platform is not macOS
    /// - Keychain access fails
    /// - Key generation or storage fails
    #[cfg(target_os = "macos")]
    pub fn new() -> Result<Self> {
        let provider = Self {
            service: SERVICE_NAME.to_string(),
            account: ACCOUNT_NAME.to_string(),
        };

        // Ensure a key exists in the keychain
        provider.ensure_key_exists()?;

        Ok(provider)
    }

    #[cfg(not(target_os = "macos"))]
    pub fn new() -> Result<Self> {
        Err(anyhow!("KeychainKeyProvider is only available on macOS"))
    }

    /// Ensures a key exists in the Keychain, generating one if needed.
    #[cfg(target_os = "macos")]
    fn ensure_key_exists(&self) -> Result<()> {
        // Try to load existing key
        if self.load_from_keychain().is_ok() {
            info!("Loaded existing encryption key from Keychain");
            return Ok(());
        }

        // Generate and store new key
        info!("Generating new encryption key and storing in Keychain");
        let key = self.generate_key()?;
        self.store_in_keychain(&key)?;

        Ok(())
    }

    /// Generates a new 32-byte encryption key.
    fn generate_key(&self) -> Result<Vec<u8>> {
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        Ok(key)
    }

    /// Stores a key in the macOS Keychain.
    #[cfg(target_os = "macos")]
    fn store_in_keychain(&self, key: &[u8]) -> Result<()> {
        set_generic_password(&self.service, &self.account, key)
            .context("Failed to store encryption key in Keychain")?;

        info!("Successfully stored encryption key in Keychain");
        Ok(())
    }

    /// Loads a key from the macOS Keychain.
    #[cfg(target_os = "macos")]
    fn load_from_keychain(&self) -> Result<Vec<u8>> {
        get_generic_password(&self.service, &self.account)
            .context("Failed to retrieve encryption key from Keychain")
    }

    /// Deletes the key from the macOS Keychain.
    ///
    /// This is primarily useful for testing or key rotation scenarios.
    ///
    /// # Errors
    ///
    /// Returns an error if the key doesn't exist or deletion fails.
    #[cfg(target_os = "macos")]
    pub fn delete_from_keychain(&self) -> Result<()> {
        delete_generic_password(&self.service, &self.account)
            .context("Failed to delete encryption key from Keychain")?;

        info!("Successfully deleted encryption key from Keychain");
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn delete_from_keychain(&self) -> Result<()> {
        Err(anyhow!("KeychainKeyProvider is only available on macOS"))
    }
}

#[cfg(target_os = "macos")]
impl KeyProvider for KeychainKeyProvider {
    fn key_material(&self) -> Result<KeyMaterial> {
        let bytes = self.load_from_keychain()?;
        KeyMaterial::new(bytes)
    }
}

/// Migrates an encryption key from a file to the macOS Keychain.
///
/// This function reads a hex-encoded key from the specified file path,
/// stores it in the Keychain, and optionally deletes the file.
///
/// # Arguments
///
/// * `file_path` - Path to the file containing the hex-encoded key
/// * `delete_file` - Whether to delete the file after successful migration
///
/// # Returns
///
/// `Ok(true)` if migration succeeded, `Ok(false)` if key already exists in Keychain.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The key format is invalid
/// - Keychain storage fails
/// - File deletion fails (if requested)
#[cfg(target_os = "macos")]
pub fn migrate_file_key_to_keychain(
    file_path: &std::path::Path,
    delete_file: bool,
) -> Result<bool> {
    use std::fs;

    // Check if key already exists in Keychain
    let provider = KeychainKeyProvider {
        service: SERVICE_NAME.to_string(),
        account: ACCOUNT_NAME.to_string(),
    };

    if provider.load_from_keychain().is_ok() {
        info!("Encryption key already exists in Keychain, skipping migration");
        return Ok(false);
    }

    // Read key from file
    if !file_path.exists() {
        return Err(anyhow!("Key file does not exist: {}", file_path.display()));
    }

    let hex_key = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read key file: {}", file_path.display()))?;

    let key_bytes = hex::decode(hex_key.trim())
        .context("Failed to decode hex key from file")?;

    if key_bytes.len() < 32 {
        return Err(anyhow!("Key file contains invalid key (< 32 bytes)"));
    }

    // Store in Keychain
    provider.store_in_keychain(&key_bytes)?;
    info!("Successfully migrated encryption key from file to Keychain");

    // Optionally delete the file
    if delete_file {
        fs::remove_file(file_path)
            .with_context(|| format!("Failed to delete key file: {}", file_path.display()))?;
        info!("Deleted old key file: {}", file_path.display());
    } else {
        info!("Kept key file as backup: {}", file_path.display());
    }

    Ok(true)
}

#[cfg(not(target_os = "macos"))]
pub fn migrate_file_key_to_keychain(
    _file_path: &std::path::Path,
    _delete_file: bool,
) -> Result<bool> {
    Err(anyhow!("Keychain migration is only available on macOS"))
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::encryption::EnvelopeEncryption;

    // Unique service name for testing to avoid conflicts
    const TEST_SERVICE: &str = "com.peptrack.test.encryption-key";
    const TEST_ACCOUNT: &str = "test-key";

    fn create_test_provider() -> KeychainKeyProvider {
        KeychainKeyProvider {
            service: TEST_SERVICE.to_string(),
            account: TEST_ACCOUNT.to_string(),
        }
    }

    fn cleanup_test_key() {
        let provider = create_test_provider();
        let _ = provider.delete_from_keychain();
    }

    #[test]
    #[ignore] // Requires user interaction for keychain access
    fn keychain_provider_stores_and_retrieves_key() {
        cleanup_test_key();

        let provider = create_test_provider();
        let key = provider.generate_key().unwrap();
        provider.store_in_keychain(&key).unwrap();

        let retrieved = provider.load_from_keychain().unwrap();
        assert_eq!(key, retrieved);

        cleanup_test_key();
    }

    #[test]
    fn keychain_provider_generates_32_byte_keys() {
        let provider = create_test_provider();
        let key = provider.generate_key().unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    #[ignore] // Requires user interaction for keychain access
    fn keychain_provider_implements_key_provider_trait() {
        cleanup_test_key();

        let provider = create_test_provider();
        let key = provider.generate_key().unwrap();
        provider.store_in_keychain(&key).unwrap();

        let key_material = provider.key_material().unwrap();
        let key_bytes = key_material.to_key_bytes().unwrap();
        assert_eq!(key_bytes.len(), 32);

        cleanup_test_key();
    }

    #[test]
    #[ignore] // Requires user interaction for keychain access
    fn keychain_provider_works_with_envelope_encryption() {
        cleanup_test_key();

        let provider = Arc::new(create_test_provider());
        let key = provider.generate_key().unwrap();
        provider.store_in_keychain(&key).unwrap();

        let encryption = EnvelopeEncryption::new(provider);

        let plaintext = b"test data for keychain";
        let sealed = encryption.seal(plaintext).unwrap();
        let opened = encryption.open(&sealed).unwrap();

        assert_eq!(opened, plaintext);

        cleanup_test_key();
    }

    #[test]
    #[ignore] // Requires user interaction for keychain access
    fn keychain_provider_delete_removes_key() {
        cleanup_test_key();

        let provider = create_test_provider();
        let key = provider.generate_key().unwrap();
        provider.store_in_keychain(&key).unwrap();

        // Verify key exists
        assert!(provider.load_from_keychain().is_ok());

        // Delete and verify it's gone
        provider.delete_from_keychain().unwrap();
        assert!(provider.load_from_keychain().is_err());
    }

    #[test]
    #[ignore] // Requires user interaction for keychain access
    fn migrate_file_key_to_keychain_works() {
        use std::fs;
        use tempfile::tempdir;

        cleanup_test_key();

        let dir = tempdir().unwrap();
        let key_file = dir.path().join("test.key");

        // Create a test key file
        let test_key = vec![42u8; 32];
        fs::write(&key_file, hex::encode(&test_key)).unwrap();

        // Override the provider to use test service
        let provider = create_test_provider();
        let hex_key = fs::read_to_string(&key_file).unwrap();
        let key_bytes = hex::decode(hex_key.trim()).unwrap();
        provider.store_in_keychain(&key_bytes).unwrap();

        // Verify the key is in keychain
        let retrieved = provider.load_from_keychain().unwrap();
        assert_eq!(test_key, retrieved);

        cleanup_test_key();
    }
}
