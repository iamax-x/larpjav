//! Platform traits

use crate::error::LarpResult;
use async_trait::async_trait;
use std::path::Path;

/// File dialog trait
#[async_trait]
pub trait FileDialog: Send + Sync {
    async fn open_file(&self, filters: &[(&str, &[&str])]) -> LarpResult<Option<std::path::PathBuf>>;
    async fn save_file(&self, filters: &[(&str, &[&str])], default_name: &str) -> LarpResult<Option<std::path::PathBuf>>;
    async fn select_folder(&self) -> LarpResult<Option<std::path::PathBuf>>;
}

/// Clipboard trait
#[async_trait]
pub trait Clipboard: Send + Sync {
    async fn get_text(&self) -> LarpResult<String>;
    async fn set_text(&self, text: &str) -> LarpResult<()>;
}

/// Notifications trait
#[async_trait]
pub trait Notifications: Send + Sync {
    async fn send(&self, title: &str, body: &str) -> LarpResult<()>;
    async fn send_with_action(&self, title: &str, body: &str, action_label: &str, action_url: &str) -> LarpResult<()>;
}

/// System tray trait
#[async_trait]
pub trait SystemTray: Send + Sync {
    async fn set_icon(&self, icon_path: &Path) -> LarpResult<()>;
    async fn set_tooltip(&self, tooltip: &str) -> LarpResult<()>;
    async fn show_menu(&self) -> LarpResult<()>;
}

/// Vibration trait (mobile)
#[async_trait]
pub trait Vibration: Send + Sync {
    async fn vibrate(&self, duration_ms: u64) -> LarpResult<()>;
}

/// Permissions trait
#[async_trait]
pub trait Permissions: Send + Sync {
    async fn request_permission(&self, permission: &str) -> LarpResult<bool>;
    async fn check_permission(&self, permission: &str) -> LarpResult<bool>;
}

/// Platform interface
pub struct Platform {
    pub file_dialog: Box<dyn FileDialog>,
    pub clipboard: Box<dyn Clipboard>,
    pub notifications: Box<dyn Notifications>,
    pub system_tray: Option<Box<dyn SystemTray>>,
    pub vibration: Option<Box<dyn Vibration>>,
    pub permissions: Option<Box<dyn Permissions>>,
}

impl Platform {
    pub fn new(
        file_dialog: Box<dyn FileDialog>,
        clipboard: Box<dyn Clipboard>,
        notifications: Box<dyn Notifications>,
    ) -> Self {
        Self {
            file_dialog,
            clipboard,
            notifications,
            system_tray: None,
            vibration: None,
            permissions: None,
        }
    }
}
