use peptrack_core::models::{DatabaseStats, HealthReport};
use tauri::State;
use tracing::info;

use crate::state::AppState;

/// Get comprehensive database health report
#[tauri::command]
pub async fn get_database_health(
    state: State<'_, std::sync::Arc<AppState>>,
) -> Result<HealthReport, String> {
    info!("Running database health check");

    state
        .storage
        .health_check()
        .map_err(|err| {
            tracing::error!("Health check failed: {:#}", err);
            err.to_string()
        })
}

/// Verify database integrity (quick check)
/// Returns Ok if healthy, Err if corrupted
#[tauri::command]
pub async fn verify_database_integrity(
    state: State<'_, std::sync::Arc<AppState>>,
) -> Result<(), String> {
    info!("Verifying database integrity");

    state
        .storage
        .verify_integrity()
        .map_err(|err| {
            tracing::error!("Integrity verification failed: {:#}", err);
            err.to_string()
        })
}

/// Optimize database performance
/// Runs PRAGMA optimize, incremental vacuum, and ANALYZE
#[tauri::command]
pub async fn optimize_database(
    state: State<'_, std::sync::Arc<AppState>>,
) -> Result<(), String> {
    info!("Optimizing database");

    state
        .storage
        .optimize()
        .map_err(|err| {
            tracing::error!("Database optimization failed: {:#}", err);
            err.to_string()
        })
}

/// Checkpoint WAL file
/// Merges Write-Ahead Log into main database
/// Mode: "PASSIVE", "FULL", "RESTART", or "TRUNCATE"
#[tauri::command]
pub async fn checkpoint_database(
    state: State<'_, std::sync::Arc<AppState>>,
    mode: Option<String>,
) -> Result<(), String> {
    let checkpoint_mode = mode.unwrap_or_else(|| "PASSIVE".to_string());
    info!("Checkpointing database (mode: {})", checkpoint_mode);

    state
        .storage
        .checkpoint_wal(&checkpoint_mode)
        .map_err(|err| {
            tracing::error!("Database checkpoint failed: {:#}", err);
            err.to_string()
        })
}

/// Get detailed database statistics
/// Returns metrics about size, fragmentation, WAL usage
#[tauri::command]
pub async fn get_database_stats(
    state: State<'_, std::sync::Arc<AppState>>,
) -> Result<DatabaseStats, String> {
    info!("Getting database statistics");

    state
        .storage
        .get_stats()
        .map_err(|err| {
            tracing::error!("Failed to get database stats: {:#}", err);
            err.to_string()
        })
}
