mod auth;
mod connector;

pub use auth::{build_auth_url, exchange_code, OAuthToken, UserProfile};
pub use connector::MicrosoftConnector;

pub async fn get_profile(access_token: &str) -> anyhow::Result<UserProfile> {
    auth::fetch_user_profile(access_token).await
}
