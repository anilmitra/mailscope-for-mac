use crate::models::{FindCriteria, MatchedMessage};
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    #[error("authentication failed: {0}")]
    AuthError(String),
    #[error("network error: {0}")]
    NetworkError(String),
    #[error("rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("provider error: {0}")]
    ProviderError(String),
    #[error("message not found")]
    NotFound,
}

pub type ConnectorResult<T> = Result<T, ConnectorError>;

#[derive(Debug, Clone)]
pub struct MailboxFolder {
    pub id: String,
    pub name: String,
    pub special_use: Option<SpecialUse>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpecialUse {
    Inbox,
    Spam,
    Junk,
    Trash,
    Sent,
    Drafts,
    Archive,
    All,
}

#[async_trait]
pub trait MailboxConnector: Send + Sync {
    fn provider(&self) -> &str;

    async fn connect(&self) -> ConnectorResult<()>;

    async fn list_folders(&self) -> ConnectorResult<Vec<MailboxFolder>>;

    async fn find_message(
        &self,
        criteria: &FindCriteria,
    ) -> ConnectorResult<Option<MatchedMessage>>;

    async fn fetch_headers(
        &self,
        remote_message_id: &str,
    ) -> ConnectorResult<HashMap<String, String>>;

    async fn refresh_token_if_needed(&self) -> ConnectorResult<()>;
}
