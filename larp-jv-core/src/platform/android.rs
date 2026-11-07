//! Android platform implementation

use crate::platform::traits::{FileDialog, Clipboard, Notifications, Vibration, Permissions};
use crate::error::LarpResult;
use async_trait::async_trait;
use std::path::Path;

/// Android file dialog (uses JNI)
pub struct AndroidFileDialog;

#[async_trait]
impl FileDialog for AndroidFileDialog {
    async fn open_file(&self, filters: &[(&str, &[&str])]) -> LarpResult<Option<std::path::PathBuf>> {
        // TODO: Implement via JNI
        Ok(None)
    }

    async fn save_file(&self, filters: &[(&str, &[&str])], default_name: &str) -> LarpResult<Option<std::path::PathBuf>> {
        // TODO: Implement via JNI
        Ok(None)
    }

    async fn select_folder(&self) -> LarpResult<Option<std::path::PathBuf>> {
        // TODO: Implement via JNI
        Ok(None)
    }
}

/// Android clipboard
pub struct AndroidClipboard;

#[async_trait]
impl Clipboard for AndroidClipboard {
    async fn get_text(&self) -> LarpResult<String> {
        // TODO: Implement via JNI
        Ok(String::new())
    }

    async fn set_text(&self, text: &str) -> LarpResult<()> {
        // TODO: Implement via JNI
        Ok(())
    }
}

/// Android notifications
pub struct AndroidNotifications;

#[async_trait]
impl Notifications for AndroidNotifications {
    async fn send(&self, title: &str, body: &str) -> LarpResult<()> {
        // TODO: Implement via JNI
        Ok(())
    }

    async fn send_with_action(&self, title: &str, body: &str, action_label: &str, action_url: &str) -> LarpResult<()> {
        // TODO: Implement via JNI
        Ok(())
    }
}

/// Android vibration
pub struct AndroidVibration;

#[async_trait]
impl Vibration for AndroidVibration {
    async fn vibrate(&self, duration_ms: u64) -> LarpResult<()> {
        // TODO: Implement via JNI
        Ok(())
    }
}

/// Android permissions
pub struct AndroidPermissions;

#[async_trait]
impl Permissions for AndroidPermissions {
    async fn request_permission(&self, permission: &str) -> LarpResult<bool> {
        // TODO: Implement via JNI
        Ok(false)
    }

    async fn check_permission(&self, permission: &str) -> LarpResult<bool> {
        // TODO: Implement via JNI
        Ok(false)
    }
}
