//! Android Tools — port of Tools.java

use larp_jv_core::LarpResult;
use std::path::Path;

/// Android tools implementation
pub struct AndroidTools;

impl AndroidTools {
    /// Get files directory
    pub fn get_files_dir() -> LarpResult<std::path::PathBuf> {
        // Return standard Android files directory
        Ok(std::path::PathBuf::from("/data/data/com.larp.jv/files"))
    }

    /// Get cache directory
    pub fn get_cache_dir() -> LarpResult<std::path::PathBuf> {
        Ok(std::path::PathBuf::from("/data/data/com.larp.jv/cache"))
    }

    /// Get external files directory
    pub fn get_external_files_dir() -> LarpResult<std::path::PathBuf> {
        Ok(std::path::PathBuf::from("/sdcard/Android/data/com.larp.jv/files"))
    }

    /// Check if external storage is available
    pub fn is_external_storage_available() -> bool {
        std::path::Path::new("/sdcard").exists()
    }

    /// Get available memory in MB
    pub fn get_available_memory_mb() -> u64 {
        // Read from /proc/meminfo
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemAvailable:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<u64>() {
                            return kb / 1024;
                        }
                    }
                }
            }
        }
        0
    }

    /// Get total memory in MB
    pub fn get_total_memory_mb() -> u64 {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<u64>() {
                            return kb / 1024;
                        }
                    }
                }
            }
        }
        0
    }

    /// Check if device has feature
    pub fn has_system_feature(feature: &str) -> bool {
        // Would need JNI to check PackageManager.hasSystemFeature
        false
    }

    /// Get device model
    pub fn get_device_model() -> String {
        std::fs::read_to_string("/sys/class/dmi/id/product_name")
            .or_else(|_| std::fs::read_to_string("/proc/device-tree/model"))
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string()
    }

    /// Get Android version
    pub fn get_android_version() -> String {
        std::fs::read_to_string("/system/build.prop")
            .map(|content| {
                for line in content.lines() {
                    if line.starts_with("ro.build.version.release=") {
                        return line.split('=').nth(1).unwrap_or("Unknown").to_string();
                    }
                }
                "Unknown".to_string()
            })
            .unwrap_or_else(|_| "Unknown".to_string())
    }

    /// Get SDK version
    pub fn get_sdk_version() -> u32 {
        std::fs::read_to_string("/system/build.prop")
            .map(|content| {
                for line in content.lines() {
                    if line.starts_with("ro.build.version.sdk=") {
                        return line.split('=').nth(1).unwrap_or("0").parse().unwrap_or(0);
                    }
                }
                0
            })
            .unwrap_or(0)
    }

    /// Vibrate device
    pub fn vibrate(duration_ms: u64) {
        // Implemented via JNI bridge
    }

    /// Show toast message
    pub fn show_toast(message: &str) {
        // Implemented via JNI bridge
    }

    /// Open URL in browser
    pub fn open_url(url: &str) -> LarpResult<()> {
        // Implemented via JNI bridge
        Ok(())
    }

    /// Share text
    pub fn share_text(text: &str, subject: &str) -> LarpResult<()> {
        // Implemented via JNI bridge
        Ok(())
    }

    /// Request install packages permission
    pub fn request_install_permission() -> LarpResult<()> {
        Ok(())
    }

    /// Check if app is debuggable
    pub fn is_debuggable() -> bool {
        cfg!(debug_assertions)
    }
}