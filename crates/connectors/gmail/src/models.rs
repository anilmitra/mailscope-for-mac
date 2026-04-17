use anyhow::{Context, Result};
use serde::Deserialize;

use crate::auth::UserProfile;

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    email: String,
    name: Option<String>,
}

pub async fn get_profile(access_token: &str) -> Result<UserProfile> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .context("userinfo request failed")?;

    let info: GoogleUserInfo = resp.json().await.context("parsing userinfo")?;
    Ok(UserProfile {
        email: info.email,
        name: info.name,
    })
}

#[derive(Debug, Deserialize)]
pub struct MessageListResponse {
    pub messages: Option<Vec<MessageRef>>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MessageRef {
    pub id: String,
    #[serde(rename = "threadId")]
    pub thread_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub id: String,
    #[serde(rename = "labelIds")]
    pub label_ids: Option<Vec<String>>,
    pub payload: Option<MessagePayload>,
    #[serde(rename = "internalDate")]
    pub internal_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MessagePayload {
    pub headers: Option<Vec<Header>>,
}

#[derive(Debug, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}
