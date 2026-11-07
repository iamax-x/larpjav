//! Account management

use crate::error::{LarpError, LarpResult};
use crate::value::{Account, AccountManager, AuthType};
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::{Aead, OsRng};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Encrypted account storage
pub struct EncryptedAccountManager {
    manager: AccountManager,
    key: [u8; 32],
    storage_path: std::path::PathBuf,
}

impl EncryptedAccountManager {
    pub fn new(storage_path: impl AsRef<Path>) -> LarpResult<Self> {
        let path = storage_path.as_ref().to_path_buf();
        let key = Self::generate_or_load_key(&path)?;
        let manager = if path.exists() {
            Self::load_encrypted(&path, &key)?
        } else {
            AccountManager::new().with_encryption_key(key)
        };

        Ok(Self {
            manager,
            key,
            storage_path: path,
        })
    }

    fn generate_or_load_key(path: &Path) -> LarpResult<[u8; 32]> {
        let key_path = path.with_extension("key");
        if key_path.exists() {
            let key_bytes = std::fs::read(&key_path)?;
            if key_bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_bytes);
                return Ok(key);
            }
        }

        // Generate new key
        let mut key = [0u8; 32];
        use rand::RngCore;
        OsRng.fill_bytes(&mut key);
        std::fs::write(&key_path, &key)?;
        Ok(key)
    }

    fn load_encrypted(path: &Path, key: &[u8; 32]) -> LarpResult<AccountManager> {
        let encrypted = std::fs::read(path)?;
        let decrypted = Self::decrypt(&encrypted, key)?;
        let manager: AccountManager = toml::from_str(&decrypted)?;
        Ok(manager.with_encryption_key(*key))
    }

    pub fn save(&self) -> LarpResult<()> {
        let serialized = toml::to_string(&self.manager)?;
        let encrypted = Self::encrypt(serialized.as_bytes(), &self.key)?;
        std::fs::write(&self.storage_path, encrypted)?;
        Ok(())
    }

    fn encrypt(data: &[u8], key: &[u8; 32]) -> LarpResult<Vec<u8>> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| LarpError::Auth(crate::error::AuthError::Crypto(e.into())))?;

        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.append(&mut ciphertext);
        Ok(result)
    }

    fn decrypt(data: &[u8], key: &[u8; 32]) -> LarpResult<String> {
        if data.len() < 12 {
            return Err(LarpError::Auth(crate::error::AuthError::Crypto("Invalid encrypted data".into())));
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| LarpError::Auth(crate::error::AuthError::Crypto(e.into())))?;

        String::from_utf8(plaintext)
            .map_err(|e| LarpError::Auth(crate::error::AuthError::String(e)))
    }

    pub fn accounts(&self) -> &AccountManager {
        &self.manager
    }

    pub fn accounts_mut(&mut self) -> &mut AccountManager {
        &mut self.manager
    }
}

/// Thread-safe account manager
pub struct SharedAccountManager {
    inner: Arc<RwLock<EncryptedAccountManager>>,
}

impl SharedAccountManager {
    pub fn new(storage_path: impl AsRef<Path>) -> LarpResult<Self> {
        Ok(Self {
            inner: Arc::new(RwLock::new(EncryptedAccountManager::new(storage_path)?)),
        })
    }

    pub async fn get_account(&self, id: &str) -> Option<Account> {
        self.inner.read().await.accounts().get_account(id).cloned()
    }

    pub async fn add_account(&self, account: Account) {
        self.inner.write().await.accounts_mut().add_account(account);
    }

    pub async fn remove_account(&self, id: &str) -> bool {
        self.inner.write().await.accounts_mut().remove_account(id)
    }

    pub async fn select_account(&self, id: &str) -> bool {
        self.inner.write().await.accounts_mut().select_account(id)
    }

    pub async fn list_accounts(&self) -> Vec<Account> {
        self.inner.read().await.accounts().list_accounts().to_vec()
    }

    pub async fn get_selected(&self) -> Option<Account> {
        self.inner.read().await.accounts().get_selected().cloned()
    }

    pub async fn save(&self) -> LarpResult<()> {
        self.inner.read().await.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_encrypted_account_manager() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("accounts.toml");
        let manager = EncryptedAccountManager::new(&path).unwrap();
        // Just verify creation works
    }
}