//! Authentication module for larp.jv
//!
//! Supports Microsoft (Xbox Live), Ely.by, and Offline authentication.

pub mod microsoft;
pub mod elyby;
pub mod offline;
pub mod account;
pub mod listener;
pub mod skin;
pub mod oauth;
pub mod background;

use crate::error::{AuthError, LarpError, LarpResult};
use crate::value::{Account, AccountManager, AuthType};
use async_trait::async_trait;
use std::sync::Arc;

/// Background login trait
#[async_trait]
pub trait BackgroundLogin: Send + Sync {
    /// Start authentication flow
    async fn start(&self) -> LarpResult<()>;

    /// Check if authentication is complete
    async fn is_complete(&self) -> bool;

    /// Get authenticated account (if complete)
    async fn get_account(&self) -> Option<Account>;

    /// Get error if authentication failed
    async fn get_error(&self) -> Option<AuthError>;

    /// Cancel authentication
    async fn cancel(&self);
}

/// Authentication manager
pub struct AuthManager {
    account_manager: AccountManager,
    microsoft_client: microsoft::MicrosoftAuthClient,
    elyby_client: elyby::ElyByAuthClient,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            account_manager: AccountManager::new(),
            microsoft_client: microsoft::MicrosoftAuthClient::new(),
            elyby_client: elyby::ElyByAuthClient::new(),
        }
    }

    pub fn with_account_manager(mut self, manager: AccountManager) -> Self {
        self.account_manager = manager;
        self
    }

    /// Get account manager
    pub fn accounts(&self) -> &AccountManager {
        &self.account_manager
    }

    /// Get mutable account manager
    pub fn accounts_mut(&mut self) -> &mut AccountManager {
        &mut self.account_manager
    }

    /// Start Microsoft authentication
    pub async fn start_microsoft_auth(&self) -> LarpResult<Box<dyn BackgroundLogin>> {
        let login = self.microsoft_client.start_device_code_flow().await?;
        Ok(Box::new(login))
    }

    /// Start Ely.by authentication
    pub async fn start_elyby_auth(&self, email: &str, password: &str) -> LarpResult<Box<dyn BackgroundLogin>> {
        let login = self.elyby_client.start_login(email, password).await?;
        Ok(Box::new(login))
    }

    /// Create offline account
    pub fn create_offline_account(&mut self, username: &str) -> Account {
        let account = Account::offline(username);
        self.account_manager.add_account(account.clone());
        account
    }

    /// Add existing account
    pub fn add_account(&mut self, account: Account) {
        self.account_manager.add_account(account);
    }

    /// Remove account
    pub fn remove_account(&mut self, id: &str) -> bool {
        self.account_manager.remove_account(id)
    }

    /// Get selected account
    pub fn selected_account(&self) -> Option<&Account> {
        self.account_manager.get_selected()
    }

    /// Select account
    pub fn select_account(&mut self, id: &str) -> bool {
        self.account_manager.select_account(id)
    }

    /// Refresh tokens for all accounts
    pub async fn refresh_all(&mut self) -> Vec<LarpResult<()>> {
        let mut results = Vec::new();
        for account in self.account_manager.list_accounts().to_vec() {
            if account.needs_refresh() {
                let result = match account.auth_type {
                    crate::value::AuthType::Microsoft => self.refresh_microsoft(&account).await,
                    crate::value::AuthType::ElyBy => self.refresh_elyby(&account).await,
                    crate::value::AuthType::Offline => Ok(()),
                };
                results.push(result);
            }
        }
        results
    }

    async fn refresh_microsoft(&mut self, account: &Account) -> LarpResult<()> {
        // TODO: Implement token refresh
        Ok(())
    }

    async fn refresh_elyby(&mut self, account: &Account) -> LarpResult<()> {
        // TODO: Implement token refresh
        Ok(())
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating background login implementations
pub struct LoginFactory;

impl LoginFactory {
    /// Create background login for auth type
    pub async fn create(auth_type: AuthType, account_manager: &AccountManager) -> LarpResult<Box<dyn BackgroundLogin>> {
        match auth_type {
            AuthType::Microsoft => {
                let client = microsoft::MicrosoftAuthClient::new();
                let login = client.start_device_code_flow().await?;
                Ok(Box::new(login))
            }
            AuthType::ElyBy => {
                // Need credentials for ElyBy
                Err(LarpError::Auth(AuthError::InvalidCredentials))
            }
            AuthType::Offline => {
                Ok(Box::new(offline::OfflineLogin::new()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_manager_creation() {
        let manager = AuthManager::new();
        assert!(manager.accounts().list_accounts().is_empty());
    }

    #[test]
    fn test_offline_account() {
        let mut manager = AuthManager::new();
        let account = manager.create_offline_account("TestPlayer");
        assert_eq!(account.username, "TestPlayer");
        assert_eq!(account.auth_type, AuthType::Offline);
        assert!(!account.needs_refresh());
    }
}
