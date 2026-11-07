//! Background authentication infrastructure

use crate::auth::{BackgroundLogin, LoginListener};
use crate::error::{AuthError, LarpError, LarpResult};
use crate::value::Account;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Background authentication manager
pub struct BackgroundAuthManager {
    login_tx: broadcast::Sender<BackgroundAuthEvent>,
    current_login: Arc<RwLock<Option<Box<dyn BackgroundLogin>>>>,
    listener: Arc<RwLock<Option<Box<dyn LoginListener>>>>,
}

/// Background authentication events
#[derive(Debug, Clone)]
pub enum BackgroundAuthEvent {
    Started(crate::value::AuthType),
    Progress(f32, String),
    Success(Account),
    Failed(AuthError),
    Cancelled,
}

impl BackgroundAuthManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(32);
        Self {
            login_tx: tx,
            current_login: Arc::new(RwLock::new(None)),
            listener: Arc::new(RwLock::new(None)),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<BackgroundAuthEvent> {
        self.login_tx.subscribe()
    }

    pub fn set_listener<L: LoginListener + 'static>(&self, listener: L) {
        *self.listener.write().await = Some(Box::new(listener));
    }

    /// Start a background login
    pub async fn start_login<L: BackgroundLogin + 'static>(&self, login: L) -> LarpResult<()> {
        // Cancel any existing login
        if let Some(current) = self.current_login.write().await.take() {
            current.cancel().await;
        }

        let login_box: Box<dyn BackgroundLogin> = Box::new(login);
        *self.current_login.write().await = Some(login_box);

        // Notify listener
        if let Some(listener) = self.listener.read().await.as_ref() {
            // We need to know the auth type - for now, we'll emit a generic event
            let _ = self.login_tx.send(BackgroundAuthEvent::Started(crate::value::AuthType::Microsoft));
        }

        Ok(())
    }

    /// Get current login status
    pub async fn is_logging_in(&self) -> bool {
        self.current_login.read().await.is_some()
    }

    /// Cancel current login
    pub async fn cancel_login(&self) {
        if let Some(login) = self.current_login.write().await.take() {
            login.cancel().await;
            let _ = self.login_tx.send(BackgroundAuthEvent::Cancelled);
        }
    }

    /// Poll for completion (for non-event-driven UIs)
    pub async fn poll_completion(&self) -> Option<LarpResult<Account>> {
        if let Some(login) = self.current_login.read().await.as_ref() {
            if login.is_complete().await {
                if let Some(account) = login.get_account().await {
                    return Some(Ok(account));
                }
                if let Some(error) = login.get_error().await {
                    return Some(Err(LarpError::Auth(error)));
                }
            }
        }
        None
    }
}

impl Default for BackgroundAuthManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Authentication state for UI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthState {
    Idle,
    InProgress(crate::value::AuthType, f32, String), // type, progress, message
    Success(Account),
    Failed(AuthError),
    Cancelled,
}

/// Simple authentication controller for UI
pub struct AuthController {
    manager: BackgroundAuthManager,
    state: Arc<RwLock<AuthState>>,
}

impl AuthController {
    pub fn new() -> Self {
        Self {
            manager: BackgroundAuthManager::new(),
            state: Arc::new(RwLock::new(AuthState::Idle)),
        }
    }

    pub async fn state(&self) -> AuthState {
        self.state.read().await.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<BackgroundAuthEvent> {
        self.manager.subscribe()
    }

    pub async fn start_microsoft(&self) -> LarpResult<()> {
        // This would use the MicrosoftAuthClient from the auth module
        // For now, just update state
        *self.state.write().await = AuthState::InProgress(
            crate::value::AuthType::Microsoft,
            0.0,
            "Starting Microsoft authentication...".to_string(),
        );
        Ok(())
    }

    pub async fn start_elyby(&self, email: &str, password: &str) -> LarpResult<()> {
        *self.state.write().await = AuthState::InProgress(
            crate::value::AuthType::ElyBy,
            0.0,
            "Starting Ely.by authentication...".to_string(),
        );
        Ok(())
    }

    pub async fn start_offline(&self, username: &str) -> LarpResult<()> {
        *self.state.write().await = AuthState::InProgress(
            crate::value::AuthType::Offline,
            1.0,
            "Creating offline account...".to_string(),
        );
        // Simulate completion
        let account = Account::offline(username);
        *self.state.write().await = AuthState::Success(account.clone());
        Ok(())
    }

    pub async fn cancel(&self) {
        self.manager.cancel_login().await;
        *self.state.write().await = AuthState::Cancelled;
    }
}

impl Default for AuthController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_controller() {
        let controller = AuthController::new();
        assert_eq!(controller.state().await, AuthState::Idle);

        controller.start_offline("TestPlayer").await.unwrap();
        assert!(matches!(controller.state().await, AuthState::Success(_)));
    }
}