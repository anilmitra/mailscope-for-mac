use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const REDIRECT_URI: &str = "http://127.0.0.1:43210/oauth/callback";
const SCOPE: &str = "https://graph.microsoft.com/Mail.Read User.Read offline_access";
const AUTHORITY: &str = "https://login.microsoftonline.com/common/oauth2/v2.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub mail: String,
}

pub fn build_auth_url(client_id: &str) -> String {
    format!(
        "{}/authorize\
        ?client_id={}\
        &redirect_uri={}\
        &response_type=code\
        &scope={}\
        &response_mode=query",
        AUTHORITY,
        urlencoding::encode(client_id),
        urlencoding::encode(REDIRECT_URI),
        urlencoding::encode(SCOPE),
    )
}

pub async fn exchange_code(client_id: &str, client_secret: &str, code: &str) -> Result<OAuthToken> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/token", AUTHORITY))
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", REDIRECT_URI),
        ])
        .send()
        .await
        .context("Microsoft token exchange failed")?;

    resp.json::<OAuthToken>().await.context("parsing Microsoft token response")
}

pub async fn fetch_user_profile(access_token: &str) -> Result<UserProfile> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://graph.microsoft.com/v1.0/me?$select=displayName,mail,userPrincipalName")
        .bearer_auth(access_token)
        .send()
        .await
        .context("Graph /me request failed")?;

    #[derive(Deserialize)]
    struct GraphUser {
        #[serde(rename = "displayName")]
        display_name: String,
        mail: Option<String>,
        #[serde(rename = "userPrincipalName")]
        user_principal_name: Option<String>,
    }

    let user: GraphUser = resp.json().await.context("parsing Graph user")?;
    let email = user
        .mail
        .or(user.user_principal_name)
        .unwrap_or_default();

    Ok(UserProfile {
        display_name: user.display_name,
        mail: email,
    })
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
