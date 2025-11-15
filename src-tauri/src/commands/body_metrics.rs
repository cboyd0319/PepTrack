use anyhow::Result;
use peptrack_core::models::BodyMetric;
use serde::Deserialize;
use tauri::State;
use time::OffsetDateTime;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyMetricPayload {
    pub date: String, // ISO 8601 string
    pub weight_kg: Option<f32>,
    pub body_fat_percentage: Option<f32>,
    pub muscle_mass_kg: Option<f32>,
    pub waist_cm: Option<f32>,
    pub notes: Option<String>,
}

/// Log a new body metric entry
#[tauri::command]
pub async fn log_body_metric(
    state: State<'_, std::sync::Arc<AppState>>,
    payload: BodyMetricPayload,
) -> Result<BodyMetric, String> {
    // Parse the date string
    let date = OffsetDateTime::parse(&payload.date, &time::format_description::well_known::Rfc3339)
        .map_err(|e| format!("Invalid date format: {}", e))?;

    let mut metric = BodyMetric::new(date);
    metric.weight_kg = payload.weight_kg;
    metric.body_fat_percentage = payload.body_fat_percentage;
    metric.muscle_mass_kg = payload.muscle_mass_kg;
    metric.waist_cm = payload.waist_cm;
    metric.notes = payload.notes;
    metric.updated_at = OffsetDateTime::now_utc();

    state
        .storage
        .upsert_body_metric(&metric)
        .map_err(|err| err.to_string())?;

    Ok(metric)
}

/// List all body metrics
#[tauri::command]
pub async fn list_body_metrics(
    state: State<'_, std::sync::Arc<AppState>>,
) -> Result<Vec<BodyMetric>, String> {
    state
        .storage
        .list_body_metrics()
        .map_err(|err| err.to_string())
}

/// Get a specific body metric by ID
#[tauri::command]
pub async fn get_body_metric(
    state: State<'_, std::sync::Arc<AppState>>,
    metric_id: String,
) -> Result<Option<BodyMetric>, String> {
    state
        .storage
        .get_body_metric(&metric_id)
        .map_err(|err| err.to_string())
}

/// Update an existing body metric
#[tauri::command]
pub async fn update_body_metric(
    state: State<'_, std::sync::Arc<AppState>>,
    metric_id: String,
    payload: BodyMetricPayload,
) -> Result<BodyMetric, String> {
    // Get existing metric
    let mut metric = state
        .storage
        .get_body_metric(&metric_id)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "Body metric not found".to_string())?;

    // Update fields
    if let Ok(date) = OffsetDateTime::parse(&payload.date, &time::format_description::well_known::Rfc3339) {
        metric.date = date;
    }
    metric.weight_kg = payload.weight_kg;
    metric.body_fat_percentage = payload.body_fat_percentage;
    metric.muscle_mass_kg = payload.muscle_mass_kg;
    metric.waist_cm = payload.waist_cm;
    metric.notes = payload.notes;
    metric.updated_at = OffsetDateTime::now_utc();

    state
        .storage
        .upsert_body_metric(&metric)
        .map_err(|err| err.to_string())?;

    Ok(metric)
}

/// Delete a specific body metric
#[tauri::command]
pub async fn delete_body_metric(
    state: State<'_, std::sync::Arc<AppState>>,
    metric_id: String,
) -> Result<(), String> {
    state
        .storage
        .delete_body_metric(&metric_id)
        .map_err(|err| err.to_string())
}

/// Bulk delete multiple body metrics
#[tauri::command]
pub async fn bulk_delete_body_metrics(
    state: State<'_, std::sync::Arc<AppState>>,
    metric_ids: Vec<String>,
) -> Result<usize, String> {
    state
        .storage
        .bulk_delete_body_metrics(&metric_ids)
        .map_err(|err| err.to_string())
}
