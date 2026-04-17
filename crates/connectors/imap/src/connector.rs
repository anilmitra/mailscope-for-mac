use async_trait::async_trait;
use futures::StreamExt;
use tokio_util::compat::TokioAsyncReadCompatExt;
use mailscope_core::{
    connector::{ConnectorError, ConnectorResult, MailboxConnector, MailboxFolder, SpecialUse},
    models::{FindCriteria, MatchedMessage},
};
use std::collections::HashMap;
use tracing::debug;

pub struct ImapConnector {
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl ImapConnector {
    pub fn new(host: String, port: u16, username: String, password: String) -> Self {
        Self {
            host,
            port,
            username,
            password,
        }
    }

    fn special_use_from_name(name: &str) -> Option<SpecialUse> {
        let lower = name.to_lowercase();
        if lower == "inbox" {
            return Some(SpecialUse::Inbox);
        }
        if lower.contains("spam") || lower.contains("junk") {
            return Some(SpecialUse::Spam);
        }
        if lower.contains("trash") || lower.contains("deleted") {
            return Some(SpecialUse::Trash);
        }
        if lower.contains("sent") {
            return Some(SpecialUse::Sent);
        }
        if lower.contains("draft") {
            return Some(SpecialUse::Drafts);
        }
        None
    }
}

#[async_trait]
impl MailboxConnector for ImapConnector {
    fn provider(&self) -> &str {
        "imap"
    }

    async fn connect(&self) -> ConnectorResult<()> {
        let tcp = tokio::net::TcpStream::connect(format!("{}:{}", self.host, self.port))
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;
        let tcp = tcp.compat();

        let tls = async_native_tls::TlsConnector::new();
        let tls_stream = tls
            .connect(&self.host, tcp)
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;

        let client = async_imap::Client::new(tls_stream);
        client
            .login(&self.username, &self.password)
            .await
            .map_err(|(e, _)| ConnectorError::AuthError(e.to_string()))?;

        Ok(())
    }

    async fn list_folders(&self) -> ConnectorResult<Vec<MailboxFolder>> {
        let tcp = tokio::net::TcpStream::connect(format!("{}:{}", self.host, self.port))
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;
        let tcp = tcp.compat();

        let tls = async_native_tls::TlsConnector::new();
        let tls_stream = tls
            .connect(&self.host, tcp)
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;

        let client = async_imap::Client::new(tls_stream);
        let mut session = client
            .login(&self.username, &self.password)
            .await
            .map_err(|(e, _)| ConnectorError::AuthError(e.to_string()))?;

        let names = session
            .list(None, Some("*"))
            .await
            .map_err(|e| ConnectorError::ProviderError(e.to_string()))?;

        let mut folders = vec![];
        let mut names = names;
        while let Some(name_result) = names.next().await {
            let name = name_result.map_err(|e| ConnectorError::ProviderError(e.to_string()))?;
            let n = name.name().to_string();
            let su = Self::special_use_from_name(&n);
            folders.push(MailboxFolder {
                id: n.clone(),
                name: n,
                special_use: su,
            });
        }
        drop(names);

        session
            .logout()
            .await
            .map_err(|e| ConnectorError::ProviderError(e.to_string()))?;

        Ok(folders)
    }

    async fn find_message(
        &self,
        criteria: &FindCriteria,
    ) -> ConnectorResult<Option<MatchedMessage>> {
        debug!("IMAP search for {}", criteria.subject_token);

        let tcp = tokio::net::TcpStream::connect(format!("{}:{}", self.host, self.port))
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;
        let tcp = tcp.compat();

        let tls = async_native_tls::TlsConnector::new();
        let tls_stream = tls
            .connect(&self.host, tcp)
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;

        let client = async_imap::Client::new(tls_stream);
        let mut session = client
            .login(&self.username, &self.password)
            .await
            .map_err(|(e, _)| ConnectorError::AuthError(e.to_string()))?;

        // Check Inbox first, then Spam/Junk
        let folders_to_check = ["INBOX", "Junk", "Spam", "Junk Email", "[Gmail]/Spam"];
        for folder_name in &folders_to_check {
            if session.select(folder_name).await.is_err() {
                continue;
            }

            let query = format!("SUBJECT \"{}\"", criteria.subject_token);
            let uids = match session.uid_search(query).await {
                Ok(u) => u,
                Err(_) => continue,
            };

            if let Some(&uid) = uids.iter().next() {
                let messages = session
                    .uid_fetch(uid.to_string(), "(UID ENVELOPE RFC822.HEADER)")
                    .await
                    .map_err(|e| ConnectorError::ProviderError(e.to_string()))?;

                let mut messages = messages;
                let msg_opt = messages.next().await;
                drop(messages);

                if let Some(msg_result) = msg_opt {
                    let msg = msg_result.map_err(|e| ConnectorError::ProviderError(e.to_string()))?;
                    let headers = parse_raw_headers(msg.header().unwrap_or_default());
                    session.logout().await.ok();
                    return Ok(Some(MatchedMessage {
                        remote_id: uid.to_string(),
                        folder: folder_name.to_string(),
                        labels: vec![],
                        headers,
                        received_at: None,
                    }));
                }
            }
        }

        session.logout().await.ok();
        Ok(None)
    }

    async fn fetch_headers(
        &self,
        remote_message_id: &str,
    ) -> ConnectorResult<HashMap<String, String>> {
        let tcp = tokio::net::TcpStream::connect(format!("{}:{}", self.host, self.port))
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;
        let tcp = tcp.compat();

        let tls = async_native_tls::TlsConnector::new();
        let tls_stream = tls
            .connect(&self.host, tcp)
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;

        let client = async_imap::Client::new(tls_stream);
        let mut session = client
            .login(&self.username, &self.password)
            .await
            .map_err(|(e, _)| ConnectorError::AuthError(e.to_string()))?;

        session
            .select("INBOX")
            .await
            .map_err(|e| ConnectorError::ProviderError(e.to_string()))?;

        let mut messages = session
            .uid_fetch(remote_message_id, "RFC822.HEADER")
            .await
            .map_err(|e| ConnectorError::ProviderError(e.to_string()))?;

        let msg_opt = messages.next().await;
        drop(messages);

        let headers = if let Some(msg_result) = msg_opt {
            msg_result
                .ok()
                .and_then(|m| m.header().map(parse_raw_headers))
                .unwrap_or_default()
        } else {
            HashMap::new()
        };

        session.logout().await.ok();
        Ok(headers)
    }

    async fn refresh_token_if_needed(&self) -> ConnectorResult<()> {
        Ok(())
    }
}

fn parse_raw_headers(raw: &[u8]) -> HashMap<String, String> {
    let text = String::from_utf8_lossy(raw);
    let mut headers = HashMap::new();
    let mut current_key: Option<String> = None;
    let mut current_val = String::new();

    for line in text.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            // Continuation
            current_val.push(' ');
            current_val.push_str(line.trim());
        } else if let Some(colon) = line.find(':') {
            if let Some(key) = current_key.take() {
                headers.insert(key, current_val.trim().to_string());
            }
            current_key = Some(line[..colon].to_string());
            current_val = line[colon + 1..].to_string();
        }
    }
    if let Some(key) = current_key {
        headers.insert(key, current_val.trim().to_string());
    }
    headers
}
