use crate::state::AppState;
use mailscope_core::{
    models::{SendingProfile, TestRun},
    orchestrator::{RunBackend, RunOrchestrator},
    token::generate_token,
};
use mailscope_storage::runs as db_runs;
use std::sync::Arc;
use tauri::State;

type Result<T> = std::result::Result<T, String>;
type AppStateRef<'a> = State<'a, Arc<AppState>>;

fn db_err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

// ── Sending profiles ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_sending_profiles(state: AppStateRef<'_>) -> Result<Vec<SendingProfile>> {
    let db = state.db.lock().map_err(db_err)?;
    db_runs::list_sending_profiles(&db).map_err(db_err)
}

#[derive(serde::Deserialize)]
pub struct CreateProfileParams {
    pub name: String,
    pub from_name: String,
    pub from_email: String,
    pub reply_to: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub async fn create_sending_profile(
    state: AppStateRef<'_>,
    params: CreateProfileParams,
) -> Result<SendingProfile> {
    let db = state.db.lock().map_err(db_err)?;
    db_runs::create_sending_profile(
        &db,
        &params.name,
        &params.from_name,
        &params.from_email,
        params.reply_to.as_deref(),
        params.notes.as_deref(),
    )
    .map_err(db_err)
}

#[tauri::command]
pub async fn delete_sending_profile(state: AppStateRef<'_>, id: i64) -> Result<()> {
    let db = state.db.lock().map_err(db_err)?;
    db_runs::delete_sending_profile(&db, id).map_err(db_err)
}

// ── Test runs ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_runs(state: AppStateRef<'_>, limit: Option<i64>) -> Result<Vec<TestRun>> {
    let db = state.db.lock().map_err(db_err)?;
    db_runs::list_runs(&db, limit.unwrap_or(20)).map_err(db_err)
}

#[tauri::command]
pub async fn get_run(state: AppStateRef<'_>, id: i64) -> Result<TestRun> {
    let db = state.db.lock().map_err(db_err)?;
    db_runs::get_run(&db, id).map_err(db_err)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunToken {
    pub subject_token: String,
    pub body_token: String,
    pub suggested_subject: String,
    pub suggested_body: String,
}

#[tauri::command]
pub async fn generate_run_token(
    _state: AppStateRef<'_>,
    _sending_profile_id: i64,
    subject_template: String,
) -> Result<RunToken> {
    let subject_token = generate_token();
    let body_token = generate_token();
    let suggested_subject = format!("{} [{}]", subject_template, subject_token);
    let suggested_body = format!(
        "This is a MailScope placement test.\r\n\r\nToken: {}\r\n",
        body_token
    );
    Ok(RunToken {
        subject_token,
        body_token,
        suggested_subject,
        suggested_body,
    })
}

#[derive(serde::Deserialize)]
pub struct CreateRunParams {
    pub sending_profile_id: i64,
    pub seed_group_id: i64,
    pub subject_template: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub subject_token: String,
    pub body_token: String,
}

#[tauri::command]
pub async fn create_run(
    state: AppStateRef<'_>,
    params: CreateRunParams,
) -> Result<TestRun> {
    let db = state.db.lock().map_err(db_err)?;
    db_runs::create_run(
        &db,
        params.sending_profile_id,
        params.seed_group_id,
        &params.subject_template,
        &params.body_text,
        params.body_html.as_deref(),
        &params.subject_token,
        &params.body_token,
    )
    .map_err(db_err)
}

#[tauri::command]
pub async fn start_run(state: AppStateRef<'_>, id: i64) -> Result<TestRun> {
    let db_guard = state.db.lock().map_err(db_err)?;
    let run = db_runs::start_run(&db_guard, id).map_err(db_err)?;
    drop(db_guard);

    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        let backend = AppRunBackend { state: state_clone };
        let orchestrator = RunOrchestrator::new(backend);
        if let Err(e) = orchestrator.run(id).await {
            tracing::error!("Run {} failed: {}", id, e);
        }
    });

    Ok(run)
}

#[tauri::command]
pub async fn cancel_run(state: AppStateRef<'_>, id: i64) -> Result<()> {
    let db = state.db.lock().map_err(db_err)?;
    db_runs::cancel_run(&db, id).map_err(db_err)
}

// ── Orchestrator backend ──────────────────────────────────────────────────────

struct AppRunBackend {
    state: Arc<AppState>,
}

#[async_trait::async_trait]
impl RunBackend for AppRunBackend {
    async fn poll_once(&self, run_id: i64) -> anyhow::Result<()> {
        // TODO: iterate seed group members, call the appropriate connector,
        //       classify placement via PlacementClassifier, store via storage.
        let _ = run_id;
        Ok(())
    }

    async fn all_matched(&self, run_id: i64) -> bool {
        let Ok(db) = self.state.db.lock() else {
            return false;
        };
        let Ok(results) = mailscope_storage::results::get_run_results(&db, run_id) else {
            return false;
        };
        !results.is_empty() && results.iter().all(|r| r.placement != "missing")
    }

    async fn is_cancelled(&self, run_id: i64) -> bool {
        let Ok(db) = self.state.db.lock() else {
            return false;
        };
        matches!(
            mailscope_storage::runs::get_run(&db, run_id),
            Ok(r) if r.status == "cancelled"
        )
    }

    async fn finalize(&self, run_id: i64) -> anyhow::Result<()> {
        let db = self.state.db.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        mailscope_storage::results::insert_missing_results_for_run(&db, run_id)?;

        let results = mailscope_storage::results::get_run_results(&db, run_id)?;
        let total = results.len() as f64;
        let (inbox, spam, missing) = if total > 0.0 {
            let i = results.iter().filter(|r| r.placement == "inbox").count() as f64;
            let s = results.iter().filter(|r| r.placement == "spam" || r.placement == "junk").count() as f64;
            let m = results.iter().filter(|r| r.placement == "missing").count() as f64;
            (i / total, s / total, m / total)
        } else {
            (0.0, 0.0, 1.0)
        };

        mailscope_storage::runs::complete_run(&db, run_id, inbox, spam, missing)?;

        // Run diagnostics
        let run = mailscope_storage::runs::get_run(&db, run_id)?;
        let issues = mailscope_diagnostics::run_diagnostics(&run, &results);
        for issue in &issues {
            mailscope_storage::results::insert_diagnostic(
                &db,
                run_id,
                &issue.check_type,
                &issue.severity,
                &issue.title,
                &issue.details,
                &issue.recommendation,
            )?;
        }
        Ok(())
    }
}
