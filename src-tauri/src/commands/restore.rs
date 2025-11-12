use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::io::Read;
use tauri::State;
use tracing::{info, warn};

use crate::commands::backup::BackupData;
use crate::state::AppState;

/// Restore data from a backup file.
///
/// If the backup is encrypted, `password` must be provided.
#[tauri::command]
pub async fn restore_from_backup(
    state: State<'_, std::sync::Arc<AppState>>,
    file_path: String,
    password: Option<String>,
) -> Result<RestoreResult, String> {
    info!("Restoring from backup: {}", file_path);

    // Read and parse backup file
    let backup_data = read_backup_file(&file_path, password.as_deref())
        .map_err(|e| format!("Failed to read backup file: {}", e))?;

    // Validate backup
    if backup_data.protocols.is_empty()
        && backup_data.dose_logs.is_empty()
        && backup_data.literature.is_empty()
    {
        return Err("Backup file appears to be empty".to_string());
    }

    let mut restored_counts = RestoreCounts {
        protocols: 0,
        dose_logs: 0,
        literature: 0,
    };

    // Restore protocols
    for protocol_value in backup_data.protocols {
        match serde_json::from_value::<peptrack_core::PeptideProtocol>(protocol_value) {
            Ok(protocol) => {
                if let Err(e) = state.storage.upsert_protocol(&protocol) {
                    warn!("Failed to restore protocol: {:#}", e);
                } else {
                    restored_counts.protocols += 1;
                }
            }
            Err(e) => {
                warn!("Failed to deserialize protocol: {:#}", e);
            }
        }
    }

    // Restore dose logs
    for dose_value in backup_data.dose_logs {
        match serde_json::from_value::<peptrack_core::DoseLog>(dose_value) {
            Ok(dose) => {
                if let Err(e) = state.storage.append_dose_log(&dose) {
                    warn!("Failed to restore dose log: {:#}", e);
                } else {
                    restored_counts.dose_logs += 1;
                }
            }
            Err(e) => {
                warn!("Failed to deserialize dose log: {:#}", e);
            }
        }
    }

    // Restore literature
    for lit_value in backup_data.literature {
        match serde_json::from_value::<peptrack_core::LiteratureEntry>(lit_value) {
            Ok(literature) => {
                if let Err(e) = state.storage.cache_literature(&literature) {
                    warn!("Failed to restore literature: {:#}", e);
                } else {
                    restored_counts.literature += 1;
                }
            }
            Err(e) => {
                warn!("Failed to deserialize literature: {:#}", e);
            }
        }
    }

    info!(
        "Restore complete: {} protocols, {} doses, {} literature",
        restored_counts.protocols, restored_counts.dose_logs, restored_counts.literature
    );

    Ok(RestoreResult {
        success: true,
        counts: restored_counts,
        metadata: backup_data.metadata,
    })
}

/// Preview backup file contents without restoring
#[tauri::command]
pub async fn preview_backup(
    file_path: String,
    password: Option<String>,
) -> Result<BackupPreview, String> {
    info!("Previewing backup: {}", file_path);

    let backup_data = read_backup_file(&file_path, password.as_deref())
        .map_err(|e| format!("Failed to read backup file: {}", e))?;

    Ok(BackupPreview {
        metadata: backup_data.metadata,
        protocols_count: backup_data.protocols.len(),
        dose_logs_count: backup_data.dose_logs.len(),
        literature_count: backup_data.literature.len(),
    })
}

// Helper functions

fn validate_backup_path(file_path: &str) -> Result<std::path::PathBuf> {
    use std::path::Path;

    let path = Path::new(file_path);

    // Resolve to canonical path to prevent path traversal
    let canonical = path.canonicalize()
        .context("Invalid file path or file does not exist")?;

    // Only allow reading from user directories (Documents, Downloads, Desktop, Home)
    let allowed_dirs = vec![
        dirs::download_dir(),
        dirs::document_dir(),
        dirs::desktop_dir(),
        dirs::home_dir(),
    ];

    let is_allowed = allowed_dirs.into_iter()
        .flatten()
        .any(|allowed| canonical.starts_with(&allowed));

    if !is_allowed {
        return Err(anyhow::anyhow!(
            "File must be in your Downloads, Documents, Desktop, or Home folder for security"
        ));
    }

    // Must have a valid extension
    let extension = canonical.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if extension != "json" && extension != "gz" {
        return Err(anyhow::anyhow!(
            "Invalid file type - backup files must be .json or .json.gz"
        ));
    }

    Ok(canonical)
}

