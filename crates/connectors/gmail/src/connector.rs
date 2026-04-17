use anyhow::Context;
use async_trait::async_trait;
use mailscope_core::{
    connector::{ConnectorError, ConnectorResult, MailboxConnector, MailboxFolder, SpecialUse},
    models::{FindCriteria, MatchedMessage},
};
use std::collections::HashMap;
use tracing::debug;

use crate::models::{Message, MessageListResponse};

const BASE: &str = "https://www.googleapis.com/gmail/v1";

pub struct GmailConnector {
    access_token: String,
    account_email: String,
}

impl GmailConnector {
    pub fn new(access_token: String, account_email: String) -> Self {
        Self {
            access_token,
            account_email,
        }
    }

    fn client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }

    async fn search_messages(&self, query: &str) -> ConnectorResult<Vec<String>> {
        let url = format!(
            "{}/users/me/messages?q={}&maxResults=10",
            BASE,
            urlencoding::encode(query)
        );
        let resp = self
            .client()
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;

        if resp.status() == 429 {
            return Err(ConnectorError::RateLimited { retry_after_secs: 30 });
        }
        if !resp.status().is_success() {
            let status = resp.status();
            if status == 401 {
                return Err(ConnectorError::AuthError("token expired".into()));
            }
            return Err(ConnectorError::ProviderError(format!(
                "HTTP {}",
                status
            )));
        }

        let list: MessageListResponse = resp
            .json()
            .await
            .map_err(|e| ConnectorError::ProviderError(e.to_string()))?;

        Ok(list
            .messages
            .unwrap_or_default()
            .into_iter()
            .map(|m| m.id)
            .collect())
    }

    async fn get_message(&self, id: &str) -> ConnectorResult<Message> {
        let url = format!(
            "{}/users/me/messages/{}?format=metadata&metadataHeaders=Authentication-Results,Return-Path,Message-ID,List-Unsubscribe,List-Unsubscribe-Post",
            BASE, id
        );
        let resp = self
            .client()
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;

        if resp.status() == 404 {
            return Err(ConnectorError::NotFound);
        }

        resp.json::<Message>()
            .await
            .map_err(|e| ConnectorError::ProviderError(e.to_string()))
    }
}

#[async_trait]
impl MailboxConnector for GmailConnector {
    fn provider(&self) -> &str {
        "gmail"
    }

    async fn connect(&self) -> ConnectorResult<()> {
        // Verify token is valid by fetching profile
        let url = format!("{}/users/me/profile", BASE);
        let resp = self
            .client()
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;

        if resp.status() == 401 {
            return Err(ConnectorError::AuthError("invalid or expired token".into()));
        }
        Ok(())
    }

    async fn list_folders(&self) -> ConnectorResult<Vec<MailboxFolder>> {
        Ok(vec![
            MailboxFolder {
                id: "INBOX".into(),
                name: "Inbox".into(),
                special_use: Some(SpecialUse::Inbox),
            },
            MailboxFolder {
                id: "SPAM".into(),
                name: "Spam".into(),
                special_use: Some(SpecialUse::Spam),
            },
        ])
    }

    async fn find_message(
        &self,
        criteria: &FindCriteria,
    ) -> ConnectorResult<Option<MatchedMessage>> {
        let query = format!(
            "subject:{} after:{}",
            criteria.subject_token,
            criteria.since.timestamp()
        );

        debug!(
            "Gmail search for {} on {}",
            criteria.subject_token, self.account_email
        );

        let ids = self.search_messages(&query).await?;
        if ids.is_empty() {
            return Ok(None);
        }

        let msg = self.get_message(&ids[0]).await?;
        let labels = msg.label_ids.unwrap_or_default();
        let folder = labels.first().cloned().unwrap_or_else(|| "UNKNOWN".into());

        let headers: HashMap<String, String> = msg
            .payload
            .and_then(|p| p.headers)
            .unwrap_or_default()
            .into_iter()
            .map(|h| (h.name, h.value))
            .collect();

        let received_at = msg.internal_date.as_deref().and_then(|ts| {
            ts.parse::<i64>()
                .ok()
                .and_then(|ms| chrono::DateTime::from_timestamp(ms / 1000, 0))
        });

        Ok(Some(MatchedMessage {
            remote_id: ids[0].clone(),
            folder,
            labels,
            headers,
            received_at,
        }))
    }

    async fn fetch_headers(
        &self,
        remote_message_id: &str,
    ) -> ConnectorResult<HashMap<String, String>> {
        let msg = self.get_message(remote_message_id).await?;
        Ok(msg
            .payload
            .and_then(|p| p.headers)
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
