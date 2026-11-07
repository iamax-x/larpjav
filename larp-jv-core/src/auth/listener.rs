//! Login listener trait

use crate::error::LarpResult;
use crate::value::Account;

/// Login listener for async authentication events
#[async_trait::async_trait]
pub trait LoginListener: Send + Sync {
    /// Called when login starts
    async fn on_login_started(&self, auth_type: crate::value::AuthType);

    /// Called when login progresses (e.g., user code displayed)
    async fn on_login_progress(&self, progress: f32, message: &str);

    /// Called when login completes successfully
    async fn on_login_success(&self, account: Account);

    /// Called when login fails
    async fn on_login_failed(&self, error: crate::error::AuthError);

    /// Called when login is cancelled
    async fn on_login_cancelled(&self);
}

/// No-op listener
pub struct NoopListener;

#[async_trait::async_trait]
impl LoginListener for NoopListener {
    async fn on_login_started(&self, _auth_type: crate::value::AuthType) {}
    async fn on_login_progress(&self, _progress: f32, _message: &str) {}
    async fn on_login_success(&self, _account: Account) {}
    async fn on_login_failed(&self, _error: crate::error::AuthError) {}
    async fn on_login_cancelled(&self) {}
}

/// Callback-based listener
pub struct CallbackListener {
    on_started: Option<Box<dyn Fn(crate::value::AuthType) + Send + Sync>>,
    on_progress: Option<Box<dyn Fn(f32, &str) + Send + Sync>>,
    on_success: Option<Box<dyn Fn(Account) + Send + Sync>>,
    on_failed: Option<Box<dyn Fn(crate::error::AuthError) + Send + Sync>>,
    on_cancelled: Option<Box<dyn Fn() + Send + Sync>>,
}

impl CallbackListener {
    pub fn new() -> Self {
        Self {
            on_started: None,
            on_progress: None,
            on_success: None,
            on_failed: None,
            on_cancelled: None,
        }
    }

    pub fn on_started<F: Fn(crate::value::AuthType) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_started = Some(Box::new(f));
        self
    }

    pub fn on_progress<F: Fn(f32, &str) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_progress = Some(Box::new(f));
        self
    }

    pub fn on_success<F: Fn(Account) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_success = Some(Box::new(f));
        self
    }

    pub fn on_failed<F: Fn(crate::error::AuthError) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_failed = Some(Box::new(f));
        self
    }

    pub fn on_cancelled<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_cancelled = Some(Box::new(f));
        self
    }
}

impl Default for CallbackListener {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl LoginListener for CallbackListener {
    async fn on_login_started(&self, auth_type: crate::value::AuthType) {
        if let Some(f) = &self.on_started {
            f(auth_type);
        }
    }

    async fn on_login_progress(&self, progress: f32, message: &str) {
        if let Some(f) = &self.on_progress {
            f(progress, message);
        }
    }

    async fn on_login_success(&self, account: Account) {
        if let Some(f) = &self.on_success {
            f(account);
        }
    }

    async fn on_login_failed(&self, error: crate::error::AuthError) {
        if let Some(f) = &self.on_failed {
            f(error);
        }
    }

    async fn on_login_cancelled(&self) {
        if let Some(f) = &self.on_cancelled {
            f();
        }
    }
}

/// Multi-listener that forwards to multiple listeners
pub struct MultiListener {
    listeners: Vec<Box<dyn LoginListener>>,
}

impl MultiListener {
    pub fn new() -> Self {
        Self { listeners: Vec::new() }
    }

    pub fn add<L: LoginListener + 'static>(&mut self, listener: L) {
        self.listeners.push(Box::new(listener));
    }
}

impl Default for MultiListener {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl LoginListener for MultiListener {
    async fn on_login_started(&self, auth_type: crate::value::AuthType) {
        for listener in &self.listeners {
            listener.on_login_started(auth_type).await;
        }
    }

    async fn on_login_progress(&self, progress: f32, message: &str) {
        for listener in &self.listeners {
            listener.on_login_progress(progress, message).await;
        }
    }

    async fn on_login_success(&self, account: Account) {
        for listener in &self.listeners {
            listener.on_login_success(account.clone()).await;
        }
    }

    async fn on_login_failed(&self, error: crate::error::AuthError) {
        for listener in &self.listeners {
            listener.on_login_failed(error).await;
        }
    }

    async fn on_login_cancelled(&self) {
        for listener in &self.listeners {
            listener.on_login_cancelled().await;
        }
    }
}