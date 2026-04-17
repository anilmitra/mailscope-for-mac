use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DomainCheckResult {
    pub spf: Option<String>,
    pub dkim: Option<String>,
    pub dmarc: Option<String>,
    pub has_list_unsubscribe: bool,
    pub mx_records: Vec<String>,
}

pub async fn check_domain(domain: &str) -> Result<DomainCheckResult> {
    debug!("Checking domain: {}", domain);

    let mut result = DomainCheckResult::default();

    // Use Google Public DNS-over-HTTPS to avoid requiring system DNS resolver
    result.spf = lookup_txt(domain, "spf").await;
    result.dmarc = lookup_txt(&format!("_dmarc.{}", domain), "v=DMARC1").await;
    result.mx_records = lookup_mx(domain).await.unwrap_or_default();

    Ok(result)
}

async fn lookup_txt(name: &str, contains: &str) -> Option<String> {
    let url = format!(
        "https://dns.google/resolve?name={}&type=TXT",
        urlencoding::encode(name)
    );

    #[derive(Deserialize)]
    struct DohResponse {
        #[serde(rename = "Answer")]
        answer: Option<Vec<DohRecord>>,
    }

    #[derive(Deserialize)]
    struct DohRecord {
        data: String,
    }

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("Accept", "application/dns-json")
        .send()
        .await
        .ok()?;

    let doh: DohResponse = resp.json().await.ok()?;
    let records = doh.answer.unwrap_or_default();

    records
        .into_iter()
        .map(|r| r.data.trim_matches('"').to_string())
        .find(|s| s.to_lowercase().contains(&contains.to_lowercase()))
}

async fn lookup_mx(name: &str) -> Option<Vec<String>> {
    let url = format!(
        "https://dns.google/resolve?name={}&type=MX",
        urlencoding::encode(name)
    );

    #[derive(Deserialize)]
    struct DohResponse {
        #[serde(rename = "Answer")]
        answer: Option<Vec<DohRecord>>,
    }

    #[derive(Deserialize)]
    struct DohRecord {
        data: String,
    }

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("Accept", "application/dns-json")
        .send()
        .await
        .ok()?;

    let doh: DohResponse = resp.json().await.ok()?;
    Some(
        doh.answer
            .unwrap_or_default()
            .into_iter()
            .map(|r| r.data)
            .collect(),
    )
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
