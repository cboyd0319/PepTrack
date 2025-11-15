use peptrack_core::models::HealthReport;
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