fn read_backup_file(file_path: &str, password: Option<&str>) -> Result<BackupData> {
    // Validate path to prevent arbitrary file reads
    let validated_path = validate_backup_path(file_path)?;

    let data =
        std::fs::read(&validated_path).with_context(|| format!("Failed to read file: {}", validated_path.display()))?;

    // Try to detect if compressed
    let is_gzipped = file_path.ends_with(".gz") || is_gzip_data(&data);

    let json = if is_gzipped {
        let mut decoder = GzDecoder::new(&data[..]);
        let mut json = String::new();
        decoder
            .read_to_string(&mut json)
            .context("Failed to decompress backup file")?;
        json
    } else {
        String::from_utf8(data).context("Backup file is not valid UTF-8")?
    };

    // Check if encrypted and decrypt if necessary
    let decrypted_json = if peptrack_core::is_encrypted_backup(&json) {
        let password = password
            .ok_or_else(|| anyhow::anyhow!("Backup is encrypted but no password was provided"))?;

        peptrack_core::decrypt_backup(&json, password)
            .context("Failed to decrypt backup - check password")?
    } else {
        json
    };

    let backup: BackupData =
        serde_json::from_str(&decrypted_json).context("Failed to parse backup file as JSON")?;

    Ok(backup)
}

fn is_gzip_data(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b
}

// Response types

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub success: bool,
    pub counts: RestoreCounts,
    pub metadata: crate::commands::backup::BackupMetadata,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreCounts {
    pub protocols: usize,
    pub dose_logs: usize,
    pub literature: usize,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPreview {
    pub metadata: crate::commands::backup::BackupMetadata,
    pub protocols_count: usize,
    pub dose_logs_count: usize,
    pub literature_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_is_gzip_data() {
        let gzip_header = vec![0x1f, 0x8b, 0x08, 0x00];
        assert!(is_gzip_data(&gzip_header));

        let json_data = b"{\"test\": \"data\"}";
        assert!(!is_gzip_data(json_data));

        let empty = vec![];
        assert!(!is_gzip_data(&empty));
    }

    #[test]
    fn test_compression_decompression_round_trip() {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        // Create test JSON data
        let test_data = r#"{"metadata":{"version":"1.0","timestamp":"2025-01-01T00:00:00Z"},"protocols":[],"doseLogs":[],"literature":[]}"#;

        // Compress the data
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(test_data.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();

        // Verify it's detected as gzip
        assert!(
            is_gzip_data(&compressed),
            "Compressed data should be detected as gzip"
        );

        // Decompress using GzDecoder (same as read_backup_file logic)
        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();

        // Verify round trip
        assert_eq!(
            test_data, decompressed,
            "Decompressed data should match original"
        );
    }

    #[test]
    fn test_compression_decompression_large_data() {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        // Create larger test data to verify compression actually reduces size
        let mut test_data = String::from(
            r#"{"metadata":{"version":"1.0","timestamp":"2025-01-01T00:00:00Z"},"protocols":["#,
        );
        for i in 0..100 {
            if i > 0 {
                test_data.push(',');
            }
            test_data.push_str(&format!(
                r#"{{"id":"protocol-{}","name":"Test Protocol {}","peptideName":"Peptide{}"}}"#,
                i, i, i
            ));
        }
        test_data.push_str(r#"],"doseLogs":[],"literature":[]}"#);

        // Compress the data
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(test_data.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();

        // Verify compression actually reduced size
        assert!(
            compressed.len() < test_data.len(),
            "Compressed size should be smaller than original"
        );

        // Decompress and verify
        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();

        assert_eq!(
            test_data, decompressed,
            "Decompressed data should match original"
        );
    }

    #[test]
    fn test_decompression_corrupted_gzip() {
        // Create corrupted gzip data (valid header but invalid body)
        let corrupted = vec![0x1f, 0x8b, 0x08, 0x00, 0xff, 0xff, 0xff, 0xff];

        assert!(
            is_gzip_data(&corrupted),
            "Should detect as gzip based on header"
        );

        // Attempt to decompress - should fail
        let mut decoder = GzDecoder::new(&corrupted[..]);
        let mut decompressed = String::new();
        let result = decoder.read_to_string(&mut decompressed);

        assert!(result.is_err(), "Decompressing corrupted gzip should fail");
    }

    #[test]
    fn test_compression_empty_backup() {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        // Empty but valid backup structure
        let test_data = r#"{"metadata":{"version":"1.0","timestamp":"2025-01-01T00:00:00Z"},"protocols":[],"doseLogs":[],"literature":[]}"#;

        // Compress
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(test_data.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();

        // Decompress
        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();

        assert_eq!(
            test_data, decompressed,
            "Empty backup should compress/decompress correctly"
        );
    }

    #[test]
    fn test_compression_different_levels() {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let test_data = "test data ".repeat(1000); // Create some repetitive data

        // Test different compression levels
        let fast = {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
            encoder.write_all(test_data.as_bytes()).unwrap();
            encoder.finish().unwrap()
        };

        let default = {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(test_data.as_bytes()).unwrap();
            encoder.finish().unwrap()
        };

        let best = {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
            encoder.write_all(test_data.as_bytes()).unwrap();
            encoder.finish().unwrap()
        };

        // All should decompress to same data
        for compressed in [&fast, &default, &best] {
            let mut decoder = GzDecoder::new(&compressed[..]);
            let mut decompressed = String::new();
            decoder.read_to_string(&mut decompressed).unwrap();
            assert_eq!(test_data, decompressed);
        }

        // Best compression should generally be smallest (though not guaranteed for all data)
        assert!(
            best.len() <= default.len(),
            "Best compression should be <= default"
        );
    }
}
