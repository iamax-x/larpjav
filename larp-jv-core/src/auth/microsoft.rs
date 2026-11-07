//! Microsoft (Xbox Live) Authentication

use crate::error::{AuthError, LarpError, LarpResult};
use crate::value::Account;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

const MS_AUTHORIZE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const MS_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const XBOX_LIVE_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MINECRAFT_AUTH_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MINECRAFT_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
const MINECRAFT_OWNERSHIP_URL: &str = "https://api.minecraftservices.com/entitlements/mcstore";
const CLIENT_ID: &str = "0000000048183522";
const REDIRECT_URI: &str = "https://login.microsoftonline.com/common/oauth2/nativeclient";
const SCOPES: &str = "XboxLive.signin XboxLive.offline_access";

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
    message: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    refresh_token: String,
    scope: String,
}

#[derive(Debug, Serialize)]
struct XboxLiveAuthRequest {
    #[serde(rename = "Properties")]
    properties: XboxLiveProperties,
    #[serde(rename = "RelyingParty")]
    relying_party: String,
    #[serde(rename = "TokenType")]
    token_type: String,
}

#[derive(Debug, Serialize)]
struct XboxLiveProperties {
    #[serde(rename = "AuthMethod")]
    auth_method: String,
    #[serde(rename = "SiteName")]
    site_name: String,
    #[serde(rename = "RpsTicket")]
    rps_ticket: String,
}

#[derive(Debug, Deserialize)]
struct XboxLiveAuthResponse {
    #[serde(rename = "IssueInstant")]
    issue_instant: String,
    #[serde(rename = "NotAfter")]
    not_after: String,
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: DisplayClaims,
}

#[derive(Debug, Deserialize)]
struct DisplayClaims {
    #[serde(rename = "xui")]
    xui: Vec<XuiClaim>,
}

#[derive(Debug, Deserialize)]
struct XuiClaim {
    uhs: String,
}

#[derive(Debug, Serialize)]
struct XstsAuthRequest {
    #[serde(rename = "Properties")]
    properties: XstsProperties,
    #[serde(rename = "RelyingParty")]
    relying_party: String,
    #[serde(rename = "TokenType")]
    token_type: String,
}

#[derive(Debug, Serialize)]
struct XstsProperties {
    #[serde(rename = "SandboxId")]
    sandbox_id: String,
    #[serde(rename = "UserTokens")]
    user_tokens: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct XstsAuthResponse {
    #[serde(rename = "IssueInstant")]
    issue_instant: String,
    #[serde(rename = "NotAfter")]
    not_after: String,
    #[serde(rename = "Token")]
    token: String,
}

#[derive(Debug, Serialize)]
struct MinecraftAuthRequest {
    identity_token: String,
    ensure_legacy_enabled: bool,
}

#[derive(Debug, Deserialize)]
struct MinecraftAuthResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
struct MinecraftProfile {
    id: String,
    name: String,
    skins: Vec<SkinInfo>,
    capes: Vec<CapeInfo>,
}

#[derive(Debug, Deserialize)]
struct SkinInfo {
    id: String,
    state: String,
    url: String,
    variant: String,
}

#[derive(Debug, Deserialize)]
struct CapeInfo {
    id: String,
    state: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct OwnershipResponse {
    items: Vec<OwnershipItem>,
}

#[derive(Debug, Deserialize)]
struct OwnershipItem {
    name: String,
    signature: String,
}

pub struct MicrosoftAuthClient {
    client: Client,
}

impl MicrosoftAuthClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub async fn start_device_code_flow(&self) -> LarpResult<MicrosoftLogin> {
        let params = [
            ("client_id", CLIENT_ID),
            ("scope", SCOPES),
        ];

        let resp = self.client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
            .form(&params)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            return Err(LarpError::Auth(AuthError::Microsoft(format!("Device code request failed: {}", err))));
        }

        let device_code: DeviceCodeResponse = resp.json().await?;

