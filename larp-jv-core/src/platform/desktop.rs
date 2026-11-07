//! Desktop platform implementation (Linux, Windows, macOS)

use crate::platform::traits::{FileDialog, Clipboard, Notifications, SystemTray};
use crate::error::LarpResult;
use async_trait::async_trait;
use std::path::Path;

/// Desktop file dialog (using rfd)
pub struct DesktopFileDialog;

#[async_trait]
impl FileDialog for DesktopFileDialog {
    async fn open_file(&self, filters: &[(&str, &[&str])]) -> LarpResult<Option<std::path::PathBuf>> {
        let mut dialog = rfd::AsyncFileDialog::new();
        for (name, extensions) in filters {
            dialog = dialog.add_filter(name, extensions);
        }
        Ok(dialog.pick_file().await.map(|f| f.path().to_owned()))
    }

    async fn save_file(&self, filters: &[(&str, &[&str])], default_name: &str) -> LarpResult<Option<std::path::PathBuf>> {
        let mut dialog = rfd::AsyncFileDialog::new();
        for (name, extensions) in filters {
            dialog = dialog.add_filter(name, extensions);
        }
        dialog = dialog.set_file_name(default_name);
        Ok(dialog.save_file().await.map(|f| f.path().to_owned()))
    }

    async fn select_folder(&self) -> LarpResult<Option<std::path::PathBuf>> {
        Ok(rfd::AsyncFileDialog::new().pick_folder().await.map(|f| f.path().to_owned()))
    }
}

/// Desktop clipboard (using arboard)
pub struct DesktopClipboard {
    clipboard: std::sync::Mutex<Option<arboard::Clipboard>>,
}

impl DesktopClipboard {
    pub fn new() -> Self {
        Self {
            clipboard: std::sync::Mutex::new(arboard::Clipboard::new().ok()),
        }
    }
}

#[async_trait]
impl Clipboard for DesktopClipboard {
    async fn get_text(&self) -> LarpResult<String> {
        let mut clipboard = self.clipboard.lock().unwrap();
        if let Some(cb) = clipboard.as_mut() {
            cb.get_text().map_err(|e| crate::error::LarpError::Other(e.to_string()))
        } else {
            Ok(String::new())
        }
    }

    async fn set_text(&self, text: &str) -> LarpResult<()> {
        let mut clipboard = self.clipboard.lock().unwrap();
        if let Some(cb) = clipboard.as_mut() {
            cb.set_text(text).map_err(|e| crate::error::LarpError::Other(e.to_string()))
        } else {
            Ok(())
        }
    }
}

/// Desktop notifications (using notify-rust on Linux, native on Windows/macOS)
pub struct DesktopNotifications;

#[async_trait]
impl Notifications for DesktopNotifications {
    async fn send(&self, title: &str, body: &str) -> LarpResult<()> {
        #[cfg(target_os = "linux")]
        {
            use notify_rust::Notification;
            Notification::new()
                .summary(title)
                .body(body)
                .appname("larp.jv")
                .show()
                .map_err(|e| crate::error::LarpError::Other(e.to_string()))?;
        }
        #[cfg(target_os = "windows")]
        {
            // Use Windows toast notifications
            // Implementation would use windows-rs crate
        }
        #[cfg(target_os = "macos")]
        {
            // Use macOS user notifications
            // Implementation would use objc crate
        }
        Ok(())
    }

    async fn send_with_action(&self, title: &str, body: &str, action_label: &str, action_url: &str) -> LarpResult<()> {
        // Extended notification with action button
        self.send(title, body).await
    }
}

/// Desktop system tray (using tray-icon)
pub struct DesktopSystemTray;

#[async_trait]
impl SystemTray for DesktopSystemTray {
    async fn set_icon(&self, icon_path: &Path) -> LarpResult<()> {
        // TODO: Implement using tray-icon crate
        Ok(())
    }

    async fn set_tooltip(&self, tooltip: &str) -> LarpResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn show_menu(&self) -> LarpResult<()> {
        // TODO: Implement
        Ok(())
    }
}
