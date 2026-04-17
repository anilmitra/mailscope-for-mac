use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedAccount {
    pub id: i64,
    pub provider: String,
    pub display_name: String,
    pub seed_email: String,
    pub auth_type: String,
    pub keychain_ref: String,
    pub status: String,
    pub last_seen_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedGroup {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub member_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedGroupMember {
    pub id: i64,
    pub seed_group_id: i64,
    pub connected_account_id: i64,
    pub account: Option<ConnectedAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendingProfile {
    pub id: i64,
    pub name: String,
    pub from_name: String,
    pub from_email: String,
    pub reply_to: Option<String>,
    pub sending_mode: String,
    pub smtp_config_ref: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TestRun {
    pub id: i64,
    pub run_uuid: String,
    pub sending_profile_id: i64,
    pub seed_group_id: i64,
    pub status: String,
    pub subject_template: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub subject_token: String,
    pub body_token: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub inbox_rate: Option<f64>,
    pub spam_rate: Option<f64>,
    pub missing_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    pub id: i64,
    pub test_run_id: i64,
    pub connected_account_id: i64,
    pub provider: String,
    pub placement: String,
    pub raw_folder: Option<String>,
    pub message_id_remote: Option<String>,
    pub matched_at: Option<String>,
    pub delivery_latency_ms: Option<i64>,
    pub spf_result: Option<String>,
    pub dkim_result: Option<String>,
    pub dmarc_result: Option<String>,
    pub auth_summary: Option<String>,
    pub headers_json: Option<serde_json::Value>,
    pub notes_json: Option<serde_json::Value>,
    pub account: Option<ConnectedAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticIssue {
    pub id: i64,
    pub test_run_id: i64,
    pub check_type: String,
    pub severity: String,
    pub title: String,
    pub details: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementSummary {
    pub provider: String,
    pub inbox: i64,
    pub spam: i64,
    pub junk: i64,
    pub promotions: i64,
    pub social: i64,
    pub missing: i64,
    pub other: i64,
    pub total: i64,
    pub inbox_rate: f64,
}

#[derive(Debug, Clone)]
pub struct FindCriteria {
    pub subject_token: String,
    pub body_token: String,
    pub since: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct MatchedMessage {
    pub remote_id: String,
    pub folder: String,
    pub labels: Vec<String>,
    pub headers: std::collections::HashMap<String, String>,
    pub received_at: Option<chrono::DateTime<chrono::Utc>>,
}
