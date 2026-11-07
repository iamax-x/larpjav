//! Notification utilities

use crate::error::LarpResult;

/// Notification priority
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Notification category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationCategory {
    Info,
    Warning,
    Error,
    Progress,
    Download,
    Auth,
    Game,
    Mod,
}

/// Notification
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub priority: NotificationPriority,
    pub category: NotificationCategory,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: Option<NotificationAction>,
}

/// Notification action
#[derive(Debug, Clone)]
pub struct NotificationAction {
    pub label: String,
    pub action_type: NotificationActionType,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationActionType {
    OpenInstance,
    OpenSettings,
    OpenUrl,
    RetryDownload,
    CancelDownload,
    ViewLog,
}

/// Notification manager trait
#[async_trait::async_trait]
pub trait NotificationManager: Send + Sync {
    async fn send(&self, notification: Notification) -> LarpResult<()>;
    async fn dismiss(&self, id: &str) -> LarpResult<()>;
    async fn dismiss_all(&self) -> LarpResult<()>;
    async fn get_history(&self, limit: usize) -> LarpResult<Vec<Notification>>;
}

/// Desktop notification manager (using notify-rust)
#[cfg(all(target_os = "linux", feature = "notify"))]
pub struct DesktopNotificationManager;

#[cfg(all(target_os = "linux", feature = "notify"))]
#[async_trait::async_trait]
impl NotificationManager for DesktopNotificationManager {
    async fn send(&self, notification: Notification) -> LarpResult<()> {
        use notify_rust::Notification;
        let mut n = Notification::new();
        n.summary(&notification.title)
            .body(&notification.body)
            .appname("larp.jv");
        if let Some(icon) = notification.category.icon_name() {
            n.icon(icon);
        }
        n.show()?;
        Ok(())
    }
    async fn dismiss(&self, _id: &str) -> LarpResult<()> { Ok(()) }
    async fn dismiss_all(&self) -> LarpResult<()> { Ok(()) }
    async fn get_history(&self, _limit: usize) -> LarpResult<Vec<Notification>> { Ok(Vec::new()) }
}

/// Android notification manager (stub - requires JNI)
#[cfg(target_os = "android")]
pub struct AndroidNotificationManager;

#[cfg(target_os = "android")]
#[async_trait::async_trait]
impl NotificationManager for AndroidNotificationManager {
    async fn send(&self, notification: Notification) -> LarpResult<()> {
        // TODO: Implement via JNI
        Ok(())
    }
    async fn dismiss(&self, _id: &str) -> LarpResult<()> { Ok(()) }
    async fn dismiss_all(&self) -> LarpResult<()> { Ok(()) }
    async fn get_history(&self, _limit: usize) -> LarpResult<Vec<Notification>> { Ok(Vec::new()) }
}

/// No-op notification manager (for testing/headless)
pub struct NoopNotificationManager;

#[async_trait::async_trait]
impl NotificationManager for NoopNotificationManager {
    async fn send(&self, _notification: Notification) -> LarpResult<()> { Ok(()) }
    async fn dismiss(&self, _id: &str) -> LarpResult<()> { Ok(()) }
    async fn dismiss_all(&self) -> LarpResult<()> { Ok(()) }
    async fn get_history(&self, _limit: usize) -> LarpResult<Vec<Notification>> { Ok(Vec::new()) }
}

impl NotificationCategory {
    pub fn icon_name(&self) -> Option<&'static str> {
        match self {
            NotificationCategory::Info => Some("dialog-information"),
            NotificationCategory::Warning => Some("dialog-warning"),
            NotificationCategory::Error => Some("dialog-error"),
            NotificationCategory::Progress => Some("dialog-information"),
            NotificationCategory::Download => Some("folder-download"),
            NotificationCategory::Auth => Some("system-users"),
            NotificationCategory::Game => Some("applications-games"),
            NotificationCategory::Mod => Some("application-x-addon"),
        }
    }
}

/// Create a simple info notification
pub fn info(title: impl Into<String>, body: impl Into<String>) -> Notification {
    Notification {
        id: uuid::Uuid::new_v4().to_string(),
        title: title.into(),
        body: body.into(),
        priority: NotificationPriority::Normal,
        category: NotificationCategory::Info,
        timestamp: chrono::Utc::now(),
        action: None,
    }
}

/// Create a download progress notification
pub fn download_progress(title: impl Into<String>, progress: f32, speed: u64) -> Notification {
    Notification {
        id: uuid::Uuid::new_v4().to_string(),
        title: title.into(),
        body: format!("{:.1}% - {}/s", progress * 100.0, format_bytes(speed)),
        priority: NotificationPriority::Low,
        category: NotificationCategory::Download,
        timestamp: chrono::Utc::now(),
        action: None,
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.1} {}", size, UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let n = info("Test", "Hello world");
        assert_eq!(n.title, "Test");
        assert_eq!(n.body, "Hello world");
        assert_eq!(n.priority, NotificationPriority::Normal);
        assert_eq!(n.category, NotificationCategory::Info);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500.0 B");
        assert_eq!(format_bytes(1500), "1.5 KB");
        assert_eq!(format_bytes(1500000), "1.4 MB");
        assert_eq!(format_bytes(1500000000), "1.4 GB");
    }
}