        Ok(MicrosoftLogin {
            client: self.client.clone(),
            device_code: device_code.device_code,
            user_code: device_code.user_code,
            verification_uri: device_code.verification_uri,
            expires_at: SystemTime::now() + Duration::from_secs(device_code.expires_in),
            interval: Duration::from_secs(device_code.interval),
            message: device_code.message,
            state: Arc::new(RwLock::new(MicrosoftLoginState::Pending)),
            account: Arc::new(RwLock::new(None)),
            error: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn exchange_code(&self, code: &str) -> LarpResult<TokenResponse> {
        let params = [
            ("client_id", CLIENT_ID),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", REDIRECT_URI),
            ("scope", SCOPES),
        ];

        let resp = self.client
            .post(MS_TOKEN_URL)
            .form(&params)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            return Err(LarpError::Auth(AuthError::Microsoft(format!("Token exchange failed: {}", err))));
        }

        Ok(resp.json().await?)
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> LarpResult<TokenResponse> {
        let params = [
            ("client_id", CLIENT_ID),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
            ("scope", SCOPES),
        ];

        let resp = self.client
            .post(MS_TOKEN_URL)
            .form(&params)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            return Err(LarpError::Auth(AuthError::Microsoft(format!("Token refresh failed: {}", err))));
        }

        Ok(resp.json().await?)
    }

    pub async fn authenticate_full(&self, login: &MicrosoftLogin) -> LarpResult<Account> {
        let token_resp = login.wait_for_token().await?;
        let xbl_token = self.xbox_live_auth(&token_resp.access_token).await?;
        let xsts_token = self.xsts_auth(&xbl_token.token).await?;
        let mc_token = self.minecraft_auth(&xbl_token.display_claims.xui[0].uhs, &xsts_token.token).await?;
        let profile = self.get_minecraft_profile(&mc_token.access_token).await?;
        self.check_ownership(&mc_token.access_token).await?;

        let account = Account {
            id: profile.id.clone(),
            username: profile.name.clone(),
            auth_type: crate::value::AuthType::Microsoft,
            access_token: Some(mc_token.access_token),
            refresh_token: Some(token_resp.refresh_token),
            expires_at: Some(chrono::DateTime::from_timestamp(
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64 + mc_token.expires_in as i64,
                0,
            ).unwrap()),
            uuid: Some(profile.id),
            skin_url: profile.skins.iter().find(|s| s.state == "ACTIVE").map(|s| s.url.clone()),
            cape_url: profile.capes.iter().find(|c| c.state == "ACTIVE").map(|c| c.url.clone()),
            skin_model: profile.skins.iter().find(|s| s.state == "ACTIVE").map(|s| s.variant.clone()),
            selected: false,
            created_at: chrono::Utc::now(),
            last_used: chrono::Utc::now(),
        };

        Ok(account)
    }

    async fn xbox_live_auth(&self, access_token: &str) -> LarpResult<XboxLiveAuthResponse> {
        let request = XboxLiveAuthRequest {
            properties: XboxLiveProperties {
                auth_method: "RPS".to_string(),
                site_name: "user.auth.xboxlive.com".to_string(),
                rps_ticket: format!("d={}", access_token),
            },
            relying_party: "http://auth.xboxlive.com".to_string(),
            token_type: "JWT".to_string(),
        };

        let resp = self.client
            .post(XBOX_LIVE_AUTH_URL)
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            return Err(LarpError::Auth(AuthError::Microsoft(format!("Xbox Live auth failed: {}", err))));
        }

        Ok(resp.json().await?)
    }

    async fn xsts_auth(&self, xbl_token: &str) -> LarpResult<XstsAuthResponse> {
        let request = XstsAuthRequest {
            properties: XstsProperties {
                sandbox_id: "RETAIL".to_string(),
                user_tokens: vec![xbl_token.to_string()],
            },
            relying_party: "rp://api.minecraftservices.com/".to_string(),
            token_type: "JWT".to_string(),
        };

        let resp = self.client
            .post(XSTS_AUTH_URL)
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            return Err(LarpError::Auth(AuthError::Microsoft(format!("XSTS auth failed: {}", err))));
        }

        Ok(resp.json().await?)
    }

    async fn minecraft_auth(&self, uhs: &str, xsts_token: &str) -> LarpResult<MinecraftAuthResponse> {
        let request = MinecraftAuthRequest {
            identity_token: format!("XBL3.0 x={};{}", uhs, xsts_token),
            ensure_legacy_enabled: true,
        };

        let resp = self.client
            .post(MINECRAFT_AUTH_URL)
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            return Err(LarpError::Auth(AuthError::Microsoft(format!("Minecraft auth failed: {}", err))));
        }

