//! Ely.by Authentication

use crate::error::{AuthError, LarpError, LarpResult};
use crate::value::Account;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

const ELYBY_API_BASE: &str = "https://api.ely.by";
const ELYBY_AUTH_URL: &str = "https://auth.ely.by";
const ELYBY_TOKEN_URL: &str = "https://auth.ely.by/oauth/token";
const ELYBY_PROFILE_URL: &str = "https://api.ely.by/v1/profile";
const ELYBY_SKIN_URL: &str = "https://api.ely.by/v1/skins";

/// Ely.by auth client
pub struct ElyByAuthClient {
    client: Client,
    client_id: String,
    client_secret: String,
}

impl ElyByAuthClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            client_id: "minecraft".to_string(), // Default client ID
            client_secret: String::new(), // No secret for public client
        }
    }

    pub fn with_credentials(mut self, client_id: String, client_secret: String) -> Self {
        self.client_id = client_id;
        self.client_secret = client_secret;
        self
    }

    /// Start login with email/password
    pub async fn start_login(&self, email: &str, password: &str) -> LarpResult<ElyByLogin> {
        // Step 1: Get authorization code via OAuth2 password grant (if supported)
        // Ely.by uses a custom OAuth2 flow - we'll use the web flow
        let auth_url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope=profile+skins",
            ELYBY_AUTH_URL,
            urlencoding::encode(&self.client_id),
            urlencoding::encode("urn:ietf:wg:oauth:2.0:oob")
        );

        Ok(ElyByLogin {
            client: self.client.clone(),
            auth_url,
            email: email.to_string(),
            password: password.to_string(),
            state: Arc::new(RwLock::new(ElyByLoginState::Pending)),
            account: Arc::new(RwLock::new(None)),
            error: Arc::new(RwLock::new(None)),
        })
    }

    /// Exchange authorization code for token
    pub async fn exchange_code(&self, code: &str) -> LarpResult<ElyByTokenResponse> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", &self.client_id),
            ("redirect_uri", "urn:ietf:wg:oauth:2.0:oob"),
        ];

        let resp = self.client
            .post(ELYBY_TOKEN_URL)
            .form(&params)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            return Err(LarpError::Auth(AuthError::ElyBy(format!("Token exchange failed: {}", err))));
        }

        Ok(resp.json().await?)
    }

    /// Refresh token
    pub async fn refresh_token(&self, refresh_token: &str) -> LarpResult<ElyByTokenResponse> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.client_id),
        ];

        let resp = self.client
            .post(ELYBY_TOKEN_URL)
            .form(&params)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            return Err(LarpError::Auth(AuthError::ElyBy(format!("Token refresh failed: {}", err))));
        }

        Ok(resp.json().await?)
    }

    /// Get user profile
    pub async fn get_profile(&self, access_token: &str) -> LarpResult<ElyByProfile> {
        let resp = self.client
            .get(ELYBY_PROFILE_URL)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            return Err(LarpError::Auth(AuthError::ElyBy(format!("Profile fetch failed: {}", err))));
        }

        Ok(resp.json().await?)
    }

    /// Get user skins
    pub async fn get_skins(&self, access_token: &str) -> LarpResult<Vec<ElyBySkin>> {
        let resp = self.client
            .get(ELYBY_SKIN_URL)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            return Err(LarpError::Auth(AuthError::ElyBy(format!("Skins fetch failed: {}", err))));
        }

        Ok(resp.json().await?)
    }
}

impl Default for ElyByAuthClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Token response
#[derive(Debug, Deserialize)]
pub struct ElyByTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub scope: String,
}

/// User profile
#[derive(Debug, Deserialize)]
pub struct ElyByProfile {
    pub uuid: String,
    pub username: String,
    pub email: String,
    pub premium: bool,
    #[serde(default)]
    pub selected_skin: Option<String>,
    #[serde(default)]
    pub selected_cape: Option<String>,
}

/// Skin info
#[derive(Debug, Deserialize)]
pub struct ElyBySkin {
    pub id: String,
    pub name: String,
    pub url: String,
    pub variant: String, // "classic" or "slim"
    #[serde(default)]
    pub is_default: bool,
}

/// Ely.by login state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ElyByLoginState {
    Pending,
    Authorizing,
    ExchangingCode,
    FetchingProfile,
    Completed,
    Failed,
    Cancelled,
}

/// Ely.by background login
pub struct ElyByLogin {
    client: Client,
    auth_url: String,
    email: String,
    password: String,
    state: Arc<RwLock<ElyByLoginState>>,
    account: Arc<RwLock<Option<Account>>>,
    error: Arc<RwLock<Option<AuthError>>>,
}

impl ElyByLogin {
    pub fn auth_url(&self) -> &str {
        &self.auth_url
    }
}

#[async_trait]
impl crate::auth::BackgroundLogin for ElyByLogin {
    async fn start(&self) -> LarpResult<()> {
        *self.state.write().await = ElyByLoginState::Authorizing;
        // In a real implementation, this would open a webview for user to log in
        // For now, we'll simulate the flow
        Ok(())
    }

    async fn is_complete(&self) -> bool {
        matches!(
            *self.state.read().await,
            ElyByLoginState::Completed | ElyByLoginState::Failed | ElyByLoginState::Cancelled
        )
    }

    async fn get_account(&self) -> Option<Account> {
        self.account.read().await.clone()
    }

    async fn get_error(&self) -> Option<AuthError> {
        self.error.read().await.clone()
    }

    async fn cancel(&self) {
        *self.state.write().await = ElyByLoginState::Cancelled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elyby_client_creation() {
        let _client = ElyByAuthClient::new();
    }
}