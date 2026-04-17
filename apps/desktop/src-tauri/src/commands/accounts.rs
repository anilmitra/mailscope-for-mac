use crate::state::AppState;
use mailscope_core::models::{ConnectedAccount, SeedGroup, SeedGroupMember};
use mailscope_storage::accounts as db_accounts;
use std::sync::Arc;
use tauri::State;

type Result<T> = std::result::Result<T, String>;
type AppStateRef<'a> = State<'a, Arc<AppState>>;

fn db_err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[tauri::command]
pub async fn list_accounts(state: AppStateRef<'_>) -> Result<Vec<ConnectedAccount>> {
    let db = state.db.lock().map_err(db_err)?;
    db_accounts::list_accounts(&db).map_err(db_err)
}

#[tauri::command]
pub async fn get_account(state: AppStateRef<'_>, id: i64) -> Result<ConnectedAccount> {
    let db = state.db.lock().map_err(db_err)?;
    db_accounts::get_account(&db, id).map_err(db_err)
}

#[derive(serde::Deserialize)]
pub struct AddImapParams {
    pub display_name: String,
    pub seed_email: String,
    pub host: String,
    pub port: u16,
    pub auth_type: String,
    pub password: String,
}

#[tauri::command]
pub async fn add_imap_account(
    state: AppStateRef<'_>,
    params: AddImapParams,
) -> Result<ConnectedAccount> {
    let db = state.db.lock().map_err(db_err)?;
    db_accounts::add_imap_account(
        &db,
        &params.display_name,
        &params.seed_email,
        &params.host,
        params.port,
        &params.auth_type,
        &params.password,
    )
    .map_err(db_err)
}

#[tauri::command]
pub async fn remove_account(state: AppStateRef<'_>, id: i64) -> Result<()> {
    let db = state.db.lock().map_err(db_err)?;
    db_accounts::remove_account(&db, id).map_err(db_err)
}

#[tauri::command]
pub async fn check_account_health(
    state: AppStateRef<'_>,
    id: i64,
) -> Result<ConnectedAccount> {
    let db = state.db.lock().map_err(db_err)?;
    db_accounts::check_and_update_health(&db, id)
        .await
        .map_err(db_err)
}

#[tauri::command]
pub async fn start_gmail_oauth(_state: AppStateRef<'_>) -> Result<String> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| "GOOGLE_CLIENT_ID not configured".to_string())?;
    let auth_url = mailscope_connector_gmail::build_auth_url(&client_id);
    Ok(auth_url)
}

#[tauri::command]
pub async fn complete_gmail_oauth(
    state: AppStateRef<'_>,
    code: String,
    state_param: String,
) -> Result<ConnectedAccount> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| "GOOGLE_CLIENT_ID not configured".to_string())?;
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
        .map_err(|_| "GOOGLE_CLIENT_SECRET not configured".to_string())?;

    let token = mailscope_connector_gmail::exchange_code(&client_id, &client_secret, &code)
        .await
        .map_err(db_err)?;
    let profile = mailscope_connector_gmail::get_profile(&token.access_token)
        .await
        .map_err(db_err)?;

    let db = state.db.lock().map_err(db_err)?;
    db_accounts::add_oauth_account(
        &db,
        "gmail",
        &profile.email,
        &profile.email,
        "oauth",
        &token.refresh_token.unwrap_or_default(),
    )
    .map_err(db_err)
}

#[tauri::command]
pub async fn start_microsoft_oauth(_state: AppStateRef<'_>) -> Result<String> {
    let client_id = std::env::var("MICROSOFT_CLIENT_ID")
        .map_err(|_| "MICROSOFT_CLIENT_ID not configured".to_string())?;
    let auth_url = mailscope_connector_microsoft::build_auth_url(&client_id);
    Ok(auth_url)
}

#[tauri::command]
pub async fn complete_microsoft_oauth(
    state: AppStateRef<'_>,
    code: String,
    _state_param: String,
) -> Result<ConnectedAccount> {
    let client_id = std::env::var("MICROSOFT_CLIENT_ID")
        .map_err(|_| "MICROSOFT_CLIENT_ID not configured".to_string())?;
    let client_secret = std::env::var("MICROSOFT_CLIENT_SECRET")
        .map_err(|_| "MICROSOFT_CLIENT_SECRET not configured".to_string())?;

    let token =
        mailscope_connector_microsoft::exchange_code(&client_id, &client_secret, &code)
            .await
            .map_err(db_err)?;
    let profile = mailscope_connector_microsoft::get_profile(&token.access_token)
        .await
        .map_err(db_err)?;

    let db = state.db.lock().map_err(db_err)?;
    db_accounts::add_oauth_account(
        &db,
        "microsoft",
        &profile.display_name,
        &profile.mail,
        "oauth",
        &token.refresh_token.unwrap_or_default(),
    )
    .map_err(db_err)
}

// ── Seed groups ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_seed_groups(state: AppStateRef<'_>) -> Result<Vec<SeedGroup>> {
    let db = state.db.lock().map_err(db_err)?;
    db_accounts::list_seed_groups(&db).map_err(db_err)
}

#[tauri::command]
pub async fn create_seed_group(
    state: AppStateRef<'_>,
    name: String,
    description: Option<String>,
) -> Result<SeedGroup> {
    let db = state.db.lock().map_err(db_err)?;
    db_accounts::create_seed_group(&db, &name, description.as_deref()).map_err(db_err)
}

#[tauri::command]
pub async fn delete_seed_group(state: AppStateRef<'_>, id: i64) -> Result<()> {
    let db = state.db.lock().map_err(db_err)?;
    db_accounts::delete_seed_group(&db, id).map_err(db_err)
}

#[tauri::command]
pub async fn add_account_to_group(
    state: AppStateRef<'_>,
    seed_group_id: i64,
    account_id: i64,
) -> Result<SeedGroupMember> {
    let db = state.db.lock().map_err(db_err)?;
    db_accounts::add_account_to_group(&db, seed_group_id, account_id).map_err(db_err)
}

#[tauri::command]
pub async fn remove_account_from_group(
    state: AppStateRef<'_>,
    seed_group_id: i64,
    account_id: i64,
) -> Result<()> {
    let db = state.db.lock().map_err(db_err)?;
    db_accounts::remove_account_from_group(&db, seed_group_id, account_id).map_err(db_err)
}

#[tauri::command]
pub async fn get_seed_group_members(
    state: AppStateRef<'_>,
    seed_group_id: i64,
) -> Result<Vec<SeedGroupMember>> {
    let db = state.db.lock().map_err(db_err)?;
    db_accounts::get_seed_group_members(&db, seed_group_id).map_err(db_err)
}
