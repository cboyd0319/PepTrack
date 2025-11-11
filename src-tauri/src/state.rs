use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use dirs::data_dir;
use peptrack_core::{StaticKeyProvider, StorageConfig, StorageManager};
use peptrack_local_ai::{AiClientConfig, LocalAiOrchestrator};
use rand::rngs::OsRng;
use rand::RngCore;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<StorageManager<StaticKeyProvider>>,
    pub ai_client: Arc<LocalAiOrchestrator>,
}

pub fn build_state() -> Result<AppState> {
    let data_dir = resolve_data_dir()?;
    let key = ensure_key_material(&data_dir)?;
    let key_provider = Arc::new(StaticKeyProvider::new(key)?);

    let storage = StorageManager::new(StorageConfig {
        data_dir: Some(data_dir),
        db_file_name: None,
        key_provider: key_provider.clone(),
    })?;
    storage.initialize()?;

    let ai_client = LocalAiOrchestrator::detect(AiClientConfig::default());

    Ok(AppState {
        storage: Arc::new(storage),
        ai_client: Arc::new(ai_client),
    })
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
