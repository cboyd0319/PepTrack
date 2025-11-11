use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;
use tauri::State;
use time::OffsetDateTime;
use tracing::{info, warn};

use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupMetadata {
    pub export_date: String,
    pub protocols_count: usize,
    pub doses_count: usize,
    pub literature_count: usize,
    pub app_version: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupData {
    pub metadata: BackupMetadata,
    pub protocols: Vec<serde_json::Value>,
    pub dose_logs: Vec<serde_json::Value>,
    pub literature: Vec<serde_json::Value>,
}

/// Exports all data to a JSON file that the user can save
#[tauri::command]
pub async fn export_backup_data(state: State<'_, AppState>) -> Result<BackupData, String> {
    info!("Starting backup export");

    // Load all data from storage
    let protocols = state
        .storage
        .list_protocols()
        .map_err(|e| {
            warn!("Failed to load protocols for backup: {:#}", e);
            format!("Could not load protocols: {}", e)
        })?;

    let doses = state
        .storage
        .list_dose_logs()
        .map_err(|e| {
            warn!("Failed to load dose logs for backup: {:#}", e);
            format!("Could not load dose logs: {}", e)
        })?;

    let literature = state
        .storage
        .list_literature()
        .map_err(|e| {
            warn!("Failed to load literature for backup: {:#}", e);
            format!("Could not load literature: {}", e)
        })?;

    let metadata = BackupMetadata {
        export_date: OffsetDateTime::now_utc().to_string(),
        protocols_count: protocols.len(),
        doses_count: doses.len(),
        literature_count: literature.len(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    info!(
        "Backup prepared: {} protocols, {} doses, {} literature entries",
        metadata.protocols_count, metadata.doses_count, metadata.literature_count
    );

    // Convert to JSON values for serialization
    let protocols_json = protocols
        .into_iter()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect();

    let doses_json = doses
        .into_iter()
        .map(|d| serde_json::to_value(d).unwrap_or_default())
        .collect();

    let literature_json = literature
        .into_iter()
        .map(|l| serde_json::to_value(l).unwrap_or_default())
        .collect();

    Ok(BackupData {
        metadata,
        protocols: protocols_json,
        dose_logs: doses_json,
        literature: literature_json,
    })
}

/// Gets recommended backup file path
#[tauri::command]
pub async fn get_backup_file_path() -> Result<String, String> {
    let now = OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::parse("[year]-[month]-[day]_[hour]-[minute]").unwrap())
        .unwrap_or_else(|_| "backup".to_string());

    let filename = format!("peptrack_backup_{}.json", timestamp);

    // Get user's downloads or documents folder
    let default_path = dirs::download_dir()
        .or_else(|| dirs::document_dir())
        .unwrap_or_else(|| PathBuf::from("."));

    let full_path = default_path.join(filename);

    Ok(full_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_backup_file_path_returns_valid_path() {
        let result = get_backup_file_path().await;
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.contains("peptrack_backup_"));
        assert!(path.ends_with(".json"));
    }

    #[tokio::test]
    async fn test_backup_metadata_serialization() {
        let metadata = BackupMetadata {
            export_date: "2024-01-15T10:30:00Z".to_string(),
            protocols_count: 5,
            doses_count: 10,
            literature_count: 3,
            app_version: "0.1.0".to_string(),
        };

        let json = serde_json::to_string(&metadata);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("exportDate"));
        assert!(json_str.contains("protocolsCount"));
        assert!(json_str.contains("dosesCount"));
        assert!(json_str.contains("literatureCount"));
        assert!(json_str.contains("appVersion"));
    }

    #[tokio::test]
    async fn test_backup_data_structure() {
        let metadata = BackupMetadata {
            export_date: "2024-01-15T10:30:00Z".to_string(),
            protocols_count: 0,
            doses_count: 0,
            literature_count: 0,
            app_version: "0.1.0".to_string(),
        };

        let backup = BackupData {
            metadata,
            protocols: vec![],
            dose_logs: vec![],
            literature: vec![],
        };

        let json = serde_json::to_string(&backup);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("metadata"));
        assert!(json_str.contains("protocols"));
        assert!(json_str.contains("doseLogs"));
        assert!(json_str.contains("literature"));
    }
}
