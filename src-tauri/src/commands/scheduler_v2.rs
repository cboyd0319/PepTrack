use anyhow::{Context, Result};
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::io::{Write as _, Read as _};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;
use time::OffsetDateTime;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use crate::state::AppState;

/// Backup frequency options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BackupFrequency {
    Hourly,
    /// Daily at a specific hour (0-23)
    DailyAt { hour: u8 },
    Weekly,
    Manual,
}

/// Backup destination options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BackupDestination {
    Local,
    GoogleDrive,
    Dropbox,
}

/// Backup history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupHistoryEntry {
    pub timestamp: String,
    pub destinations: Vec<BackupDestination>,
    pub success: bool,
    pub error_message: Option<String>,
    pub size_bytes: Option<u64>,
    pub compressed: bool,
}

/// Cleanup settings for old backups
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupSettings {
    pub enabled: bool,
    /// Keep last N backups
    pub keep_last_n: Option<usize>,
    /// Delete backups older than N days
    pub older_than_days: Option<u32>,
}

impl Default for CleanupSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            keep_last_n: Some(10),
            older_than_days: Some(30),
        }
    }
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
    pub backup_on_close: bool,
    pub compress: bool,
    pub cleanup_settings: CleanupSettings,
    pub max_retries: u32,
}

impl Default for BackupSchedule {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: BackupFrequency::Manual,
            destinations: vec![BackupDestination::Local],
            last_backup: None,
            next_backup: None,
            backup_on_close: false,
            compress: true,
            cleanup_settings: CleanupSettings::default(),
            max_retries: 3,
        }
    }
}

/// Backup progress for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupProgress {
    pub is_running: bool,
    pub current_step: String,
    pub completed_steps: Vec<String>,
    pub failed_steps: Vec<String>,
}

/// Scheduler state for managing background tasks
#[derive(Clone)]
pub struct SchedulerState {
    schedule: Arc<RwLock<BackupSchedule>>,
    history: Arc<RwLock<Vec<BackupHistoryEntry>>>,
    progress: Arc<RwLock<BackupProgress>>,
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    backup_lock: Arc<Mutex<()>>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
}

impl Default for SchedulerState {
    fn default() -> Self {
        Self::new()
    }
}

const SCHEDULE_FILENAME: &str = "backup_schedule.json";
const HISTORY_FILENAME: &str = "backup_history.json";
const MAX_HISTORY_ENTRIES: usize = 100;

