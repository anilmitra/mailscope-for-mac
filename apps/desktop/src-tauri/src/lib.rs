mod commands;
mod state;

use mailscope_storage::Database;
use state::AppState;
use std::sync::Arc;
use tauri::Manager;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn run() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_dir)?;

            let db_path = app_dir.join("mailscope.db");
            let db = Database::open(&db_path).expect("failed to open database");
            db.migrate().expect("database migration failed");

            app.manage(Arc::new(AppState::new(db)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Accounts
            commands::accounts::list_accounts,
            commands::accounts::get_account,
            commands::accounts::add_imap_account,
            commands::accounts::remove_account,
            commands::accounts::check_account_health,
            commands::accounts::start_gmail_oauth,
            commands::accounts::complete_gmail_oauth,
            commands::accounts::start_microsoft_oauth,
            commands::accounts::complete_microsoft_oauth,
            // Seed groups
            commands::accounts::list_seed_groups,
            commands::accounts::create_seed_group,
            commands::accounts::delete_seed_group,
            commands::accounts::add_account_to_group,
            commands::accounts::remove_account_from_group,
            commands::accounts::get_seed_group_members,
            // Sending profiles
            commands::runs::list_sending_profiles,
            commands::runs::create_sending_profile,
            commands::runs::delete_sending_profile,
            // Test runs
            commands::runs::list_runs,
            commands::runs::get_run,
            commands::runs::generate_run_token,
            commands::runs::create_run,
            commands::runs::start_run,
            commands::runs::cancel_run,
            // Results
            commands::results::get_run_results,
            commands::results::get_run_diagnostics,
            commands::results::export_run_csv,
            commands::results::export_run_json,
            // Dashboard
            commands::results::get_dashboard_stats,
            // DNS
            commands::results::check_domain,
            commands::results::lookup_provider_by_email,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MailScope");
}
