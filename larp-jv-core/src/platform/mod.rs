//! Platform abstraction layer

pub mod traits;
pub mod android;
pub mod desktop;

use crate::error::LarpResult;

/// Platform type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformType {
    Android,
    Linux,
    Windows,
    MacOS,
    Unknown,
}

/// Get current platform
pub fn current_platform() -> PlatformType {
    #[cfg(target_os = "android")] return PlatformType::Android;
    #[cfg(target_os = "linux")] return PlatformType::Linux;
    #[cfg(target_os = "windows")] return PlatformType::Windows;
    #[cfg(target_os = "macos")] return PlatformType::MacOS;
    PlatformType::Unknown
}

/// Platform capabilities
#[derive(Debug, Clone)]
pub struct PlatformCapabilities {
    pub has_file_dialog: bool,
    pub has_system_tray: bool,
    pub has_notifications: bool,
    pub has_clipboard: bool,
    pub has_vulkan: bool,
    pub has_gles: bool,
    pub supports_background: bool,
}

impl Default for PlatformCapabilities {
    fn default() -> Self {
        match current_platform() {
            PlatformType::Android => Self {
                has_file_dialog: true,
                has_system_tray: false,
                has_notifications: true,
                has_clipboard: true,
                has_vulkan: true,
                has_gles: true,
                supports_background: true,
            },
            PlatformType::Linux | PlatformType::Windows | PlatformType::MacOS => Self {
                has_file_dialog: true,
                has_system_tray: true,
                has_notifications: true,
                has_clipboard: true,
                has_vulkan: true,
                has_gles: false,
                supports_background: true,
            },
            _ => Self {
                has_file_dialog: false,
                has_system_tray: false,
                has_notifications: false,
                has_clipboard: false,
                has_vulkan: false,
                has_gles: false,
                supports_background: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_platform() {
        let platform = current_platform();
        assert_ne!(platform, PlatformType::Unknown);
    }
}
