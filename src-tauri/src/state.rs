use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use dirs::data_dir;
use peptrack_core::{KeyProvider, StaticKeyProvider, StorageConfig, StorageManager};
use peptrack_local_ai::{AiClientConfig, LocalAiOrchestrator};
use rand::rngs::OsRng;
use rand::RngCore;
use tracing::{info, warn};

#[cfg(target_os = "macos")]
use peptrack_core::{migrate_file_key_to_keychain, KeychainKeyProvider};

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<StorageManager>,
    pub ai_client: Arc<LocalAiOrchestrator>,
}

pub fn build_state() -> Result<AppState> {
    let data_dir = resolve_data_dir()?;

    // Attempt to migrate file key to Keychain on macOS (non-blocking)
    #[cfg(target_os = "macos")]
    attempt_keychain_migration(&data_dir);

    // Select key provider: prefer Keychain on macOS, fallback to file-based
    let key_provider: Arc<dyn KeyProvider> = select_key_provider(&data_dir)?;

    let storage = StorageManager::new(StorageConfig {
        data_dir: Some(data_dir),
        db_file_name: None,
        key_provider,
    })?;
    storage.initialize()?;

    let ai_client = LocalAiOrchestrator::detect(AiClientConfig::default());

    Ok(AppState {
        storage: Arc::new(storage),
        ai_client: Arc::new(ai_client),
    })
}

/// Selects the appropriate key provider for the platform.
///
/// On macOS:
/// - Tries KeychainKeyProvider first
/// - Falls back to file-based StaticKeyProvider if Keychain fails
/// - Logs the decision for transparency
///
/// On other platforms:
/// - Always uses file-based StaticKeyProvider
fn select_key_provider(data_dir: &Path) -> Result<Arc<dyn KeyProvider>> {
    #[cfg(target_os = "macos")]
    {
        // Try Keychain first
        match KeychainKeyProvider::new() {
            Ok(provider) => {
                info!("Using macOS Keychain for encryption key storage");
                return Ok(Arc::new(provider));
            }
            Err(err) => {
                warn!("Keychain provider unavailable, falling back to file-based storage: {err:#}");
                // Fall through to file-based provider
            }
        }
    }

    // Fallback: file-based key provider
    let key = ensure_key_material(data_dir)?;
    let provider = StaticKeyProvider::new(key)?;

    #[cfg(target_os = "macos")]
    info!("Using file-based encryption key storage (fallback)");

    #[cfg(not(target_os = "macos"))]
    info!("Using file-based encryption key storage");

    Ok(Arc::new(provider))
}

/// Attempts to migrate the file-based encryption key to macOS Keychain.
///
/// This is best-effort and will not fail the application if it doesn't succeed.
/// The key file is kept as a backup even after successful migration.
#[cfg(target_os = "macos")]
fn attempt_keychain_migration(data_dir: &Path) {
    let key_file = data_dir.join("peptrack.key");

    if !key_file.exists() {
        return;
    }

    match migrate_file_key_to_keychain(&key_file, false) {
        Ok(true) => {
            info!("Successfully migrated encryption key to macOS Keychain");
            info!("File key kept as backup at: {}", key_file.display());
        }
        Ok(false) => {
            info!("Encryption key already exists in Keychain");
        }
        Err(err) => {
            warn!("Failed to migrate key to Keychain (continuing with file): {err:#}");
        }
    }
}

fn resolve_data_dir() -> Result<PathBuf> {
    let mut dir = data_dir().context("Unable to determine OS data directory")?;
    dir.push("PepTrack");
    std::fs::create_dir_all(&dir).context("Unable to create PepTrack data dir")?;
    Ok(dir)
}

fn ensure_key_material(dir: &Path) -> Result<Vec<u8>> {
    let key_path = dir.join("peptrack.key");
    if let Ok(raw) = std::fs::read_to_string(&key_path) {
        let bytes = hex::decode(raw.trim()).context("Stored encryption key is corrupted")?;
        return Ok(bytes);
    }

    let mut bytes = vec![0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    std::fs::write(&key_path, hex::encode(&bytes)).context("Unable to persist encryption key")?;
    Ok(bytes)
}
