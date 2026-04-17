use anyhow::{Context, Result};
use mailscope_core::models::{SendingProfile, TestRun};

use crate::db::Database;

// ── Sending profiles ──────────────────────────────────────────────────────────

pub fn list_sending_profiles(db: &Database) -> Result<Vec<SendingProfile>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, from_name, from_email, reply_to, sending_mode,
                smtp_config_ref, notes, created_at
         FROM sending_profiles ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], map_profile)?;
    rows.collect::<Result<Vec<_>, _>>().context("list_sending_profiles")
}

pub fn create_sending_profile(
    db: &Database,
    name: &str,
    from_name: &str,
    from_email: &str,
    reply_to: Option<&str>,
    notes: Option<&str>,
) -> Result<SendingProfile> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT INTO sending_profiles (name, from_name, from_email, reply_to, notes)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![name, from_name, from_email, reply_to, notes],
    )?;
    let id = conn.last_insert_rowid();
    drop(conn);
    get_sending_profile(db, id)
}

pub fn get_sending_profile(db: &Database, id: i64) -> Result<SendingProfile> {
    let conn = db.conn()?;
    conn.query_row(
        "SELECT id, name, from_name, from_email, reply_to, sending_mode,
                smtp_config_ref, notes, created_at
         FROM sending_profiles WHERE id = ?1",
        [id],
        map_profile,
    )
    .context("get_sending_profile")
}

pub fn delete_sending_profile(db: &Database, id: i64) -> Result<()> {
    let conn = db.conn()?;
    conn.execute("DELETE FROM sending_profiles WHERE id = ?1", [id])?;
    Ok(())
}

fn map_profile(row: &rusqlite::Row<'_>) -> rusqlite::Result<SendingProfile> {
    Ok(SendingProfile {
        id: row.get(0)?,
        name: row.get(1)?,
        from_name: row.get(2)?,
        from_email: row.get(3)?,
        reply_to: row.get(4)?,
        sending_mode: row.get(5)?,
        smtp_config_ref: row.get(6)?,
        notes: row.get(7)?,
        created_at: row.get(8)?,
    })
}

// ── Test runs ─────────────────────────────────────────────────────────────────

pub fn list_runs(db: &Database, limit: i64) -> Result<Vec<TestRun>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, run_uuid, sending_profile_id, seed_group_id, status,
                subject_template, body_text, body_html, subject_token, body_token,
                started_at, completed_at, inbox_rate, spam_rate, missing_rate
         FROM test_runs ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit], map_run)?;
    rows.collect::<Result<Vec<_>, _>>().context("list_runs")
}

pub fn get_run(db: &Database, id: i64) -> Result<TestRun> {
    let conn = db.conn()?;
    conn.query_row(
        "SELECT id, run_uuid, sending_profile_id, seed_group_id, status,
                subject_template, body_text, body_html, subject_token, body_token,
                started_at, completed_at, inbox_rate, spam_rate, missing_rate
         FROM test_runs WHERE id = ?1",
        [id],
        map_run,
    )
    .context("get_run")
}

pub fn get_last_run(db: &Database) -> Result<Option<TestRun>> {
    let conn = db.conn()?;
    let result = conn.query_row(
        "SELECT id, run_uuid, sending_profile_id, seed_group_id, status,
                subject_template, body_text, body_html, subject_token, body_token,
                started_at, completed_at, inbox_rate, spam_rate, missing_rate
         FROM test_runs WHERE status = 'complete' ORDER BY completed_at DESC LIMIT 1",
        [],
        map_run,
    );
    match result {
        Ok(run) => Ok(Some(run)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn create_run(
    db: &Database,
    sending_profile_id: i64,
    seed_group_id: i64,
    subject_template: &str,
    body_text: &str,
    body_html: Option<&str>,
    subject_token: &str,
    body_token: &str,
) -> Result<TestRun> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT INTO test_runs
            (sending_profile_id, seed_group_id, subject_template, body_text, body_html,
             subject_token, body_token)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            sending_profile_id,
            seed_group_id,
            subject_template,
            body_text,
            body_html,
            subject_token,
            body_token
        ],
    )?;
    let id = conn.last_insert_rowid();
    drop(conn);
    get_run(db, id)
}

pub fn start_run(db: &Database, id: i64) -> Result<TestRun> {
    let conn = db.conn()?;
    conn.execute(
        "UPDATE test_runs SET status = 'running', started_at = datetime('now') WHERE id = ?1",
        [id],
    )?;
    drop(conn);
    get_run(db, id)
}

pub fn complete_run(db: &Database, id: i64, inbox_rate: f64, spam_rate: f64, missing_rate: f64) -> Result<()> {
    let conn = db.conn()?;
    conn.execute(
        "UPDATE test_runs SET status = 'complete', completed_at = datetime('now'),
                inbox_rate = ?2, spam_rate = ?3, missing_rate = ?4
         WHERE id = ?1",
        rusqlite::params![id, inbox_rate, spam_rate, missing_rate],
    )?;
    Ok(())
}

pub fn cancel_run(db: &Database, id: i64) -> Result<()> {
    let conn = db.conn()?;
    conn.execute(
        "UPDATE test_runs SET status = 'cancelled', completed_at = datetime('now') WHERE id = ?1 AND status = 'running'",
        [id],
    )?;
    Ok(())
}

fn map_run(row: &rusqlite::Row<'_>) -> rusqlite::Result<TestRun> {
    Ok(TestRun {
        id: row.get(0)?,
        run_uuid: row.get(1)?,
        sending_profile_id: row.get(2)?,
        seed_group_id: row.get(3)?,
        status: row.get(4)?,
        subject_template: row.get(5)?,
        body_text: row.get(6)?,
        body_html: row.get(7)?,
        subject_token: row.get(8)?,
        body_token: row.get(9)?,
        started_at: row.get(10)?,
        completed_at: row.get(11)?,
        inbox_rate: row.get(12)?,
        spam_rate: row.get(13)?,
        missing_rate: row.get(14)?,
    })
}
