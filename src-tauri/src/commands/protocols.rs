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
pub async fn list_protocols(state: State<'_, AppState>) -> Result<Vec<PeptideProtocol>, String> {
    state
        .storage
        .list_protocols()
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn save_protocol(
    state: State<'_, AppState>,
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
