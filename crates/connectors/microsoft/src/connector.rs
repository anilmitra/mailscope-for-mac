use async_trait::async_trait;
use mailscope_core::{
    connector::{ConnectorError, ConnectorResult, MailboxConnector, MailboxFolder, SpecialUse},
    models::{FindCriteria, MatchedMessage},
};
use serde::Deserialize;
use std::collections::HashMap;
use tracing::debug;

const GRAPH: &str = "https://graph.microsoft.com/v1.0";

pub struct MicrosoftConnector {
    access_token: String,
    account_email: String,
}

impl MicrosoftConnector {
    pub fn new(access_token: String, account_email: String) -> Self {
        Self {
            access_token,
            account_email,
        }
    }

    fn client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }
}

#[derive(Debug, Deserialize)]
struct MessageList {
    value: Vec<GraphMessage>,
}

#[derive(Debug, Deserialize)]
struct GraphMessage {
    id: String,
    subject: Option<String>,
    #[serde(rename = "parentFolderId")]
    parent_folder_id: Option<String>,
    #[serde(rename = "receivedDateTime")]
    received_date_time: Option<String>,
    #[serde(rename = "internetMessageHeaders")]
    internet_message_headers: Option<Vec<GraphHeader>>,
}

#[derive(Debug, Deserialize)]
struct GraphHeader {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct FolderList {
    value: Vec<GraphFolder>,
}

#[derive(Debug, Deserialize)]
struct GraphFolder {
    id: String,
    #[serde(rename = "displayName")]
    display_name: String,
}

fn folder_to_special_use(name: &str) -> Option<SpecialUse> {
    match name.to_lowercase().as_str() {
        "inbox" => Some(SpecialUse::Inbox),
        "junkemail" | "junk email" | "spam" => Some(SpecialUse::Junk),
        "deleteditems" | "deleted items" => Some(SpecialUse::Trash),
        "sentitems" | "sent items" | "sent" => Some(SpecialUse::Sent),
        "drafts" => Some(SpecialUse::Drafts),
        _ => None,
    }
}

#[async_trait]
impl MailboxConnector for MicrosoftConnector {
    fn provider(&self) -> &str {
        "microsoft"
    }

    async fn connect(&self) -> ConnectorResult<()> {
        let resp = self
            .client()
            .get(format!("{}/me/mailFolders/inbox", GRAPH))
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;

        if resp.status() == 401 {
            return Err(ConnectorError::AuthError("token expired".into()));
        }
        Ok(())
    }

    async fn list_folders(&self) -> ConnectorResult<Vec<MailboxFolder>> {
        let resp = self
            .client()
            .get(format!("{}/me/mailFolders?$top=50", GRAPH))
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;

        let list: FolderList = resp
            .json()
            .await
            .map_err(|e| ConnectorError::ProviderError(e.to_string()))?;

        Ok(list
            .value
            .into_iter()
            .map(|f| {
                let su = folder_to_special_use(&f.display_name);
                MailboxFolder {
                    id: f.id,
                    name: f.display_name.clone(),
                    special_use: su,
                }
            })
            .collect())
    }

    async fn find_message(
        &self,
        criteria: &FindCriteria,
    ) -> ConnectorResult<Option<MatchedMessage>> {
        let filter = format!(
            "contains(subject,'{}') and receivedDateTime ge {}",
            criteria.subject_token,
            criteria.since.format("%Y-%m-%dT%H:%M:%SZ"),
        );
        let url = format!(
            "{}/me/messages?$filter={}&$top=5&$select=id,subject,parentFolderId,receivedDateTime,internetMessageHeaders",
            GRAPH,
            urlencoding::encode(&filter)
        );

        debug!(
            "Microsoft search for {} on {}",
            criteria.subject_token, self.account_email
        );

        let resp = self
            .client()
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;

        if resp.status() == 401 {
            return Err(ConnectorError::AuthError("token expired".into()));
        }

        let list: MessageList = resp
            .json()
            .await
            .map_err(|e| ConnectorError::ProviderError(e.to_string()))?;

        let msg = match list.value.into_iter().next() {
            Some(m) => m,
            None => return Ok(None),
        };

        // Resolve folder name from id
        let folder_name = msg
            .parent_folder_id
            .as_deref()
            .and_then(|fid| {
                // Simple heuristic: common well-known folder IDs
                match fid {
                    s if s.starts_with("Inbox") || s.to_lowercase().contains("inbox") => {
                        Some("Inbox".to_string())
                    }
                    s if s.to_lowercase().contains("junk") => Some("JunkEmail".to_string()),
                    _ => None,
                }
            })
            .unwrap_or_else(|| {
                msg.parent_folder_id
                    .clone()
                    .unwrap_or_else(|| "Unknown".into())
            });

        let headers: HashMap<String, String> = msg
            .internet_message_headers
            .unwrap_or_default()
            .into_iter()
            .map(|h| (h.name, h.value))
            .collect();

        let received_at = msg
            .received_date_time
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        Ok(Some(MatchedMessage {
            remote_id: msg.id,
            folder: folder_name,
            labels: vec![],
            headers,
            received_at,
        }))
    }

    async fn fetch_headers(
        &self,
        remote_message_id: &str,
    ) -> ConnectorResult<HashMap<String, String>> {
        let url = format!(
            "{}/me/messages/{}?$select=internetMessageHeaders",
            GRAPH, remote_message_id
        );
        let resp = self
            .client()
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;

        let msg: GraphMessage = resp
            .json()
            .await
            .map_err(|e| ConnectorError::ProviderError(e.to_string()))?;

        Ok(msg
            .internet_message_headers
            .unwrap_or_default()
            .into_iter()
            .map(|h| (h.name, h.value))
            .collect())
    }

    async fn refresh_token_if_needed(&self) -> ConnectorResult<()> {
        Ok(())
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
