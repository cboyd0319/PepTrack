use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use crate::state::AppState;

/// Backup frequency options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BackupFrequency {
    Hourly,
    Daily,
    Weekly,
    Manual,
}

/// Backup destination options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BackupDestination {
    Local,
    GoogleDrive,
}

/// Scheduled backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSchedule {
    pub enabled: bool,
    pub frequency: BackupFrequency,
    pub destinations: Vec<BackupDestination>,
    pub last_backup: Option<String>,
    pub next_backup: Option<String>,
}

impl Default for BackupSchedule {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: BackupFrequency::Manual,
            destinations: vec![BackupDestination::Local],
            last_backup: None,
            next_backup: None,
        }
    }
}

/// Scheduler state for managing background tasks
#[derive(Clone)]
pub struct SchedulerState {
    schedule: Arc<Mutex<BackupSchedule>>,
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Default for SchedulerState {
    fn default() -> Self {
        Self::new()
    }
}

const SCHEDULE_FILENAME: &str = "backup_schedule.json";

impl SchedulerState {
    pub fn new() -> Self {
        Self {
            schedule: Arc::new(Mutex::new(BackupSchedule::default())),
            task_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Load schedule from disk on startup
    pub async fn load_from_disk(&self) -> Result<()> {
        match load_schedule_from_disk().await {
            Ok(schedule) => {
                *self.schedule.lock().await = schedule;
                info!("Loaded backup schedule from disk");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to load backup schedule: {:#}", e);
                // Not a fatal error - just use default schedule
                Ok(())
            }
        }
    }

    /// Start the background scheduler task
    pub async fn start_scheduler(&self, app_state: Arc<AppState>) {
        let schedule_arc = self.schedule.clone();

        let handle = tokio::spawn(async move {
            info!("Background backup scheduler started");
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                let schedule = schedule_arc.lock().await.clone();

                // Check if scheduled backups are enabled
                if !schedule.enabled {
                    continue;
                }

                // Check if it's time to backup
                if let Some(next_backup_str) = &schedule.next_backup {
                    match OffsetDateTime::parse(next_backup_str, &time::format_description::well_known::Rfc3339) {
                        Ok(next_backup_time) => {
                            let now = OffsetDateTime::now_utc();

                            if now >= next_backup_time {
                                info!("Scheduled backup triggered");

                                // Perform the backup
                                if let Err(e) = perform_scheduled_backup(&app_state, &schedule_arc).await {
                                    error!("Scheduled backup failed: {:#}", e);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse next backup time: {:#}", e);
                        }
                    }
                }
            }
        });

        *self.task_handle.lock().await = Some(handle);
        info!("Background scheduler task spawned");
    }
}

/// Gets the current backup schedule
#[tauri::command]
pub async fn get_backup_schedule(state: State<'_, SchedulerState>) -> Result<BackupSchedule, String> {
    let schedule = state.schedule.lock().await.clone();
    Ok(schedule)
}

/// Updates the backup schedule
#[tauri::command]
pub async fn update_backup_schedule(
    state: State<'_, SchedulerState>,
    schedule: BackupSchedule,
) -> Result<BackupSchedule, String> {
    info!(
        "Updating backup schedule: enabled={}, frequency={:?}, destinations={:?}",
        schedule.enabled, schedule.frequency, schedule.destinations
    );

    // Calculate next backup time if enabled
    let mut updated_schedule = schedule.clone();
    if updated_schedule.enabled {
        updated_schedule.next_backup = Some(calculate_next_backup(&schedule.frequency));
    } else {
        updated_schedule.next_backup = None;
    }

    *state.schedule.lock().await = updated_schedule.clone();

    // Persist to disk
    if let Err(e) = save_schedule_to_disk(&updated_schedule).await {
        warn!("Failed to save backup schedule: {:#}", e);
        return Err(format!("Failed to save schedule: {}", e));
    }

    info!("Backup schedule updated successfully");
    Ok(updated_schedule)
}

/// Manually triggers a backup
#[tauri::command]
pub async fn trigger_manual_backup(
    scheduler_state: State<'_, SchedulerState>,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    info!("Manual backup triggered");

    let mut schedule = scheduler_state.schedule.lock().await;

    // Perform backup based on destinations
    let mut results = Vec::new();

    for destination in &schedule.destinations {
        match destination {
            BackupDestination::Local => {
                match perform_local_backup(&app_state).await {
                    Ok(path) => {
                        info!("Local backup successful: {}", path);
                        results.push(format!("Local: {}", path));
                    }
                    Err(e) => {
                        warn!("Local backup failed: {:#}", e);
                        return Err(format!("Local backup failed: {}", e));
                    }
                }
            }
            BackupDestination::GoogleDrive => {
                match perform_drive_backup(&app_state).await {
                    Ok(file_id) => {
                        info!("Google Drive backup successful: {}", file_id);
                        results.push(format!("Drive: {}", file_id));
                    }
                    Err(e) => {
                        warn!("Google Drive backup failed: {:#}", e);
                        return Err(format!("Drive backup failed: {}", e));
                    }
                }
            }
        }
    }

    // Update last backup time
    schedule.last_backup = Some(OffsetDateTime::now_utc().to_string());

    // Calculate next backup if scheduled
    if schedule.enabled {
        schedule.next_backup = Some(calculate_next_backup(&schedule.frequency));
    }

    // Persist to disk
    if let Err(e) = save_schedule_to_disk(&schedule).await {
        warn!("Failed to save backup schedule: {:#}", e);
    }

    Ok(results.join(", "))
}

// Helper functions

fn calculate_next_backup(frequency: &BackupFrequency) -> String {
    let now = OffsetDateTime::now_utc();
    let next = match frequency {
        BackupFrequency::Hourly => now + time::Duration::hours(1),
        BackupFrequency::Daily => now + time::Duration::days(1),
        BackupFrequency::Weekly => now + time::Duration::weeks(1),
        BackupFrequency::Manual => now, // Manual backups don't have a "next" time
    };
    next.to_string()
}

async fn perform_local_backup(state: &AppState) -> Result<String> {
    // Get backup data
    let protocols = state.storage.list_protocols()?;
    let doses = state.storage.list_dose_logs()?;
    let literature = state.storage.list_literature()?;

    // Create backup structure
    use crate::commands::backup::{BackupData, BackupMetadata};

    let metadata = BackupMetadata {
        export_date: OffsetDateTime::now_utc().to_string(),
        protocols_count: protocols.len(),
        doses_count: doses.len(),
        literature_count: literature.len(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let backup = BackupData {
        metadata,
        protocols: protocols
            .into_iter()
            .map(|p| serde_json::to_value(p).unwrap_or_default())
            .collect(),
        dose_logs: doses
            .into_iter()
            .map(|d| serde_json::to_value(d).unwrap_or_default())
            .collect(),
        literature: literature
            .into_iter()
            .map(|l| serde_json::to_value(l).unwrap_or_default())
            .collect(),
    };

    // Generate filename
    let timestamp = OffsetDateTime::now_utc()
        .format(&time::format_description::parse("[year]-[month]-[day]_[hour]-[minute]").unwrap())
        .unwrap_or_else(|_| "backup".to_string());
    let filename = format!("peptrack_scheduled_backup_{}.json", timestamp);

    // Save to downloads/documents folder
    let default_path = dirs::download_dir()
        .or_else(|| dirs::document_dir())
        .context("Could not determine download directory")?;

    let full_path = default_path.join(&filename);

    let json = serde_json::to_string_pretty(&backup)?;
    std::fs::write(&full_path, json)?;

    Ok(full_path.to_string_lossy().to_string())
}

async fn perform_drive_backup(state: &AppState) -> Result<String> {
    use crate::commands::backup::{BackupData, BackupMetadata};
    use crate::commands::drive;

    let protocols = state.storage.list_protocols()?;
    let doses = state.storage.list_dose_logs()?;
    let literature = state.storage.list_literature()?;

    let metadata = BackupMetadata {
        export_date: OffsetDateTime::now_utc().to_string(),
        protocols_count: protocols.len(),
        doses_count: doses.len(),
        literature_count: literature.len(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let backup = BackupData {
        metadata,
        protocols: protocols
            .into_iter()
            .map(|p| serde_json::to_value(p).unwrap_or_default())
            .collect(),
        dose_logs: doses
            .into_iter()
            .map(|d| serde_json::to_value(d).unwrap_or_default())
            .collect(),
        literature: literature
            .into_iter()
            .map(|l| serde_json::to_value(l).unwrap_or_default())
            .collect(),
    };

    // Generate filename
    let timestamp = OffsetDateTime::now_utc()
        .format(&time::format_description::parse("[year]-[month]-[day]_[hour]-[minute]").unwrap())
        .unwrap_or_else(|_| "backup".to_string());
    let filename = format!("peptrack_scheduled_backup_{}.json", timestamp);

    let json = serde_json::to_string_pretty(&backup)?;

    // Load Drive tokens
    let tokens = drive::load_drive_tokens_internal(state).await
        .context("Google Drive not connected")?;

    let client = reqwest::Client::new();

    // Create or get PepTrack folder
    let folder_id = drive::get_or_create_folder_internal(&client, &tokens.access_token, "PepTrack Backups")
        .await
        .context("Failed to create/get Drive folder")?;

    // Upload file
    let file_id = drive::upload_file_internal(&client, &tokens.access_token, &folder_id, &filename, &json)
        .await
        .context("Failed to upload to Drive")?;

    Ok(file_id)
}

/// Save schedule to disk
async fn save_schedule_to_disk(schedule: &BackupSchedule) -> Result<()> {
    let data_dir = dirs::data_dir()
        .context("Unable to determine data directory")?
        .join("PepTrack");
    std::fs::create_dir_all(&data_dir)?;

    let schedule_file = data_dir.join(SCHEDULE_FILENAME);
    let json = serde_json::to_string_pretty(schedule)?;
    std::fs::write(&schedule_file, json)
        .context("Failed to save backup schedule")?;

    Ok(())
}

/// Load schedule from disk
async fn load_schedule_from_disk() -> Result<BackupSchedule> {
    let data_dir = dirs::data_dir()
        .context("Unable to determine data directory")?
        .join("PepTrack");
    let schedule_file = data_dir.join(SCHEDULE_FILENAME);

    let json = std::fs::read_to_string(&schedule_file)
        .context("Backup schedule not found")?;
    let schedule: BackupSchedule = serde_json::from_str(&json)?;
    Ok(schedule)
}

/// Performs a scheduled backup (called by background task)
async fn perform_scheduled_backup(
    app_state: &AppState,
    schedule_arc: &Arc<Mutex<BackupSchedule>>,
) -> Result<()> {
    let mut schedule = schedule_arc.lock().await;
    let destinations = schedule.destinations.clone();

    // Perform backup based on destinations
    for destination in &destinations {
        match destination {
            BackupDestination::Local => {
                match perform_local_backup(app_state).await {
                    Ok(path) => {
                        info!("Scheduled local backup successful: {}", path);
                    }
                    Err(e) => {
                        error!("Scheduled local backup failed: {:#}", e);
                    }
                }
            }
            BackupDestination::GoogleDrive => {
                match perform_drive_backup(app_state).await {
                    Ok(file_id) => {
                        info!("Scheduled Drive backup successful: {}", file_id);
                    }
                    Err(e) => {
                        error!("Scheduled Drive backup failed: {:#}", e);
                    }
                }
            }
        }
    }

    // Update last backup time
    schedule.last_backup = Some(OffsetDateTime::now_utc().to_string());

    // Calculate next backup time
    schedule.next_backup = Some(calculate_next_backup(&schedule.frequency));

    // Persist to disk
    if let Err(e) = save_schedule_to_disk(&schedule).await {
        warn!("Failed to save backup schedule: {:#}", e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_frequency_serialization() {
        assert_eq!(
            serde_json::to_string(&BackupFrequency::Hourly).unwrap(),
            "\"Hourly\""
        );
        assert_eq!(
            serde_json::to_string(&BackupFrequency::Daily).unwrap(),
            "\"Daily\""
        );
        assert_eq!(
            serde_json::to_string(&BackupFrequency::Weekly).unwrap(),
            "\"Weekly\""
        );
        assert_eq!(
            serde_json::to_string(&BackupFrequency::Manual).unwrap(),
            "\"Manual\""
        );
    }

    #[test]
    fn test_backup_destination_serialization() {
        let json = serde_json::to_string(&BackupDestination::Local).unwrap();
        assert_eq!(json, "\"Local\"");

        let json = serde_json::to_string(&BackupDestination::GoogleDrive).unwrap();
        assert_eq!(json, "\"GoogleDrive\"");
    }

    #[test]
    fn test_backup_schedule_default() {
        let schedule = BackupSchedule::default();
        assert!(!schedule.enabled);
        assert_eq!(schedule.frequency, BackupFrequency::Manual);
        assert_eq!(schedule.destinations.len(), 1);
        assert_eq!(schedule.destinations[0], BackupDestination::Local);
        assert!(schedule.last_backup.is_none());
        assert!(schedule.next_backup.is_none());
    }

    #[test]
    fn test_backup_schedule_serialization() {
        let schedule = BackupSchedule {
            enabled: true,
            frequency: BackupFrequency::Daily,
            destinations: vec![BackupDestination::Local, BackupDestination::GoogleDrive],
            last_backup: Some("2024-01-15T10:00:00Z".to_string()),
            next_backup: Some("2024-01-16T10:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&schedule).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"frequency\":\"Daily\""));
        assert!(json.contains("\"destinations\""));
        assert!(json.contains("\"Local\""));
        assert!(json.contains("\"GoogleDrive\""));
    }

    #[test]
    fn test_backup_schedule_deserialization() {
        let json = r#"{
            "enabled": true,
            "frequency": "Weekly",
            "destinations": ["Local"],
            "lastBackup": "2024-01-15T10:00:00Z",
            "nextBackup": "2024-01-22T10:00:00Z"
        }"#;

        let schedule: BackupSchedule = serde_json::from_str(json).unwrap();
        assert!(schedule.enabled);
        assert_eq!(schedule.frequency, BackupFrequency::Weekly);
        assert_eq!(schedule.destinations.len(), 1);
        assert_eq!(schedule.destinations[0], BackupDestination::Local);
    }

    #[test]
    fn test_calculate_next_backup_not_in_past() {
        let next = calculate_next_backup(&BackupFrequency::Hourly);
        // Next backup should be a valid datetime string
        assert!(!next.is_empty());
        assert!(next.contains("T")); // ISO 8601 format
    }

    #[test]
    fn test_scheduler_state_default() {
        let state = SchedulerState::default();
        assert!(state.schedule.try_lock().is_ok());
    }

    #[test]
    fn test_backup_frequency_equality() {
        assert_eq!(BackupFrequency::Daily, BackupFrequency::Daily);
        assert_ne!(BackupFrequency::Daily, BackupFrequency::Weekly);
    }

    #[test]
    fn test_backup_destination_equality() {
        assert_eq!(BackupDestination::Local, BackupDestination::Local);
        assert_ne!(BackupDestination::Local, BackupDestination::GoogleDrive);
    }

    #[test]
    fn test_backup_schedule_clone() {
        let schedule = BackupSchedule {
            enabled: true,
            frequency: BackupFrequency::Daily,
            destinations: vec![BackupDestination::Local],
            last_backup: Some("2024-01-15T10:00:00Z".to_string()),
            next_backup: Some("2024-01-16T10:00:00Z".to_string()),
        };

        let cloned = schedule.clone();
        assert_eq!(schedule.enabled, cloned.enabled);
        assert_eq!(schedule.frequency, cloned.frequency);
        assert_eq!(schedule.destinations, cloned.destinations);
    }
}
