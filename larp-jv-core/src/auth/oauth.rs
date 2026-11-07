//! OAuth2 types and utilities

use crate::error::LarpResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// OAuth2 token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: String,
    #[serde(default)]
    pub id_token: Option<String>,
}

impl OAuthTokenResponse {
    pub fn is_expired(&self) -> bool {
        if self.expires_in == 0 {
            return false;
        }
        let expiry = SystemTime::now() + Duration::from_secs(self.expires_in);
        SystemTime::now() >= expiry
    }

    pub fn expires_at(&self) -> SystemTime {
        SystemTime::now() + Duration::from_secs(self.expires_in)
    }
}

/// OAuth2 authorization request
#[derive(Debug, Serialize)]
pub struct OAuthAuthRequest {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: String,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_challenge: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_challenge_method: Option<String>,
}

/// OAuth2 token request
#[derive(Debug, Serialize)]
pub struct OAuthTokenRequest {
    pub grant_type: String,
    pub client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_code: Option<String>,
}

/// OAuth2 device code request
#[derive(Debug, Serialize)]
pub struct OAuthDeviceCodeRequest {
    pub client_id: String,
    pub scope: String,
}

/// OAuth2 device code response
#[derive(Debug, Deserialize)]
pub struct OAuthDeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub expires_in: u64,
    pub interval: u64,
}

/// OAuth2 error response
#[derive(Debug, Deserialize)]
pub struct OAuthErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
    pub error_uri: Option<String>,
}

/// PKCE (Proof Key for Code Exchange) utilities
pub mod pkce {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use rand::RngCore;
    use sha2::{Digest, Sha256};

    /// Generate code verifier (43-128 chars)
    pub fn generate_code_verifier() -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        URL_SAFE_NO_PAD.encode(bytes)
    }

    /// Generate code challenge from verifier
    pub fn generate_code_challenge(verifier: &str) -> String {
        let hash = Sha256::digest(verifier.as_bytes());
        URL_SAFE_NO_PAD.encode(hash)
    }
}

/// Build authorization URL
pub fn build_auth_url(base_url: &str, request: &OAuthAuthRequest) -> LarpResult<String> {
    let mut url = url::Url::parse(base_url)?;
    let mut pairs = url.query_pairs_mut();
    pairs.clear();
    pairs.append_pair("client_id", &request.client_id);
    pairs.append_pair("redirect_uri", &request.redirect_uri);
    pairs.append_pair("response_type", &request.response_type);
    pairs.append_pair("scope", &request.scope);
    pairs.append_pair("state", &request.state);
    if let Some(challenge) = &request.code_challenge {
        pairs.append_pair("code_challenge", challenge);
    }
    if let Some(method) = &request.code_challenge_method {
        pairs.append_pair("code_challenge_method", method);
    }
    Ok(url.to_string())
}

/// Parse token response
pub fn parse_token_response(body: &str) -> LarpResult<OAuthTokenResponse> {
    Ok(serde_json::from_str(body)?)
}

/// Parse device code response
pub fn parse_device_code_response(body: &str) -> LarpResult<OAuthDeviceCodeResponse> {
    Ok(serde_json::from_str(body)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce() {
        let verifier = pkce::generate_code_verifier();
        assert!(verifier.len() >= 43);
        let challenge = pkce::generate_code_challenge(&verifier);
        assert!(!challenge.is_empty());
    }

    #[test]
    fn test_oauth_token_response() {
        let response = OAuthTokenResponse {
            access_token: "test".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: Some("refresh".to_string()),
            scope: "test".to_string(),
            id_token: None,
        };
        assert!(!response.is_expired());
    }
}