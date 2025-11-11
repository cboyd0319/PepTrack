use anyhow::Result;
use peptrack_core::models::DoseLog;
use serde::Deserialize;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogDosePayload {
    pub protocol_id: String,
    pub site: String,
    pub amount_mg: f32,
    pub notes: Option<String>,
}

/// Logs a new dose
#[tauri::command]
pub async fn log_dose(
    state: State<'_, AppState>,
    payload: LogDosePayload,
) -> Result<DoseLog, String> {
    let mut log = DoseLog::new(payload.protocol_id, payload.site, payload.amount_mg);
    log.notes = payload.notes;

    state
        .storage
        .append_dose_log(&log)
        .map_err(|err| err.to_string())?;

    Ok(log)
}

/// Lists all dose logs
#[tauri::command]
pub async fn list_dose_logs(state: State<'_, AppState>) -> Result<Vec<DoseLog>, String> {
    state.storage.list_dose_logs().map_err(|err| err.to_string())
}

/// Lists dose logs for a specific protocol
#[tauri::command]
pub async fn list_dose_logs_for_protocol(
    state: State<'_, AppState>,
    protocol_id: String,
) -> Result<Vec<DoseLog>, String> {
    state
        .storage
        .list_dose_logs_for_protocol(&protocol_id)
        .map_err(|err| err.to_string())
}

/// Deletes a specific dose log
#[tauri::command]
pub async fn delete_dose_log(
    state: State<'_, AppState>,
    log_id: String,
) -> Result<(), String> {
    state
        .storage
        .delete_dose_log(&log_id)
        .map_err(|err| err.to_string())
}
