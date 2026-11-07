//! Platform module — JNI bridge, app lifecycle, tools, vulkan, multirt

pub mod callback;
pub mod app;
pub mod tools;
pub mod vulkan;
pub mod multirt;

use larp_jv_core::platform::{FileDialog, Clipboard, Notifications, SystemTray, Vibration, Permissions};
use crate::platform::callback::JniBridge;
use std::sync::Arc;

/// Android platform implementation
pub struct AndroidPlatform {
    jni_bridge: Arc<JniBridge>,
}

impl AndroidPlatform {
    pub fn new(jni_bridge: Arc<JniBridge>) -> Self {
        Self { jni_bridge }
    }
}

// Platform trait implementations
impl FileDialog for AndroidPlatform {
    fn open_file(&self, _filters: &[(&str, &[&str])]) -> larp_jv_core::LarpResult<Option<std::path::PathBuf>> {
        // TODO: Implement via JNI
        Ok(None)
    }

    fn save_file(&self, _filters: &[(&str, &[&str])], _default_name: &str) -> larp_jv_core::LarpResult<Option<std::path::PathBuf>> {
        Ok(None)
    }

    fn select_folder(&self) -> larp_jv_core::LarpResult<Option<std::path::PathBuf>> {
        Ok(None)
    }
}

impl Clipboard for AndroidPlatform {
    fn get_text(&self) -> larp_jv_core::LarpResult<String> {
        // TODO: Implement via JNI
        Ok(String::new())
    }

    fn set_text(&self, _text: &str) -> larp_jv_core::LarpResult<()> {
        Ok(())
    }
}

impl Notifications for AndroidPlatform {
    fn send(&self, _title: &str, _body: &str) -> larp_jv_core::LarpResult<()> {
        Ok(())
    }

    fn send_with_action(&self, _title: &str, _body: &str, _action_label: &str, _action_url: &str) -> larp_jv_core::LarpResult<()> {
        Ok(())
    }
}

impl SystemTray for AndroidPlatform {
    fn set_icon(&self, _icon_path: &std::path::Path) -> larp_jv_core::LarpResult<()> {
        Ok(())
    }

    fn set_tooltip(&self, _tooltip: &str) -> larp_jv_core::LarpResult<()> {
        Ok(())
    }

    fn show_menu(&self) -> larp_jv_core::LarpResult<()> {
        Ok(())
    }
}

impl Vibration for AndroidPlatform {
    fn vibrate(&self, duration_ms: u64) -> larp_jv_core::LarpResult<()> {
        self.jni_bridge.vibrate(duration_ms);
        Ok(())
    }
}

impl Permissions for AndroidPlatform {
    fn request_permission(&self, permission: &str) -> larp_jv_core::LarpResult<bool> {
        self.jni_bridge.request_permission(permission)
    }

    fn check_permission(&self, permission: &str) -> larp_jv_core::LarpResult<bool> {
        self.jni_bridge.check_permission(permission)
    }
}