        Ok(resp.json().await?)
    }

    async fn get_minecraft_profile(&self, access_token: &str) -> LarpResult<MinecraftProfile> {
        let resp = self.client
            .get(MINECRAFT_PROFILE_URL)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            return Err(LarpError::Auth(AuthError::Microsoft(format!("Profile fetch failed: {}", err))));
        }

        Ok(resp.json().await?)
    }

    async fn check_ownership(&self, access_token: &str) -> LarpResult<()> {
        let resp = self.client
            .get(MINECRAFT_OWNERSHIP_URL)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            return Err(LarpError::Auth(AuthError::Microsoft(format!("Ownership check failed: {}", err))));
        }

        let ownership: OwnershipResponse = resp.json().await?;
        if ownership.items.is_empty() {
            return Err(LarpError::Auth(AuthError::Microsoft("Account does not own Minecraft".to_string())));
        }

        Ok(())
    }
}

impl Default for MicrosoftAuthClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MicrosoftLoginState {
    Pending,
    Authorized,
    TokenReceived,
    Authenticating,
    Completed,
    Failed,
    Cancelled,
}

pub struct MicrosoftLogin {
    client: Client,
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_at: SystemTime,
    interval: Duration,
    message: String,
    state: Arc<RwLock<MicrosoftLoginState>>,
    account: Arc<RwLock<Option<Account>>>,
    error: Arc<RwLock<Option<AuthError>>>,
}

impl MicrosoftLogin {
    pub fn user_code(&self) -> &str {
        &self.user_code
    }

    pub fn verification_uri(&self) -> &str {
        &self.verification_uri
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn is_expired(&self) -> bool {
        SystemTime::now() >= self.expires_at
    }

    async fn wait_for_token(&self) -> LarpResult<TokenResponse> {
        let mut interval = tokio::time::interval(self.interval);

        loop {
            if *self.state.read().await == MicrosoftLoginState::Cancelled {
                return Err(LarpError::Auth(AuthError::UserCancelled));
            }

            if self.is_expired() {
                *self.state.write().await = MicrosoftLoginState::Failed;
                return Err(LarpError::Auth(AuthError::Microsoft("Device code expired".to_string())));
            }

            interval.tick().await;

            let params = [
                ("client_id", CLIENT_ID),
                ("device_code", &self.device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ];

            let resp = self.client
                .post(MS_TOKEN_URL)
                .form(&params)
                .send()
                .await?;

            if resp.status().is_success() {
                let token: TokenResponse = resp.json().await?;
                *self.state.write().await = MicrosoftLoginState::TokenReceived;
                return Ok(token);
            }

            let error: serde_json::Value = resp.json().await.unwrap_or_default();
            if let Some(error_code) = error.get("error").and_then(|e| e.as_str()) {
                match error_code {
                    "authorization_pending" => continue,
                    "slow_down" => {
                        interval = tokio::time::interval(self.interval * 2);
                        continue;
                    }
                    "expired_token" => {
                        *self.state.write().await = MicrosoftLoginState::Failed;
                        return Err(LarpError::Auth(AuthError::Microsoft("Device code expired".to_string())));
                    }
                    "access_denied" => {
                        *self.state.write().await = MicrosoftLoginState::Failed;
                        return Err(LarpError::Auth(AuthError::UserCancelled));
                    }
                    _ => {
                        *self.state.write().await = MicrosoftLoginState::Failed;
                        return Err(LarpError::Auth(AuthError::Microsoft(format!("Token request failed: {}", error_code))));
                    }
                }
            }
        }
    }
}

#[async_trait]
impl crate::auth::BackgroundLogin for MicrosoftLogin {
    async fn start(&self) -> LarpResult<()> {
        *self.state.write().await = MicrosoftLoginState::Authorized;
        Ok(())
    }

    async fn is_complete(&self) -> bool {
        matches!(
            *self.state.read().await,
            MicrosoftLoginState::Completed | MicrosoftLoginState::Failed | MicrosoftLoginState::Cancelled
        )
    }

    async fn get_account(&self) -> Option<Account> {
        self.account.read().await.clone()
    }

    async fn get_error(&self) -> Option<AuthError> {
        self.error.read().await.clone()
    }

    async fn cancel(&self) {
        *self.state.write().await = MicrosoftLoginState::Cancelled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_microsoft_client_creation() {
        let _client = MicrosoftAuthClient::new();
    }
}