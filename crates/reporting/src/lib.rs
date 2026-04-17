use anyhow::Result;
use mailscope_core::models::{DiagnosticIssue, TestResult, TestRun};

pub fn export_csv(run: &TestRun, results: &[TestResult]) -> Result<String> {
    let mut out = String::new();

    // Header
    out.push_str(
        "run_uuid,subject_template,subject_token,account_id,provider,placement,\
         raw_folder,matched_at,delivery_latency_ms,spf_result,dkim_result,dmarc_result\n",
    );

    for r in results {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}\n",
            run.run_uuid,
            csv_escape(&run.subject_template),
            run.subject_token,
            r.connected_account_id,
            r.provider,
            r.placement,
            r.raw_folder.as_deref().unwrap_or(""),
            r.matched_at.as_deref().unwrap_or(""),
            r.delivery_latency_ms.map(|v| v.to_string()).unwrap_or_default(),
            r.spf_result.as_deref().unwrap_or(""),
            r.dkim_result.as_deref().unwrap_or(""),
            r.dmarc_result.as_deref().unwrap_or(""),
        ));
    }

    Ok(out)
}

pub fn export_json(
    run: &TestRun,
    results: &[TestResult],
    diagnostics: &[DiagnosticIssue],
) -> Result<String> {
    let payload = serde_json::json!({
        "run": run,
        "results": results,
        "diagnostics": diagnostics,
        "exported_at": chrono::Utc::now().to_rfc3339(),
    });
    Ok(serde_json::to_string_pretty(&payload)?)
}

pub fn export_html_report(run: &TestRun, results: &[TestResult], diagnostics: &[DiagnosticIssue]) -> String {
    let inbox = results.iter().filter(|r| r.placement == "inbox").count();
    let spam = results.iter().filter(|r| r.placement == "spam").count();
    let junk = results.iter().filter(|r| r.placement == "junk").count();
    let missing = results.iter().filter(|r| r.placement == "missing").count();
    let total = results.len();
    let inbox_pct = if total > 0 { inbox * 100 / total } else { 0 };

    let mut rows = String::new();
    for r in results {
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            r.connected_account_id,
            r.provider,
            r.placement,
            r.spf_result.as_deref().unwrap_or("-"),
            r.dkim_result.as_deref().unwrap_or("-"),
        ));
    }

    let mut diag_rows = String::new();
    for d in diagnostics {
        diag_rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            d.severity, d.check_type, d.title, d.recommendation
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8"><title>MailScope Report: {subject}</title>
<style>
  body {{ font-family: system-ui, sans-serif; max-width: 900px; margin: 40px auto; color: #1a1a1a; }}
  h1 {{ font-size: 1.4rem; }} h2 {{ font-size: 1rem; color: #555; margin-top: 2rem; }}
  .summary {{ display: flex; gap: 24px; margin: 1rem 0; }}
  .stat {{ background: #f5f5f5; border-radius: 8px; padding: 16px 24px; text-align: center; }}
  .stat .value {{ font-size: 2rem; font-weight: 700; }}
  .stat .label {{ font-size: 0.8rem; color: #777; }}
  table {{ width: 100%; border-collapse: collapse; font-size: 0.85rem; }}
  th {{ background: #f0f0f0; padding: 8px 12px; text-align: left; }}
  td {{ padding: 8px 12px; border-bottom: 1px solid #eee; }}
</style>
</head>
<body>
<h1>MailScope Inbox Placement Report</h1>
<p><strong>Subject:</strong> {subject}</p>
<p><strong>Token:</strong> <code>{token}</code> &nbsp;|&nbsp; <strong>Run ID:</strong> {run_uuid}</p>

<div class="summary">
  <div class="stat"><div class="value" style="color:#2ecc71">{inbox_pct}%</div><div class="label">Inbox Rate</div></div>
  <div class="stat"><div class="value">{inbox}</div><div class="label">Inbox</div></div>
  <div class="stat"><div class="value" style="color:#e74c3c">{spam}</div><div class="label">Spam</div></div>
  <div class="stat"><div class="value" style="color:#e67e22">{junk}</div><div class="label">Junk</div></div>
  <div class="stat"><div class="value" style="color:#999">{missing}</div><div class="label">Missing</div></div>
</div>

<h2>Per-Seed Results</h2>
<table>
  <tr><th>Account</th><th>Provider</th><th>Placement</th><th>SPF</th><th>DKIM</th></tr>
  {rows}
</table>

<h2>Diagnostic Issues</h2>
{diag_section}
</body>
</html>"#,
        subject = html_escape(&run.subject_template),
        token = run.subject_token,
        run_uuid = run.run_uuid,
        inbox_pct = inbox_pct,
        inbox = inbox,
        spam = spam,
        junk = junk,
        missing = missing,
        rows = rows,
        diag_section = if diagnostics.is_empty() {
            "<p>No issues detected.</p>".to_string()
        } else {
            format!(
                "<table><tr><th>Severity</th><th>Type</th><th>Title</th><th>Fix</th></tr>{}</table>",
                diag_rows
            )
        }
    )
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
