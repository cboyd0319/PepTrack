mod commands;
mod state;

use tauri::Manager;
use tracing::info;

use commands::{
    ai::summarize_text,
    protocols::{list_protocols, save_protocol},
};
use state::build_state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = build_state().expect("Failed to initialize application state");

    tauri::Builder::default()
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            app.manage(state.clone());
            info!("PepTrack initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_protocols,
            save_protocol,
            summarize_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
