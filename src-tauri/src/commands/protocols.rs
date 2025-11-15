use anyhow::Result;
use peptrack_core::models::PeptideProtocol;
use serde::Deserialize;
use tauri::State;
use time::OffsetDateTime;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolPayload {
    pub name: String,
    pub peptide_name: String,
    pub notes: Option<String>,
    pub target_concentration_mg_ml: Option<f32>,
}

#[tauri::command]
pub async fn list_protocols(
    state: State<'_, std::sync::Arc<AppState>>,
) -> Result<Vec<PeptideProtocol>, String> {
    state
        .storage
        .list_protocols()
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn save_protocol(
    state: State<'_, std::sync::Arc<AppState>>,
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

/// Toggle the favorite status of a protocol
#[tauri::command]
pub async fn toggle_protocol_favorite(
    state: State<'_, std::sync::Arc<AppState>>,
    protocol_id: String,
) -> Result<bool, String> {
    state
        .storage
        .toggle_protocol_favorite(&protocol_id)
        .map_err(|err| err.to_string())
}

/// Update tags for a protocol
#[tauri::command]
pub async fn update_protocol_tags(
    state: State<'_, std::sync::Arc<AppState>>,
    protocol_id: String,
    tags: Vec<String>,
) -> Result<Vec<String>, String> {
    state
        .storage
        .update_protocol_tags(&protocol_id, tags)
        .map_err(|err| err.to_string())
}

/// Add a tag to a protocol
#[tauri::command]
pub async fn add_protocol_tag(
    state: State<'_, std::sync::Arc<AppState>>,
    protocol_id: String,
    tag: String,
) -> Result<Vec<String>, String> {
    state
        .storage
        .add_protocol_tag(&protocol_id, tag)
        .map_err(|err| err.to_string())
}

/// Remove a tag from a protocol
#[tauri::command]
pub async fn remove_protocol_tag(
    state: State<'_, std::sync::Arc<AppState>>,
    protocol_id: String,
    tag: String,
) -> Result<Vec<String>, String> {
    state
        .storage
        .remove_protocol_tag(&protocol_id, &tag)
        .map_err(|err| err.to_string())
}
