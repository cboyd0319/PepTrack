use anyhow::Result;
use peptrack_core::StorageManager;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use time::{OffsetDateTime, Time};
use tracing::{info, warn};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoseSchedule {
    pub id: String,
    pub protocol_id: String,
    pub protocol_name: String,
    pub peptide_name: String,
    pub amount_mg: f32,
    pub site: Option<String>,
    pub time_of_day: String, // Format: "HH:MM" (24-hour)
    pub days_of_week: Vec<u8>, // 0=Sunday, 1=Monday, ..., 6=Saturday
    pub enabled: bool,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSchedulePayload {
    pub protocol_id: String,
    pub amount_mg: f32,
    pub site: Option<String>,
    pub time_of_day: String,
    pub days_of_week: Vec<u8>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSchedulePayload {
    pub id: String,
    pub amount_mg: Option<f32>,
    pub site: Option<String>,
    pub time_of_day: Option<String>,
    pub days_of_week: Option<Vec<u8>>,
    pub enabled: Option<bool>,
    pub notes: Option<String>,
}

/// Create the schedules table if it doesn't exist
fn ensure_schedules_table(storage: &StorageManager) -> Result<()> {
    let conn = storage.connection();
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS dose_schedules (
            id TEXT PRIMARY KEY,
            protocol_id TEXT NOT NULL,
            amount_mg REAL NOT NULL,
            site TEXT,
            time_of_day TEXT NOT NULL,
            days_of_week TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            notes TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (protocol_id) REFERENCES protocols(id)
        )
        "#,
        [],
    )?;
    Ok(())
}

#[tauri::command]
pub async fn create_dose_schedule(
    state: State<'_, std::sync::Arc<AppState>>,
    payload: CreateSchedulePayload,
) -> Result<DoseSchedule, String> {
    info!("Creating dose schedule for protocol {}", payload.protocol_id);

    ensure_schedules_table(&state.storage).map_err(|e| format!("Database error: {}", e))?;

    // Validate time format
    if !is_valid_time_format(&payload.time_of_day) {
        return Err("Invalid time format. Use HH:MM (24-hour)".to_string());
    }

    // Validate days of week
    if payload.days_of_week.is_empty() || payload.days_of_week.iter().any(|&d| d > 6) {
        return Err("Invalid days of week. Use 0-6 (Sunday-Saturday)".to_string());
    }

    // Get protocol details
    let protocol = state
        .storage
        .get_protocol(&payload.protocol_id)
        .map_err(|e| format!("Failed to get protocol: {}", e))?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = OffsetDateTime::now_utc();
    let now_str = now.unix_timestamp().to_string();
    let days_json = serde_json::to_string(&payload.days_of_week)
        .map_err(|e| format!("Failed to serialize days: {}", e))?;

    let conn = state.storage.connection();
    conn.execute(
        r#"
        INSERT INTO dose_schedules (id, protocol_id, amount_mg, site, time_of_day, days_of_week, enabled, notes, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?9)
        "#,
        rusqlite::params![
            &id,
            &payload.protocol_id,
            payload.amount_mg,
            &payload.site,
            &payload.time_of_day,
            &days_json,
            &payload.notes,
            &now_str,
            &now_str,
        ],
    )
    .map_err(|e| format!("Failed to create schedule: {}", e))?;

    Ok(DoseSchedule {
        id,
        protocol_id: payload.protocol_id,
        protocol_name: protocol.name,
        peptide_name: protocol.peptide_name,
        amount_mg: payload.amount_mg,
        site: payload.site,
        time_of_day: payload.time_of_day,
        days_of_week: payload.days_of_week,
        enabled: true,
        notes: payload.notes,
        created_at: now_str.clone(),
        updated_at: now_str,
    })
}

#[tauri::command]
pub async fn list_dose_schedules(
    state: State<'_, std::sync::Arc<AppState>>,
) -> Result<Vec<DoseSchedule>, String> {
    ensure_schedules_table(&state.storage).map_err(|e| format!("Database error: {}", e))?;

    let conn = state.storage.connection();
    let mut stmt = conn
        .prepare(
            r#"
        SELECT
            ds.id, ds.protocol_id, ds.amount_mg, ds.site, ds.time_of_day,
            ds.days_of_week, ds.enabled, ds.notes, ds.created_at, ds.updated_at,
            p.name as protocol_name, p.peptide_name
        FROM dose_schedules ds
        LEFT JOIN protocols p ON ds.protocol_id = p.id
        ORDER BY ds.time_of_day ASC
        "#,
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let schedules = stmt
        .query_map([], |row| {
            let days_str: String = row.get(5)?;
            let days_of_week: Vec<u8> = serde_json::from_str(&days_str).unwrap_or_default();

            Ok(DoseSchedule {
                id: row.get(0)?,
                protocol_id: row.get(1)?,
                amount_mg: row.get(2)?,
                site: row.get(3)?,
                time_of_day: row.get(4)?,
                days_of_week,
                enabled: row.get::<_, i64>(6)? != 0,
                notes: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                protocol_name: row.get(10)?,
                peptide_name: row.get(11)?,
            })
        })
        .map_err(|e| format!("Failed to query schedules: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect schedules: {}", e))?;

    Ok(schedules)
}

#[tauri::command]
pub async fn update_dose_schedule(
    state: State<'_, std::sync::Arc<AppState>>,
    payload: UpdateSchedulePayload,
) -> Result<DoseSchedule, String> {
    info!("Updating dose schedule {}", payload.id);

    ensure_schedules_table(&state.storage).map_err(|e| format!("Database error: {}", e))?;

    // Validate time if provided
    if let Some(ref time) = payload.time_of_day {
        if !is_valid_time_format(time) {
            return Err("Invalid time format. Use HH:MM (24-hour)".to_string());
        }
    }

    // Validate days if provided
    if let Some(ref days) = payload.days_of_week {
        if days.is_empty() || days.iter().any(|&d| d > 6) {
            return Err("Invalid days of week. Use 0-6 (Sunday-Saturday)".to_string());
        }
    }

    let conn = state.storage.connection();
    let now = OffsetDateTime::now_utc().unix_timestamp().to_string();

    // Build dynamic SQL based on what's being updated
    let mut updates = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(amount) = payload.amount_mg {
        updates.push("amount_mg = ?");
        params.push(Box::new(amount));
    }
    if let Some(site) = payload.site {
        updates.push("site = ?");
        params.push(Box::new(site));
    }
    if let Some(time) = payload.time_of_day {
        updates.push("time_of_day = ?");
        params.push(Box::new(time));
    }
    if let Some(days) = payload.days_of_week {
        let days_json = serde_json::to_string(&days).unwrap();
        updates.push("days_of_week = ?");
        params.push(Box::new(days_json));
    }
    if let Some(enabled) = payload.enabled {
        updates.push("enabled = ?");
        params.push(Box::new(if enabled { 1 } else { 0 }));
    }
    if let Some(notes) = payload.notes {
        updates.push("notes = ?");
        params.push(Box::new(notes));
    }

    updates.push("updated_at = ?");
    params.push(Box::new(now));
    params.push(Box::new(payload.id.clone()));

    if updates.len() > 1 {
        // has updates beyond updated_at
        let sql = format!("UPDATE dose_schedules SET {} WHERE id = ?", updates.join(", "));
        conn.execute(&sql, rusqlite::params_from_iter(params))
            .map_err(|e| format!("Failed to update schedule: {}", e))?;
    }

    // Fetch and return updated schedule
    list_dose_schedules(state)
        .await?
        .into_iter()
        .find(|s| s.id == payload.id)
        .ok_or_else(|| "Schedule not found after update".to_string())
}

#[tauri::command]
pub async fn delete_dose_schedule(
    state: State<'_, std::sync::Arc<AppState>>,
    schedule_id: String,
) -> Result<(), String> {
    info!("Deleting dose schedule {}", schedule_id);

    ensure_schedules_table(&state.storage).map_err(|e| format!("Database error: {}", e))?;

    let conn = state.storage.connection();
    conn.execute("DELETE FROM dose_schedules WHERE id = ?1", [&schedule_id])
        .map_err(|e| format!("Failed to delete schedule: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_pending_dose_reminders(
    state: State<'_, std::sync::Arc<AppState>>,
    _app: AppHandle,
) -> Result<Vec<DoseSchedule>, String> {
    ensure_schedules_table(&state.storage).map_err(|e| format!("Database error: {}", e))?;

    let schedules = list_dose_schedules(state).await?;
    let now = OffsetDateTime::now_utc();
    let current_time = now.time();
    let current_weekday = now.weekday().number_days_from_sunday(); // 0-6

    // Filter schedules that should trigger now
    let pending: Vec<DoseSchedule> = schedules
        .into_iter()
        .filter(|s| {
            if !s.enabled {
                return false;
            }

            // Check if today is a scheduled day
            if !s.days_of_week.contains(&current_weekday) {
                return false;
            }

            // Parse schedule time
            if let Some(schedule_time) = parse_time(&s.time_of_day) {
                // Within 15 minute window
                let diff_minutes = time_diff_minutes(current_time, schedule_time);
                diff_minutes >= 0 && diff_minutes <= 15
            } else {
                false
            }
        })
        .collect();

    Ok(pending)
}

fn is_valid_time_format(time_str: &str) -> bool {
    time_str.len() == 5 && time_str.chars().nth(2) == Some(':')
}

fn parse_time(time_str: &str) -> Option<Time> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let hour: u8 = parts[0].parse().ok()?;
    let minute: u8 = parts[1].parse().ok()?;

    Time::from_hms(hour, minute, 0).ok()
}

fn time_diff_minutes(current: Time, target: Time) -> i32 {
    let current_minutes = current.hour() as i32 * 60 + current.minute() as i32;
    let target_minutes = target.hour() as i32 * 60 + target.minute() as i32;
    target_minutes - current_minutes
}
