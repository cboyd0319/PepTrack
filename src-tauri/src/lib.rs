use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use dirs::data_dir;
use hex;
use peptrack_core::{models::PeptideProtocol, StaticKeyProvider, StorageConfig, StorageManager};
use peptrack_local_ai::{
    AiClientConfig, LocalAiClient, LocalAiOrchestrator, SummarizeRequest, SummaryFormat,
};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use time::OffsetDateTime;
use tracing::info;

#[derive(Clone)]
struct AppState {
    storage: Arc<StorageManager<StaticKeyProvider>>,
    ai_client: Arc<LocalAiOrchestrator>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProtocolPayload {
    name: String,
    peptide_name: String,
    notes: Option<String>,
    target_concentration_mg_ml: Option<f32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SummarizePayload {
    title: String,
    content: String,
    format: Option<SummaryFormat>,
}

#[derive(Debug, Serialize)]
struct SummarizeResult {
    provider: String,
    output: String,
}

#[tauri::command]
async fn list_protocols(state: tauri::State<'_, AppState>) -> Result<Vec<PeptideProtocol>, String> {
    state
        .storage
        .list_protocols()
        .map_err(|err| err.to_string())
}

#[tauri::command]
async fn save_protocol(
    state: tauri::State<'_, AppState>,
    payload: ProtocolPayload,
) -> Result<PeptideProtocol, String> {
    let mut protocol = PeptideProtocol::new(payload.name, payload.peptide_name);
    protocol.notes = payload.notes;
    protocol.target_concentration_mg_ml = payload.target_concentration_mg_ml;
    protocol.updated_at = OffsetDateTime::now_utc();

    state
        .storage
        .upsert_protocol(&protocol)
        .map_err(|err| err.to_string())?;

    Ok(protocol)
}

#[tauri::command]
async fn summarize_text(
    state: tauri::State<'_, AppState>,
    payload: SummarizePayload,
) -> Result<SummarizeResult, String> {
    let request = SummarizeRequest {
        title: payload.title,
        content: payload.content,
        format: payload.format.unwrap_or(SummaryFormat::Markdown),
    };

    let response = state
        .ai_client
        .summarize(request)
        .await
        .map_err(|err| err.to_string())?;

    Ok(SummarizeResult {
        provider: format!("{:?}", response.provider),
        output: response.raw_output,
    })
}

fn build_state() -> Result<AppState> {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = build_state().expect("Failed to initialize application state");

    tauri::Builder::default()
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            app.manage(state.clone());
            info!("PepTrack initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_protocols,
            save_protocol,
            summarize_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
