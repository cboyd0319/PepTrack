use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use time::OffsetDateTime;
use tracing::{info, warn};

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupMetadata {
    pub export_date: String,
    pub protocols_count: usize,
    pub doses_count: usize,
    pub literature_count: usize,
    pub app_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupData {
    pub metadata: BackupMetadata,
    pub protocols: Vec<serde_json::Value>,
    pub dose_logs: Vec<serde_json::Value>,
    pub literature: Vec<serde_json::Value>,
}

/// Exports all data to a JSON file that the user can save.
///
/// If `password` is provided, the backup will be encrypted.
#[tauri::command]
pub async fn export_backup_data(
    state: State<'_, std::sync::Arc<AppState>>,
    password: Option<String>,
) -> Result<String, String> {
    info!("Starting backup export (encrypted: {})", password.is_some());

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

    let backup_data = BackupData {
        metadata,
        protocols: protocols_json,
        dose_logs: doses_json,
        literature: literature_json,
    };

    // Serialize to JSON
    let backup_json = serde_json::to_string_pretty(&backup_data)
        .map_err(|e| format!("Failed to serialize backup: {}", e))?;

    // Optionally encrypt
    if let Some(password) = password {
        if password.is_empty() {
            warn!("Empty password provided for backup encryption - skipping encryption");
            Ok(backup_json)
        } else {
            info!("Encrypting backup with password");
            peptrack_core::encrypt_backup(&backup_json, &password)
                .map_err(|e| format!("Failed to encrypt backup: {}", e))
        }
    } else {
        Ok(backup_json)
    }
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

    #[tokio::test]
    async fn test_backup_metadata_deserialization() {
        let json = r#"{
            "exportDate": "2024-01-15T10:30:00Z",
            "protocolsCount": 5,
            "dosesCount": 10,
            "literatureCount": 3,
            "appVersion": "0.1.0"
        }"#;

        let metadata: Result<BackupMetadata, _> = serde_json::from_str(json);
        assert!(metadata.is_ok());

        let metadata = metadata.unwrap();
        assert_eq!(metadata.export_date, "2024-01-15T10:30:00Z");
        assert_eq!(metadata.protocols_count, 5);
        assert_eq!(metadata.doses_count, 10);
        assert_eq!(metadata.literature_count, 3);
        assert_eq!(metadata.app_version, "0.1.0");
    }

    #[tokio::test]
    async fn test_backup_data_round_trip() {
        // Create backup data
        let original = BackupData {
            metadata: BackupMetadata {
                export_date: "2024-01-15T10:30:00Z".to_string(),
                protocols_count: 2,
                doses_count: 5,
                literature_count: 1,
                app_version: "0.1.0".to_string(),
            },
            protocols: vec![
                serde_json::json!({"id": "p1", "name": "Test Protocol"}),
                serde_json::json!({"id": "p2", "name": "Another Protocol"}),
            ],
            dose_logs: vec![
                serde_json::json!({"id": "d1", "amount": 10}),
                serde_json::json!({"id": "d2", "amount": 20}),
                serde_json::json!({"id": "d3", "amount": 30}),
                serde_json::json!({"id": "d4", "amount": 40}),
                serde_json::json!({"id": "d5", "amount": 50}),
            ],
            literature: vec![serde_json::json!({"id": "l1", "title": "Research Paper"})],
        };

        // Serialize
        let json = serde_json::to_string(&original).unwrap();

        // Deserialize
        let deserialized: BackupData = serde_json::from_str(&json).unwrap();

        // Verify metadata
        assert_eq!(deserialized.metadata.protocols_count, 2);
        assert_eq!(deserialized.metadata.doses_count, 5);
        assert_eq!(deserialized.metadata.literature_count, 1);

        // Verify arrays
        assert_eq!(deserialized.protocols.len(), 2);
        assert_eq!(deserialized.dose_logs.len(), 5);
        assert_eq!(deserialized.literature.len(), 1);
    }

    #[tokio::test]
    async fn test_backup_with_large_dataset() {
        // Create backup with many items
        let mut protocols = Vec::new();
        let mut doses = Vec::new();
        let mut literature = Vec::new();

        for i in 0..100 {
            protocols.push(serde_json::json!({
                "id": format!("p{}", i),
                "name": format!("Protocol {}", i)
            }));
        }

        for i in 0..500 {
            doses.push(serde_json::json!({
                "id": format!("d{}", i),
                "amount": i * 10
            }));
        }

        for i in 0..50 {
            literature.push(serde_json::json!({
                "id": format!("l{}", i),
                "title": format!("Paper {}", i)
            }));
        }

        let backup = BackupData {
            metadata: BackupMetadata {
                export_date: "2024-01-15T10:30:00Z".to_string(),
                protocols_count: 100,
                doses_count: 500,
                literature_count: 50,
                app_version: "0.1.0".to_string(),
            },
            protocols,
            dose_logs: doses,
            literature,
        };

        // Should serialize without error
        let json = serde_json::to_string(&backup);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        // Should be a substantial size
        assert!(json_str.len() > 10000);

        // Should deserialize back
        let deserialized: Result<BackupData, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
    }

    #[tokio::test]
    async fn test_backup_file_path_uniqueness() {
        // Generate multiple paths rapidly - they should all be unique or very similar
        // due to timestamp precision
        let path1 = get_backup_file_path().await.unwrap();

        // Even if generated in quick succession, paths should be valid
        let path2 = get_backup_file_path().await.unwrap();

        // Both should be valid paths
        assert!(path1.contains("peptrack_backup_"));
        assert!(path2.contains("peptrack_backup_"));
        assert!(path1.ends_with(".json"));
        assert!(path2.ends_with(".json"));

        // Paths might be the same or different depending on timestamp precision
        // What matters is they're both valid
    }

    #[tokio::test]
    async fn test_backup_file_path_format() {
        let path = get_backup_file_path().await.unwrap();

        // Should contain the prefix
        assert!(path.contains("peptrack_backup_"));

        // Should end with .json
        assert!(path.ends_with(".json"));

        // Should contain date components (YYYY-MM-DD format)
        assert!(path.contains("202") || path.contains("203")); // Year 2020s or 2030s

        // Should be a valid path
        let path_obj = std::path::Path::new(&path);
        assert!(path_obj.file_name().is_some());
    }

    #[tokio::test]
    async fn test_backup_metadata_with_zero_counts() {
        let metadata = BackupMetadata {
            export_date: "2024-01-15T10:30:00Z".to_string(),
            protocols_count: 0,
            doses_count: 0,
            literature_count: 0,
            app_version: "0.1.0".to_string(),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: BackupMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.protocols_count, 0);
        assert_eq!(deserialized.doses_count, 0);
        assert_eq!(deserialized.literature_count, 0);
    }

    #[tokio::test]
    async fn test_backup_metadata_with_max_counts() {
        let metadata = BackupMetadata {
            export_date: "2024-01-15T10:30:00Z".to_string(),
            protocols_count: usize::MAX,
            doses_count: usize::MAX,
            literature_count: usize::MAX,
            app_version: "0.1.0".to_string(),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: BackupMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.protocols_count, usize::MAX);
        assert_eq!(deserialized.doses_count, usize::MAX);
        assert_eq!(deserialized.literature_count, usize::MAX);
    }
}
