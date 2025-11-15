mod commands;
mod state;

use tauri::Manager;
use tracing::info;

use commands::{
    ai::{check_ai_availability, summarize_text},
    analytics::{
        add_price_history, clear_all_alerts, compare_prices, create_alert, delete_summary,
        dismiss_alert, get_latest_price, list_alerts, list_price_history, list_summary_history,
        mark_alert_read, save_summary,
    },
    backup::{export_backup_data, get_backup_file_path},
    body_metrics::{bulk_delete_body_metrics, delete_body_metric, get_body_metric, list_body_metrics, log_body_metric, update_body_metric},
    defaults::{get_default_peptides, populate_default_peptides},
    doses::{bulk_delete_doses, delete_dose_log, list_dose_logs, list_dose_logs_for_protocol, log_dose},
    drive::{
        check_drive_status, complete_drive_oauth, disconnect_drive, start_drive_oauth,
        upload_to_drive, OAuthState,
    },
    health::{checkpoint_database, get_database_health, get_database_stats, optimize_database, verify_database_integrity},
    literature::{list_literature, open_external_url, search_cached_literature, search_literature},
    protocols::{add_protocol_tag, bulk_add_tag_to_protocols, bulk_delete_protocols, bulk_toggle_favorite_protocols, delete_protocol, list_protocols, remove_protocol_tag, save_protocol, toggle_protocol_favorite, update_protocol_tags},
    restore::{preview_backup, restore_from_backup},
    schedules::{
        create_dose_schedule, delete_dose_schedule, get_pending_dose_reminders,
        list_dose_schedules, update_dose_schedule,
    },
    scheduler_v2::{
        get_backup_history, get_backup_progress, get_backup_schedule, trigger_manual_backup,
        update_backup_schedule, SchedulerState,
    },
    suppliers::{
        create_inventory_item, create_supplier, delete_inventory_item, delete_supplier,
        get_inventory_item, get_supplier, list_inventory, list_inventory_by_protocol,
        list_suppliers, scrape_supplier_website, update_inventory_item, update_supplier,
    },
};
use state::build_state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let state = build_state().map_err(|err| {
                let msg = format!("Failed to initialize application state: {err:#}");
                eprintln!("{msg}");
                let boxed: Box<dyn std::error::Error> = err.into();
                tauri::Error::Setup(boxed.into())
            })?;

            let scheduler_state = SchedulerState::new();
            let state_arc = std::sync::Arc::new(state);

            // Run database health check on startup
            info!("Running startup database health check...");
            match state_arc.storage.health_check() {
                Ok(report) if report.is_healthy => {
                    info!(
                        "✓ Database health check: OK ({:.2} MB, WAL: {}, FK: {})",
                        report.size_mb,
                        report.wal_mode,
                        report.foreign_keys_enabled
                    );
                }
                Ok(report) => {
                    tracing::error!(
                        "✗ Database corruption detected: {}",
                        report.integrity_result
                    );
                    tracing::error!("Please restore from a backup or contact support");
                    // Continue loading but warn user
                }
                Err(e) => {
                    tracing::warn!("Health check failed: {:#}", e);
                    // Non-fatal, continue loading
                }
            }

            // Store app handle for notifications
            let scheduler_clone_handle = scheduler_state.clone();
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                scheduler_clone_handle.set_app_handle(app_handle).await;
            });

            // Load schedule from disk
            let scheduler_clone = scheduler_state.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = scheduler_clone.load_from_disk().await {
                    eprintln!("Failed to load backup schedule: {:#}", e);
                }
            });

            // Start background scheduler
            let scheduler_clone2 = scheduler_state.clone();
            let state_clone = state_arc.clone();
            tauri::async_runtime::spawn(async move {
                // Give the app a moment to fully initialize
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                scheduler_clone2.start_scheduler(state_clone).await;
            });

            app.manage(state_arc);
            app.manage(OAuthState::default());
            app.manage(scheduler_state);
            info!("PepTrack initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_protocols,
            save_protocol,
            toggle_protocol_favorite,
            update_protocol_tags,
            add_protocol_tag,
            remove_protocol_tag,
            delete_protocol,
            bulk_delete_protocols,
            bulk_add_tag_to_protocols,
            bulk_toggle_favorite_protocols,
            check_ai_availability,
            summarize_text,
            list_literature,
            open_external_url,
            search_cached_literature,
            search_literature,
            log_dose,
            list_dose_logs,
            list_dose_logs_for_protocol,
            delete_dose_log,
            bulk_delete_doses,
            // Body metrics commands
            log_body_metric,
            list_body_metrics,
            get_body_metric,
            update_body_metric,
            delete_body_metric,
            bulk_delete_body_metrics,
            export_backup_data,
            get_backup_file_path,
            start_drive_oauth,
            complete_drive_oauth,
            check_drive_status,
            disconnect_drive,
            upload_to_drive,
            get_backup_schedule,
            get_backup_history,
            get_backup_progress,
            update_backup_schedule,
            trigger_manual_backup,
            restore_from_backup,
            preview_backup,
            // Supplier commands
            create_supplier,
            list_suppliers,
            get_supplier,
            update_supplier,
            delete_supplier,
            scrape_supplier_website,
            // Inventory commands
            create_inventory_item,
            list_inventory,
            list_inventory_by_protocol,
            get_inventory_item,
            update_inventory_item,
            delete_inventory_item,
            // Analytics commands
            add_price_history,
            list_price_history,
            get_latest_price,
            compare_prices,
            create_alert,
            list_alerts,
            mark_alert_read,
            dismiss_alert,
            clear_all_alerts,
            save_summary,
            list_summary_history,
            delete_summary,
            // Dose schedule commands
            create_dose_schedule,
            list_dose_schedules,
            update_dose_schedule,
            delete_dose_schedule,
            get_pending_dose_reminders,
            // Health & diagnostics commands
            get_database_health,
            verify_database_integrity,
            optimize_database,
            checkpoint_database,
            get_database_stats,
            // Default peptides
            get_default_peptides,
            populate_default_peptides
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
