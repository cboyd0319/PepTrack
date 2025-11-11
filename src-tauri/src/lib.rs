mod commands;
mod state;

use tauri::Manager;
use tracing::info;

use commands::{
    ai::{check_ai_availability, summarize_text},
    backup::{export_backup_data, get_backup_file_path},
    doses::{delete_dose_log, list_dose_logs, list_dose_logs_for_protocol, log_dose},
    literature::{list_literature, search_cached_literature, search_literature},
    protocols::{list_protocols, save_protocol},
};
use state::build_state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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

            app.manage(state);
            info!("PepTrack initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_protocols,
            save_protocol,
            check_ai_availability,
            summarize_text,
            list_literature,
            search_cached_literature,
            search_literature,
            log_dose,
            list_dose_logs,
            list_dose_logs_for_protocol,
            delete_dose_log,
            export_backup_data,
            get_backup_file_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
