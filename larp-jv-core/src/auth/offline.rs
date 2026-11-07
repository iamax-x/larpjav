//! Offline Authentication

use crate::error::{AuthError, LarpError, LarpResult};
use crate::value::Account;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Offline login - no network required
pub struct OfflineLogin {
    username: String,
    state: Arc<RwLock<OfflineLoginState>>,
    account: Arc<RwLock<Option<Account>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OfflineLoginState {
    Pending,
    Completed,
    Failed,
    Cancelled,
}

impl OfflineLogin {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            state: Arc::new(RwLock::new(OfflineLoginState::Pending)),
            account: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_username(mut self, username: &str) -> Self {
        self.username = username.to_string();
        self
    }
}

impl Default for OfflineLogin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::auth::BackgroundLogin for OfflineLogin {
    async fn start(&self) -> LarpResult<()> {
        if self.username.trim().is_empty() {
            *self.state.write().await = OfflineLoginState::Failed;
            return Err(LarpError::Auth(AuthError::InvalidCredentials));
        }

        let account = Account::offline(&self.username);
        *self.account.write().await = Some(account);
        *self.state.write().await = OfflineLoginState::Completed;
        Ok(())
    }

    async fn is_complete(&self) -> bool {
        matches!(
            *self.state.read().await,
            OfflineLoginState::Completed | OfflineLoginState::Failed | OfflineLoginState::Cancelled
        )
    }

    async fn get_account(&self) -> Option<Account> {
        self.account.read().await.clone()
    }

    async fn get_error(&self) -> Option<AuthError> {
        match *self.state.read().await {
            OfflineLoginState::Failed => Some(AuthError::InvalidCredentials),
            OfflineLoginState::Cancelled => Some(AuthError::UserCancelled),
            _ => None,
        }
    }

    async fn cancel(&self) {
        *self.state.write().await = OfflineLoginState::Cancelled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_offline_login() {
        let login = OfflineLogin::new().with_username("TestPlayer");
        login.start().await.unwrap();
        assert!(login.is_complete().await);
        let account = login.get_account().await.unwrap();
        assert_eq!(account.username, "TestPlayer");
        assert_eq!(account.auth_type, crate::value::AuthType::Offline);
    }

    #[tokio::test]
    async fn test_offline_login_empty() {
        let login = OfflineLogin::new().with_username("");
        assert!(login.start().await.is_err());
    }
}