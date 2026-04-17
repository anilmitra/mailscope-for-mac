use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const REDIRECT_URI: &str = "http://127.0.0.1:43210/oauth/callback";
const SCOPE: &str =
    "https://www.googleapis.com/auth/gmail.readonly https://www.googleapis.com/auth/userinfo.email";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub email: String,
    pub name: Option<String>,
}

pub fn build_auth_url(client_id: &str) -> String {
    format!(
        "https://accounts.google.com/o/oauth2/v2/auth\
        ?client_id={}\
        &redirect_uri={}\
        &response_type=code\
        &scope={}\
        &access_type=offline\
        &prompt=consent",
        urlencoding::encode(client_id),
        urlencoding::encode(REDIRECT_URI),
        urlencoding::encode(SCOPE),
    )
}

pub async fn exchange_code(client_id: &str, client_secret: &str, code: &str) -> Result<OAuthToken> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", REDIRECT_URI),
        ])
        .send()
        .await
        .context("token exchange request failed")?;

    resp.json::<OAuthToken>().await.context("parsing token response")
}

pub async fn refresh_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<OAuthToken> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .context("token refresh request failed")?;

    resp.json::<OAuthToken>().await.context("parsing refresh response")
}

// Simple percent-encode helper to avoid a full url dep just for this
mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
