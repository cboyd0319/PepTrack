use anyhow::Result;
use peptrack_core::models::SideEffect;
use serde::Deserialize;
use tauri::State;
use time::OffsetDateTime;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SideEffectPayload {
    pub protocol_id: Option<String>,
    pub dose_log_id: Option<String>,
    pub date: String, // ISO 8601 string
    pub severity: String,
    pub symptom: String,
    pub description: Option<String>,
    pub duration_minutes: Option<i32>,
    pub resolved: Option<bool>,
}

/// Log a new side effect entry
#[tauri::command]
pub async fn log_side_effect(
    state: State<'_, std::sync::Arc<AppState>>,
    payload: SideEffectPayload,
) -> Result<SideEffect, String> {
    // Parse the date string
    let date = OffsetDateTime::parse(&payload.date, &time::format_description::well_known::Rfc3339)
        .map_err(|e| format!("Invalid date format: {}", e))?;

    let mut effect = SideEffect::new(date, &payload.severity, &payload.symptom);
    effect.protocol_id = payload.protocol_id;
    effect.dose_log_id = payload.dose_log_id;
    effect.description = payload.description;
    effect.duration_minutes = payload.duration_minutes;
    effect.resolved = payload.resolved.unwrap_or(false);
    effect.updated_at = OffsetDateTime::now_utc();

    state
        .storage
        .upsert_side_effect(&effect)
        .map_err(|err| err.to_string())?;

    Ok(effect)
}

/// List all side effects
#[tauri::command]
pub async fn list_side_effects(
    state: State<'_, std::sync::Arc<AppState>>,
) -> Result<Vec<SideEffect>, String> {
    state
        .storage
        .list_side_effects()
        .map_err(|err| err.to_string())
}

/// Get a specific side effect by ID
#[tauri::command]
pub async fn get_side_effect(
    state: State<'_, std::sync::Arc<AppState>>,
    effect_id: String,
) -> Result<Option<SideEffect>, String> {
    state
        .storage
        .get_side_effect(&effect_id)
        .map_err(|err| err.to_string())
}

/// List side effects for a specific protocol
#[tauri::command]
pub async fn list_side_effects_by_protocol(
    state: State<'_, std::sync::Arc<AppState>>,
    protocol_id: String,
) -> Result<Vec<SideEffect>, String> {
    state
        .storage
        .list_side_effects_by_protocol(&protocol_id)
        .map_err(|err| err.to_string())
}

/// Update an existing side effect
#[tauri::command]
pub async fn update_side_effect(
    state: State<'_, std::sync::Arc<AppState>>,
    effect_id: String,
    payload: SideEffectPayload,
) -> Result<SideEffect, String> {
    // Get existing effect
    let mut effect = state
        .storage
        .get_side_effect(&effect_id)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "Side effect not found".to_string())?;

    // Update fields
    if let Ok(date) =
        OffsetDateTime::parse(&payload.date, &time::format_description::well_known::Rfc3339)
    {
        effect.date = date;
    }
    effect.protocol_id = payload.protocol_id;
    effect.dose_log_id = payload.dose_log_id;
    effect.severity = payload.severity;
    effect.symptom = payload.symptom;
    effect.description = payload.description;
    effect.duration_minutes = payload.duration_minutes;
    if let Some(resolved) = payload.resolved {
        effect.resolved = resolved;
    }
    effect.updated_at = OffsetDateTime::now_utc();

    state
        .storage
        .upsert_side_effect(&effect)
        .map_err(|err| err.to_string())?;

    Ok(effect)
}

/// Toggle the resolved status of a side effect
#[tauri::command]
pub async fn toggle_side_effect_resolved(
    state: State<'_, std::sync::Arc<AppState>>,
    effect_id: String,
    resolved: bool,
) -> Result<(), String> {
    state
        .storage
        .update_side_effect_resolved(&effect_id, resolved)
        .map_err(|err| err.to_string())
}

/// Delete a specific side effect
#[tauri::command]
pub async fn delete_side_effect(
    state: State<'_, std::sync::Arc<AppState>>,
    effect_id: String,
) -> Result<(), String> {
    state
        .storage
        .delete_side_effect(&effect_id)
        .map_err(|err| err.to_string())
}

/// Bulk delete multiple side effects
#[tauri::command]
pub async fn bulk_delete_side_effects(
    state: State<'_, std::sync::Arc<AppState>>,
    effect_ids: Vec<String>,
) -> Result<usize, String> {
    state
        .storage
        .bulk_delete_side_effects(&effect_ids)
        .map_err(|err| err.to_string())
}