impl SchedulerState {
    pub fn new() -> Self {
        Self {
            schedule: Arc::new(RwLock::new(BackupSchedule::default())),
            history: Arc::new(RwLock::new(Vec::new())),
            progress: Arc::new(RwLock::new(BackupProgress {
                is_running: false,
                current_step: String::new(),
                completed_steps: Vec::new(),
                failed_steps: Vec::new(),
            })),
            task_handle: Arc::new(Mutex::new(None)),
            backup_lock: Arc::new(Mutex::new(())),
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_app_handle(&self, handle: AppHandle) {
        *self.app_handle.lock().await = Some(handle);
    }

    async fn send_notification(&self, title: &str, body: &str) {
        if let Some(handle) = self.app_handle.lock().await.as_ref() {
            handle
                .notification()
                .builder()
                .title(title)
                .body(body)
                .show()
                .ok();
        }
    }

    /// Load schedule from disk on startup
    pub async fn load_from_disk(&self) -> Result<()> {
        // Load schedule
        match load_schedule_from_disk().await {
            Ok(schedule) => {
                *self.schedule.write().await = schedule;
                info!("Loaded backup schedule from disk");
            }
            Err(e) => {
                warn!("Failed to load backup schedule: {:#}", e);
            }
        }

        // Load history
        match load_history_from_disk().await {
            Ok(history) => {
                *self.history.write().await = history;
                info!("Loaded backup history from disk");
            }
            Err(e) => {
                warn!("Failed to load backup history: {:#}", e);
            }
        }

        Ok(())
    }

    /// Start the background scheduler task
    pub async fn start_scheduler(&self, app_state: Arc<AppState>) {
        let schedule_arc = self.schedule.clone();
        let history_arc = self.history.clone();
        let progress_arc = self.progress.clone();
        let backup_lock = self.backup_lock.clone();
        let notif_state = self.clone();

        let handle = tokio::spawn(async move {
            info!("Background backup scheduler started");

            loop {
                // Check if enabled
                let schedule = schedule_arc.read().await.clone();

                if !schedule.enabled {
                    // Sleep longer when disabled to save CPU
                    tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
                    continue;
                }

                // Check if it's time to backup
                if let Some(next_backup_str) = &schedule.next_backup {
                    match OffsetDateTime::parse(next_backup_str, &time::format_description::well_known::Rfc3339) {
                        Ok(next_backup_time) => {
                            let now = OffsetDateTime::now_utc();

                            if now >= next_backup_time {
                                info!("Scheduled backup triggered");

                                // Try to acquire lock (non-blocking)
                                if let Ok(_guard) = backup_lock.try_lock() {
                                    if let Err(e) = perform_scheduled_backup_with_retry(
                                        &app_state,
                                        &schedule_arc,
                                        &history_arc,
                                        &progress_arc,
                                        &notif_state,
                                    ).await {
                                        error!("Scheduled backup failed: {:#}", e);
                                        notif_state.send_notification(
                                            "Backup Failed",
                                            &format!("Scheduled backup failed: {}", e)
                                        ).await;
                                    }
                                } else {
                                    warn!("Backup already in progress, skipping");
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse next backup time: {:#}", e);
                        }
                    }
                }

                // Check every minute
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });

        *self.task_handle.lock().await = Some(handle);
        info!("Background scheduler task spawned");
    }

    pub async fn trigger_backup_on_close(&self, app_state: &AppState) -> Result<()> {
        let schedule = self.schedule.read().await.clone();

        if !schedule.backup_on_close {
            return Ok(());
        }

        info!("Triggering backup on app close");

        // Try to acquire lock
        let _guard = self.backup_lock.try_lock()
            .map_err(|_| anyhow::anyhow!("Backup already in progress"))?;

        perform_scheduled_backup_with_retry(
            app_state,
            &self.schedule,
            &self.history,
            &self.progress,
            self,
        ).await?;

        Ok(())
    }
}

/// Gets the current backup schedule
#[tauri::command]
pub async fn get_backup_schedule(state: State<'_, SchedulerState>) -> Result<BackupSchedule, String> {
    let schedule = state.schedule.read().await.clone();
    Ok(schedule)
}

/// Gets backup history
#[tauri::command]
pub async fn get_backup_history(state: State<'_, SchedulerState>) -> Result<Vec<BackupHistoryEntry>, String> {
    let history = state.history.read().await.clone();
    Ok(history)
}

/// Gets current backup progress
#[tauri::command]
pub async fn get_backup_progress(state: State<'_, SchedulerState>) -> Result<BackupProgress, String> {
    let progress = state.progress.read().await.clone();
    Ok(progress)
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

    *state.schedule.write().await = updated_schedule.clone();

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

    // Try to acquire lock
    let _guard = scheduler_state.backup_lock.try_lock()
        .map_err(|_| "A backup is already in progress".to_string())?;

    let result = perform_scheduled_backup_with_retry(
        &app_state,
        &scheduler_state.schedule,
        &scheduler_state.history,
        &scheduler_state.progress,
        &scheduler_state,
    ).await;

    match result {
        Ok(msg) => {
            scheduler_state.send_notification("Backup Complete", &msg).await;
            Ok(msg)
        }
        Err(e) => {
            let error_msg = format!("Backup failed: {}", e);
            scheduler_state.send_notification("Backup Failed", &error_msg).await;
            Err(error_msg)
        }
    }
}

// Helper functions

fn calculate_next_backup(frequency: &BackupFrequency) -> String {
    let now = OffsetDateTime::now_utc();
    let next = match frequency {
        BackupFrequency::Hourly => now + time::Duration::hours(1),
        BackupFrequency::DailyAt { hour } => {
            // Calculate next occurrence of the specified hour
            let target_hour = *hour as i8;
            let current_hour = now.hour() as i8;

            let hours_until_target = if target_hour > current_hour {
                target_hour - current_hour
            } else {
                24 + target_hour - current_hour
            };

            now + time::Duration::hours(hours_until_target as i64)
        },
        BackupFrequency::Weekly => now + time::Duration::weeks(1),
        BackupFrequency::Manual => now,
    };
    next.to_string()
}

async fn perform_scheduled_backup_with_retry(
    app_state: &AppState,
    schedule_arc: &Arc<RwLock<BackupSchedule>>,
    history_arc: &Arc<RwLock<Vec<BackupHistoryEntry>>>,
    progress_arc: &Arc<RwLock<BackupProgress>>,
    _notif_state: &SchedulerState,
) -> Result<String> {
    let schedule = schedule_arc.read().await.clone();
    let max_retries = schedule.max_retries;
    let compress = schedule.compress;

    let mut last_error = None;

    for attempt in 1..=max_retries {
        if attempt > 1 {
            info!("Retry attempt {} of {}", attempt, max_retries);
            // Exponential backoff
            let wait_secs = 2u64.pow(attempt - 1);
            tokio::time::sleep(tokio::time::Duration::from_secs(wait_secs)).await;
        }

        match perform_single_backup(
            app_state,
            &schedule,
            progress_arc,
            compress,
        ).await {
            Ok(result) => {
                // Success! Record history
                let entry = BackupHistoryEntry {
                    timestamp: OffsetDateTime::now_utc().to_string(),
                    destinations: schedule.destinations.clone(),
                    success: true,
                    error_message: None,
                    size_bytes: Some(result.size_bytes),
                    compressed: compress,
                };

                add_history_entry(history_arc, entry).await;

                // Update schedule
                let mut sched = schedule_arc.write().await;
                sched.last_backup = Some(OffsetDateTime::now_utc().to_string());
                if sched.enabled {
                    sched.next_backup = Some(calculate_next_backup(&sched.frequency));
                }
                save_schedule_to_disk(&sched).await.ok();

                // Clear progress
                let mut progress = progress_arc.write().await;
                progress.is_running = false;

                return Ok(result.message);
            }
            Err(e) => {
                error!("Backup attempt {} failed: {:#}", attempt, e);
                last_error = Some(e);
            }
        }
    }

    // All retries failed
    let error = last_error.unwrap();
    let entry = BackupHistoryEntry {
        timestamp: OffsetDateTime::now_utc().to_string(),
        destinations: schedule.destinations.clone(),
        success: false,
        error_message: Some(error.to_string()),
        size_bytes: None,
        compressed: compress,
    };

    add_history_entry(history_arc, entry).await;

    Err(error)
}

struct BackupResult {
    message: String,
    size_bytes: u64,
}

async fn perform_single_backup(
    app_state: &AppState,
    schedule: &BackupSchedule,
    progress_arc: &Arc<RwLock<BackupProgress>>,
    compress: bool,
) -> Result<BackupResult> {
    // Update progress
    {
        let mut progress = progress_arc.write().await;
        progress.is_running = true;
        progress.current_step = "Preparing backup...".to_string();
        progress.completed_steps.clear();
        progress.failed_steps.clear();
    }

    let mut results = Vec::new();
    let mut total_size = 0u64;

    for destination in &schedule.destinations {
        // Update progress
        {
            let mut progress = progress_arc.write().await;
            progress.current_step = format!("Backing up to {:?}...", destination);
        }

        match destination {
            BackupDestination::Local => {
                match perform_local_backup(app_state, compress).await {
                    Ok((path, size)) => {
                        info!("Local backup successful: {}", path);
                        results.push(format!("Local: {}", path));
                        total_size += size;

                        let mut progress = progress_arc.write().await;
                        progress.completed_steps.push(format!("Local backup: {}", path));
                    }
                    Err(e) => {
                        error!("Local backup failed: {:#}", e);
                        let mut progress = progress_arc.write().await;
                        progress.failed_steps.push(format!("Local backup: {}", e));
                        return Err(e);
                    }
                }
            }
            BackupDestination::GoogleDrive => {
                // Check Drive connection first
                match check_drive_connection(app_state).await {
                    Ok(true) => {
                        match perform_drive_backup(app_state, compress).await {
                            Ok((file_id, size)) => {
                                info!("Google Drive backup successful: {}", file_id);
                                results.push(format!("Drive: {}", file_id));
                                total_size += size;

                                let mut progress = progress_arc.write().await;
                                progress.completed_steps.push(format!("Drive backup: {}", file_id));
                            }
                            Err(e) => {
                                error!("Google Drive backup failed: {:#}", e);
                                let mut progress = progress_arc.write().await;
                                progress.failed_steps.push(format!("Drive backup: {}", e));
                                return Err(e);
                            }
                        }
                    }
                    Ok(false) => {
                        let err = anyhow::anyhow!("Google Drive not connected");
                        let mut progress = progress_arc.write().await;
                        progress.failed_steps.push("Drive backup: Not connected".to_string());
                        return Err(err);
                    }
                    Err(e) => {
                        let mut progress = progress_arc.write().await;
                        progress.failed_steps.push(format!("Drive backup: {}", e));
                        return Err(e);
                    }
                }
            }
            BackupDestination::Dropbox => {
                // TODO: Implement Dropbox backup
                warn!("Dropbox backup not yet implemented");
                let mut progress = progress_arc.write().await;
                progress.failed_steps.push("Dropbox backup: Not implemented".to_string());
            }
        }
    }

    // Perform cleanup if enabled
    if schedule.cleanup_settings.enabled {
        let mut progress = progress_arc.write().await;
        progress.current_step = "Cleaning up old backups...".to_string();

        if let Err(e) = perform_cleanup(&schedule.cleanup_settings).await {
            warn!("Cleanup failed: {:#}", e);
            progress.failed_steps.push(format!("Cleanup: {}", e));
        } else {
            progress.completed_steps.push("Cleanup completed".to_string());
        }
    }

    Ok(BackupResult {
        message: results.join(", "),
        size_bytes: total_size,
    })
}

async fn perform_local_backup(state: &AppState, compress: bool) -> Result<(String, u64)> {
    use crate::commands::backup::{BackupData, BackupMetadata};

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

    let timestamp = OffsetDateTime::now_utc()
        .format(&time::format_description::parse("[year]-[month]-[day]_[hour]-[minute]").unwrap())
        .unwrap_or_else(|_| "backup".to_string());

    let default_path = dirs::download_dir()
        .or_else(|| dirs::document_dir())
        .context("Could not determine download directory")?;

    let json = serde_json::to_string_pretty(&backup)?;

    let (filename, final_data, size) = if compress {
        let filename = format!("peptrack_backup_{}.json.gz", timestamp);
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(json.as_bytes())?;
        let compressed = encoder.finish()?;
        let size = compressed.len() as u64;
        (filename, compressed, size)
    } else {
        let filename = format!("peptrack_backup_{}.json", timestamp);
        let size = json.len() as u64;
        (filename, json.into_bytes(), size)
    };

    let full_path = default_path.join(&filename);
    std::fs::write(&full_path, final_data)?;

    // Verify backup
    verify_backup(&full_path, compress)?;

    Ok((full_path.to_string_lossy().to_string(), size))
}

async fn perform_drive_backup(state: &AppState, compress: bool) -> Result<(String, u64)> {
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

    let timestamp = OffsetDateTime::now_utc()
        .format(&time::format_description::parse("[year]-[month]-[day]_[hour]-[minute]").unwrap())
        .unwrap_or_else(|_| "backup".to_string());

    let json = serde_json::to_string_pretty(&backup)?;

    let (filename, content, size) = if compress {
        let filename = format!("peptrack_backup_{}.json.gz", timestamp);
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(json.as_bytes())?;
        let compressed = encoder.finish()?;
        let size = compressed.len() as u64;
        // Base64 encode for upload
        use base64::Engine as _;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&compressed);
        (filename, encoded, size)
    } else {
        let filename = format!("peptrack_backup_{}.json", timestamp);
        let size = json.len() as u64;
        (filename, json, size)
    };

    let tokens = drive::load_drive_tokens_internal(state).await
        .context("Google Drive not connected")?;

    let client = reqwest::Client::new();
    let folder_id = drive::get_or_create_folder_internal(&client, &tokens.access_token, "PepTrack Backups")
        .await
        .context("Failed to create/get Drive folder")?;

    let file_id = drive::upload_file_internal(&client, &tokens.access_token, &folder_id, &filename, &content)
        .await
        .context("Failed to upload to Drive")?;

    Ok((file_id, size))
}

async fn check_drive_connection(state: &AppState) -> Result<bool> {
    use crate::commands::drive;

    match drive::load_drive_tokens_internal(state).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

fn verify_backup(path: &std::path::Path, compressed: bool) -> Result<()> {
    let data = std::fs::read(path)?;

    if compressed {
        let mut decoder = GzDecoder::new(&data[..]);
        let mut json = String::new();
        decoder.read_to_string(&mut json)?;
        serde_json::from_str::<serde_json::Value>(&json)?;
    } else {
        let json = String::from_utf8(data)?;
        serde_json::from_str::<serde_json::Value>(&json)?;
    }

    Ok(())
}

async fn perform_cleanup(settings: &CleanupSettings) -> Result<()> {
    let download_dir = dirs::download_dir()
        .or_else(|| dirs::document_dir())
        .context("Could not determine download directory")?;

    // Find all peptrack backup files
    let entries = std::fs::read_dir(&download_dir)?;
    let mut backups: Vec<(std::path::PathBuf, std::time::SystemTime)> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("peptrack_backup_") && (name.ends_with(".json") || name.ends_with(".json.gz")) {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        backups.push((path, modified));
                    }
                }
            }
        }
    }

    // Sort by modification time (newest first)
    backups.sort_by(|a, b| b.1.cmp(&a.1));

    let mut to_delete = Vec::new();

    // Apply keep_last_n rule
    if let Some(keep_n) = settings.keep_last_n {
        if backups.len() > keep_n {
            to_delete.extend(backups.iter().skip(keep_n).map(|(p, _)| p.clone()));
        }
    }

    // Apply older_than_days rule
    if let Some(days) = settings.older_than_days {
        let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(days as u64 * 86400);
        for (path, modified) in &backups {
            if *modified < cutoff && !to_delete.contains(path) {
                to_delete.push(path.clone());
            }
        }
    }

    // Delete files
    for path in to_delete {
        info!("Deleting old backup: {:?}", path);
        std::fs::remove_file(path)?;
    }

    Ok(())
}

async fn add_history_entry(history_arc: &Arc<RwLock<Vec<BackupHistoryEntry>>>, entry: BackupHistoryEntry) {
    let mut history = history_arc.write().await;
    history.insert(0, entry);

    // Limit history size
    if history.len() > MAX_HISTORY_ENTRIES {
        history.truncate(MAX_HISTORY_ENTRIES);
    }

    // Save to disk
    save_history_to_disk(&history).await.ok();
}

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

async fn save_history_to_disk(history: &[BackupHistoryEntry]) -> Result<()> {
    let data_dir = dirs::data_dir()
        .context("Unable to determine data directory")?
        .join("PepTrack");
    std::fs::create_dir_all(&data_dir)?;

    let history_file = data_dir.join(HISTORY_FILENAME);
    let json = serde_json::to_string_pretty(history)?;
    std::fs::write(&history_file, json)
        .context("Failed to save backup history")?;

    Ok(())
}

async fn load_history_from_disk() -> Result<Vec<BackupHistoryEntry>> {
    let data_dir = dirs::data_dir()
        .context("Unable to determine data directory")?
        .join("PepTrack");
    let history_file = data_dir.join(HISTORY_FILENAME);

    let json = std::fs::read_to_string(&history_file)
        .context("Backup history not found")?;
    let history: Vec<BackupHistoryEntry> = serde_json::from_str(&json)?;
    Ok(history)
}
