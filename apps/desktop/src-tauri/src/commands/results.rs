use crate::state::AppState;
use mailscope_core::models::{DiagnosticIssue, TestResult};
use mailscope_reporting::{export_csv, export_json};
use mailscope_storage::{results as db_results, runs as db_runs};
use std::sync::Arc;
use tauri::State;

type Result<T> = std::result::Result<T, String>;
type AppStateRef<'a> = State<'a, Arc<AppState>>;

fn db_err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[tauri::command]
pub async fn get_run_results(
    state: AppStateRef<'_>,
    run_id: i64,
) -> Result<Vec<TestResult>> {
    let db = state.db.lock().map_err(db_err)?;
    db_results::get_run_results(&db, run_id).map_err(db_err)
}

#[tauri::command]
pub async fn get_run_diagnostics(
    state: AppStateRef<'_>,
    run_id: i64,
) -> Result<Vec<DiagnosticIssue>> {
    let db = state.db.lock().map_err(db_err)?;
    db_results::get_run_diagnostics(&db, run_id).map_err(db_err)
}

#[tauri::command]
pub async fn export_run_csv(state: AppStateRef<'_>, run_id: i64) -> Result<String> {
    let (run, results) = {
        let db = state.db.lock().map_err(db_err)?;
        let run = db_runs::get_run(&db, run_id).map_err(db_err)?;
        let results = db_results::get_run_results(&db, run_id).map_err(db_err)?;
        (run, results)
    };
    export_csv(&run, &results).map_err(db_err)
}

#[tauri::command]
pub async fn export_run_json(state: AppStateRef<'_>, run_id: i64) -> Result<String> {
    let (run, results, diagnostics) = {
        let db = state.db.lock().map_err(db_err)?;
        let run = db_runs::get_run(&db, run_id).map_err(db_err)?;
        let results = db_results::get_run_results(&db, run_id).map_err(db_err)?;
        let diagnostics = db_results::get_run_diagnostics(&db, run_id).map_err(db_err)?;
        (run, results, diagnostics)
    };
    export_json(&run, &results, &diagnostics).map_err(db_err)
}

#[derive(serde::Serialize)]
pub struct DashboardStats {
    pub last_run: Option<mailscope_core::models::TestRun>,
    pub total_runs: i64,
    pub avg_inbox_rate: f64,
    pub avg_spam_rate: f64,
    pub avg_missing_rate: f64,
    pub recent_runs: Vec<mailscope_core::models::TestRun>,
    pub provider_summary: Vec<mailscope_core::models::PlacementSummary>,
}

#[tauri::command]
pub async fn get_dashboard_stats(state: AppStateRef<'_>) -> Result<DashboardStats> {
    let db = state.db.lock().map_err(db_err)?;
    let runs = db_runs::list_runs(&db, 50).map_err(db_err)?;
    let total_runs = runs.len() as i64;
    let (avg_inbox, avg_spam, avg_missing) = {
        let completed: Vec<_> = runs.iter().filter(|r| r.status == "complete").collect();
        if completed.is_empty() {
            (0.0, 0.0, 0.0)
        } else {
            let n = completed.len() as f64;
            (
                completed.iter().map(|r| r.inbox_rate.unwrap_or(0.0)).sum::<f64>() / n,
                completed.iter().map(|r| r.spam_rate.unwrap_or(0.0)).sum::<f64>() / n,
                completed.iter().map(|r| r.missing_rate.unwrap_or(0.0)).sum::<f64>() / n,
            )
        }
    };
    let recent_runs = runs.into_iter().take(10).collect();

    let last_run = db_runs::get_last_run(&db).map_err(db_err)?;
    let provider_summary = if let Some(ref run) = last_run {
        db_results::get_provider_summary(&db, run.id).map_err(db_err)?
    } else {
        vec![]
    };

    Ok(DashboardStats {
        last_run,
        total_runs,
        avg_inbox_rate: avg_inbox,
        avg_spam_rate: avg_spam,
        avg_missing_rate: avg_missing,
        recent_runs,
        provider_summary,
    })
}

#[tauri::command]
pub async fn check_domain(
    _state: AppStateRef<'_>,
    domain: String,
) -> Result<serde_json::Value> {
    let result = mailscope_diagnostics::check_domain(&domain)
        .await
        .map_err(db_err)?;
    Ok(serde_json::to_value(result).map_err(db_err)?)
}

#[tauri::command]
pub async fn lookup_provider_by_email(
    _state: AppStateRef<'_>,
    email: String,
) -> Result<String> {
    Ok(mailscope_diagnostics::lookup_provider(&email).to_string())
}
