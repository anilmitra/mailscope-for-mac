use mailscope_core::models::{TestResult, TestRun};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticResult {
    pub check_type: String,
    pub severity: String,
    pub title: String,
    pub details: String,
    pub recommendation: String,
}

pub fn run_diagnostics(run: &TestRun, results: &[TestResult]) -> Vec<DiagnosticResult> {
    let mut issues = vec![];

    if results.is_empty() {
        return issues;
    }

    check_dkim_dmarc_alignment(results, &mut issues);
    check_outlook_junk_concentration(results, &mut issues);
    check_missing_list_unsubscribe(results, &mut issues);
    check_slow_delivery(results, &mut issues);
    check_low_inbox_rate(results, &mut issues);
    check_spf_failures(results, &mut issues);

    let _ = run;
    issues
}

fn inbox_rate(results: &[TestResult]) -> f64 {
    if results.is_empty() {
        return 0.0;
    }
    let inbox = results.iter().filter(|r| r.placement == "inbox").count();
    inbox as f64 / results.len() as f64
}

fn provider_results<'a>(results: &'a [TestResult], provider: &str) -> Vec<&'a TestResult> {
    results.iter().filter(|r| r.provider == provider).collect()
}

fn check_dkim_dmarc_alignment(results: &[TestResult], issues: &mut Vec<DiagnosticResult>) {
    let misaligned: Vec<_> = results
        .iter()
        .filter(|r| {
            r.dkim_result.as_deref() == Some("pass")
                && r.dmarc_result.as_deref() == Some("fail")
        })
        .collect();

    if !misaligned.is_empty() {
        issues.push(DiagnosticResult {
            check_type: "dmarc".into(),
            severity: "error".into(),
            title: "DKIM passes but DMARC fails".into(),
            details: format!(
                "Seen on {} result(s). DKIM is signing but the d= domain is not aligned with the From header domain.",
                misaligned.len()
            ),
            recommendation: "Ensure the DKIM d= tag matches your From domain, or add the signing domain to your DMARC policy with relaxed alignment.".into(),
        });
    }
}

fn check_outlook_junk_concentration(results: &[TestResult], issues: &mut Vec<DiagnosticResult>) {
    let gmail_results = provider_results(results, "gmail");
    let ms_results = provider_results(results, "microsoft");

    if gmail_results.is_empty() || ms_results.is_empty() {
        return;
    }

    let gmail_inbox = gmail_results.iter().filter(|r| r.placement == "inbox").count() as f64
        / gmail_results.len() as f64;
    let ms_junk = ms_results
        .iter()
        .filter(|r| r.placement == "junk" || r.placement == "spam")
        .count() as f64
        / ms_results.len() as f64;

    if gmail_inbox > 0.8 && ms_junk > 0.5 {
        issues.push(DiagnosticResult {
            check_type: "content".into(),
            severity: "warning".into(),
            title: "Outlook/M365 junk concentration despite Gmail inbox".into(),
            details: format!(
                "Gmail inbox rate: {:.0}%, Microsoft junk rate: {:.0}%. This pattern often indicates Microsoft-specific reputation signals.",
                gmail_inbox * 100.0,
                ms_junk * 100.0
            ),
            recommendation: "Check Microsoft SNDS for complaint data, review link reputation, and ensure List-Unsubscribe is present. Consider submitting to Microsoft's JMRP.".into(),
        });
    }
}

fn check_missing_list_unsubscribe(results: &[TestResult], issues: &mut Vec<DiagnosticResult>) {
    let missing_unsub = results.iter().filter(|r| {
        r.headers_json
            .as_ref()
            .and_then(|h| {
                let obj = h.as_object()?;
                let has = obj.keys().any(|k| k.to_lowercase() == "list-unsubscribe");
                Some(!has)
            })
            .unwrap_or(false)
    });

    if missing_unsub.count() > 0 {
        issues.push(DiagnosticResult {
            check_type: "list_unsubscribe".into(),
            severity: "warning".into(),
            title: "Missing List-Unsubscribe header".into(),
            details: "Some received messages have no List-Unsubscribe header. Gmail and Yahoo require this for bulk senders.".into(),
            recommendation: "Add both mailto: and HTTPS List-Unsubscribe headers, and implement List-Unsubscribe-Post for one-click unsubscribe (RFC 8058).".into(),
        });
    }
}

fn check_slow_delivery(results: &[TestResult], issues: &mut Vec<DiagnosticResult>) {
    let latencies: Vec<i64> = results
        .iter()
        .filter_map(|r| r.delivery_latency_ms)
        .collect();

    if latencies.is_empty() {
        return;
    }

    let mut sorted = latencies.clone();
    sorted.sort_unstable();
    let median = sorted[sorted.len() / 2];

    if median > 300_000 {
        // > 5 minutes
        issues.push(DiagnosticResult {
            check_type: "latency".into(),
            severity: "warning".into(),
            title: "Slow median delivery time".into(),
            details: format!(
                "Median delivery latency is {:.1} minutes across matched seeds.",
                median as f64 / 60_000.0
            ),
            recommendation: "Inspect sending IP for deferrals, check sending IP reputation, and review provider-side throttling. Consider dedicated IPs if on shared infrastructure.".into(),
        });
    }
}

fn check_low_inbox_rate(results: &[TestResult], issues: &mut Vec<DiagnosticResult>) {
    let rate = inbox_rate(results);
    if rate < 0.5 && results.len() >= 3 {
        issues.push(DiagnosticResult {
            check_type: "content".into(),
            severity: "critical".into(),
            title: "Inbox rate below 50%".into(),
            details: format!(
                "Only {:.0}% of seeds received the message in inbox across all providers.",
                rate * 100.0
            ),
            recommendation: "Review SPF, DKIM, and DMARC alignment. Check sending domain and IP reputation on major blocklists. Inspect message content for spam signals.".into(),
        });
    }
}

fn check_spf_failures(results: &[TestResult], issues: &mut Vec<DiagnosticResult>) {
    let spf_failures = results
        .iter()
        .filter(|r| {
            matches!(
                r.spf_result.as_deref(),
                Some("fail") | Some("softfail") | Some("none")
            )
        })
        .count();

    if spf_failures > 0 {
        issues.push(DiagnosticResult {
            check_type: "spf".into(),
            severity: "error".into(),
            title: "SPF failures detected".into(),
            details: format!(
                "{} seed(s) received SPF fail, softfail, or none.",
                spf_failures
            ),
            recommendation: "Add the sending IP or include the sending service's SPF record in your domain's TXT record. Ensure you're not exceeding the 10 DNS lookup limit.".into(),
        });
    }
}
