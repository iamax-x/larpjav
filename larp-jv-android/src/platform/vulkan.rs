//! Android Vulkan Driver Manager — port of DriverManager.java + drivers

use larp_jv_core::{LarpResult, utils::download::download_file};
use reqwest::Client;
use std::path::Path;
use std::sync::Arc;

/// Vulkan driver types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanDriverType {
    Adreno,
    Builtin,
    Display,
}

/// Vulkan driver info
#[derive(Debug, Clone)]
pub struct VulkanDriver {
    pub driver_type: VulkanDriverType,
    pub name: String,
    pub version: String,
    pub path: std::path::PathBuf,
    pub is_installed: bool,
}

/// Vulkan driver manager
pub struct VulkanDriverManager {
    client: Client,
    driver_dir: std::path::PathBuf,
    installed_drivers: Vec<VulkanDriver>,
}

impl VulkanDriverManager {
    pub fn new(driver_dir: impl AsRef<Path>) -> LarpResult<Self> {
        let driver_dir = driver_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&driver_dir)?;
        
        Ok(Self {
            client: Client::new(),
            driver_dir,
            installed_drivers: Vec::new(),
        })
    }

    /// Get available drivers for current device
    pub fn get_available_drivers(&self) -> Vec<VulkanDriver> {
        let mut drivers = Vec::new();
        
        // Check for Adreno
        if Self::is_adreno_device() {
            drivers.push(VulkanDriver {
                driver_type: VulkanDriverType::Adreno,
                name: "Adreno Vulkan Driver".to_string(),
                version: "latest".to_string(),
                path: self.driver_dir.join("adreno"),
                is_installed: self.driver_dir.join("adreno").exists(),
            });
        }
        
        // Builtin (swiftshader/moltenvk fallback)
        drivers.push(VulkanDriver {
            driver_type: VulkanDriverType::Builtin,
            name: "Builtin Vulkan Driver".to_string(),
            version: "1.0".to_string(),
            path: self.driver_dir.join("builtin"),
            is_installed: true,
        });
        
        // Display driver
        drivers.push(VulkanDriver {
            driver_type: VulkanDriverType::Display,
            name: "Display Vulkan Driver".to_string(),
            version: "1.0".to_string(),
            path: self.driver_dir.join("display"),
            is_installed: self.driver_dir.join("display").exists(),
        });
        
        drivers
    }

    /// Check if device has Adreno GPU
    fn is_adreno_device() -> bool {
        if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
            content.contains("Adreno") || content.contains("Qualcomm")
        } else {
            false
        }
    }

    /// Install driver
    pub async fn install_driver(&mut self, driver_type: VulkanDriverType) -> LarpResult<()> {
        match driver_type {
            VulkanDriverType::Adreno => self.install_adreno().await,
            VulkanDriverType::Builtin => self.install_builtin().await,
            VulkanDriverType::Display => self.install_display().await,
        }
    }

    async fn install_adreno(&mut self) -> LarpResult<()> {
        let dest_dir = self.driver_dir.join("adreno");
        std::fs::create_dir_all(&dest_dir)?;
        
        // URLs from MojoLauncher for Adreno drivers
        const ADRENO_URLS: &[&str] = &[
            "https://github.com/MojoLauncher/MojoLauncher/releases/download/drivers/adreno_vulkan_1.zip",
            "https://github.com/MojoLauncher/MojoLauncher/releases/download/drivers/adreno_vulkan_2.zip",
        ];
        
        for url in ADRENO_URLS {
            let dest = dest_dir.join("driver.zip");
            download_file(&self.client, url, &dest, None, false).await?;
            
            // Extract zip
            larp_jv_core::utils::zip::extract_zip(&dest, &dest_dir).await?;
            std::fs::remove_file(&dest).ok();
        }
        
        Ok(())
    }

    async fn install_builtin(&self) -> LarpResult<()> {
        // Builtin driver (swiftshader/moltenvk fallback) - already available
        Ok(())
    }

    async fn install_display(&mut self) -> LarpResult<()> {
        let dest_dir = self.driver_dir.join("display");
        std::fs::create_dir_all(&dest_dir)?;
        
        // Display driver for devices without Vulkan
        // Would download from MojoLauncher releases
        Ok(())
    }

    /// Get driver path for Vulkan loader
    pub fn get_driver_path(&self, driver_type: VulkanDriverType) -> Option<std::path::PathBuf> {
        let path = match driver_type {
            VulkanDriverType::Adreno => self.driver_dir.join("adreno"),
            VulkanDriverType::Builtin => self.driver_dir.join("builtin"),
            VulkanDriverType::Display => self.driver_dir.join("display"),
        };
        
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }
}

/// Get recommended driver for current device
pub fn get_recommended_driver() -> VulkanDriverType {
    if VulkanDriverManager::is_adreno_device() {
        VulkanDriverType::Adreno
    } else {
        VulkanDriverType::Builtin
    }
}