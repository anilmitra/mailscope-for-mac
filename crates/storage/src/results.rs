use anyhow::{Context, Result};
use mailscope_core::models::{DiagnosticIssue, PlacementSummary, TestResult};

use crate::db::Database;

pub fn get_run_results(db: &Database, run_id: i64) -> Result<Vec<TestResult>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT r.id, r.test_run_id, r.connected_account_id, r.provider, r.placement,
                r.raw_folder, r.message_id_remote, r.matched_at, r.delivery_latency_ms,
                r.spf_result, r.dkim_result, r.dmarc_result, r.auth_summary,
                r.headers_json, r.notes_json
         FROM test_results r WHERE r.test_run_id = ?1 ORDER BY r.id",
    )?;
    let rows = stmt.query_map([run_id], map_result)?;
    rows.collect::<Result<Vec<_>, _>>().context("get_run_results")
}

pub fn upsert_result(
    db: &Database,
    run_id: i64,
    account_id: i64,
    provider: &str,
    placement: &str,
    raw_folder: Option<&str>,
    message_id_remote: Option<&str>,
    delivery_latency_ms: Option<i64>,
    spf_result: Option<&str>,
    dkim_result: Option<&str>,
    dmarc_result: Option<&str>,
    auth_summary: Option<&str>,
    headers_json: Option<&str>,
) -> Result<()> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT INTO test_results
            (test_run_id, connected_account_id, provider, placement, raw_folder,
             message_id_remote, matched_at, delivery_latency_ms,
             spf_result, dkim_result, dmarc_result, auth_summary, headers_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), ?7, ?8, ?9, ?10, ?11, ?12)
         ON CONFLICT(test_run_id, connected_account_id) DO UPDATE SET
             placement = excluded.placement,
             raw_folder = excluded.raw_folder,
             message_id_remote = excluded.message_id_remote,
             matched_at = excluded.matched_at,
             delivery_latency_ms = excluded.delivery_latency_ms,
             spf_result = excluded.spf_result,
             dkim_result = excluded.dkim_result,
             dmarc_result = excluded.dmarc_result,
             auth_summary = excluded.auth_summary,
             headers_json = excluded.headers_json",
        rusqlite::params![
            run_id,
            account_id,
            provider,
            placement,
            raw_folder,
            message_id_remote,
            delivery_latency_ms,
            spf_result,
            dkim_result,
            dmarc_result,
            auth_summary,
            headers_json
        ],
    )?;
    Ok(())
}

pub fn insert_missing_results_for_run(db: &Database, run_id: i64) -> Result<()> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT OR IGNORE INTO test_results (test_run_id, connected_account_id, provider, placement)
         SELECT ?1, m.connected_account_id, a.provider, 'missing'
         FROM test_runs r
         JOIN seed_group_members m ON m.seed_group_id = r.seed_group_id
         JOIN connected_accounts a ON a.id = m.connected_account_id
         WHERE r.id = ?1",
        [run_id],
    )?;
    Ok(())
}

pub fn get_run_diagnostics(db: &Database, run_id: i64) -> Result<Vec<DiagnosticIssue>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, test_run_id, check_type, severity, title, details, recommendation
         FROM diagnostics WHERE test_run_id = ?1 ORDER BY
         CASE severity WHEN 'critical' THEN 0 WHEN 'error' THEN 1 WHEN 'warning' THEN 2 ELSE 3 END",
    )?;
    let rows = stmt.query_map([run_id], |row| {
        Ok(DiagnosticIssue {
            id: row.get(0)?,
            test_run_id: row.get(1)?,
            check_type: row.get(2)?,
            severity: row.get(3)?,
            title: row.get(4)?,
            details: row.get(5)?,
            recommendation: row.get(6)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().context("get_run_diagnostics")
}

pub fn insert_diagnostic(
    db: &Database,
    run_id: i64,
    check_type: &str,
    severity: &str,
    title: &str,
    details: &str,
    recommendation: &str,
) -> Result<()> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT INTO diagnostics (test_run_id, check_type, severity, title, details, recommendation)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![run_id, check_type, severity, title, details, recommendation],
    )?;
    Ok(())
}

pub fn get_provider_summary(db: &Database, run_id: i64) -> Result<Vec<PlacementSummary>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT provider,
                SUM(CASE WHEN placement='inbox'      THEN 1 ELSE 0 END) AS inbox,
                SUM(CASE WHEN placement='spam'       THEN 1 ELSE 0 END) AS spam,
                SUM(CASE WHEN placement='junk'       THEN 1 ELSE 0 END) AS junk,
                SUM(CASE WHEN placement='promotions' THEN 1 ELSE 0 END) AS promotions,
                SUM(CASE WHEN placement='social'     THEN 1 ELSE 0 END) AS social,
                SUM(CASE WHEN placement='missing'    THEN 1 ELSE 0 END) AS missing,
                SUM(CASE WHEN placement NOT IN ('inbox','spam','junk','promotions','social','missing') THEN 1 ELSE 0 END) AS other,
                COUNT(*) AS total
         FROM test_results WHERE test_run_id = ?1 GROUP BY provider",
    )?;
    let rows = stmt.query_map([run_id], |row| {
        let inbox: i64 = row.get(1)?;
        let total: i64 = row.get(8)?;
        let inbox_rate = if total > 0 { inbox as f64 / total as f64 * 100.0 } else { 0.0 };
        Ok(PlacementSummary {
            provider: row.get(0)?,
            inbox,
            spam: row.get(2)?,
            junk: row.get(3)?,
            promotions: row.get(4)?,
            social: row.get(5)?,
            missing: row.get(6)?,
            other: row.get(7)?,
            total,
            inbox_rate,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().context("get_provider_summary")
}

fn map_result(row: &rusqlite::Row<'_>) -> rusqlite::Result<TestResult> {
    let headers_raw: Option<String> = row.get(13)?;
    let notes_raw: Option<String> = row.get(14)?;

    let headers_json = headers_raw
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok());
    let notes_json = notes_raw
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok());

    Ok(TestResult {
        id: row.get(0)?,
        test_run_id: row.get(1)?,
        connected_account_id: row.get(2)?,
        provider: row.get(3)?,
        placement: row.get(4)?,
        raw_folder: row.get(5)?,
        message_id_remote: row.get(6)?,
        matched_at: row.get(7)?,
        delivery_latency_ms: row.get(8)?,
        spf_result: row.get(9)?,
        dkim_result: row.get(10)?,
        dmarc_result: row.get(11)?,
        auth_summary: row.get(12)?,
        headers_json,
        notes_json,
        account: None,
    })
}
