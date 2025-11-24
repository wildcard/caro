// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod database;
mod models;

use commands::*;
use database::Database;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Database>>,
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("cmdai_gui=debug".parse().unwrap()),
        )
        .init();

    tauri::Builder::default()
        .setup(|app| {
            // Initialize database
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join("cmdai.db");

            let db = Database::new(db_path)?;
            let state = AppState {
                db: Arc::new(Mutex::new(db)),
            };

            app.manage(state);
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            // Config commands
            get_config,
            update_config,
            // Execution history commands
            get_execution_history,
            add_execution,
            get_execution_by_id,
            delete_execution,
            // Rating commands
            rate_execution,
            get_execution_ratings,
            // Voting commands
            vote_execution,
            get_execution_votes,
            // Command generation
            generate_command,
            // Analytics
            get_analytics,
            // Export
            export_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
