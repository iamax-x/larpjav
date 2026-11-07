//! Vulkan Driver System — Port of Driver, AdrenoDriver, BuiltinDriver, DisplayDriver, DriverManager, DriverInstaller

use crate::error::LarpResult;
use std::path::Path;
use async_trait::async_trait;

/// Vulkan Driver Trait — Port of Driver.java
#[async_trait]
pub trait VulkanDriver: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn is_available(&self) -> bool;
    async fn install(&self, dest_dir: &Path) -> LarpResult<()>;
    fn get_library_path(&self) -> Option<std::path::PathBuf>;
    fn get_icd_path(&self) -> Option<std::path::PathBuf>;
}

/// Adreno Driver — Port of AdrenoDriver.java
pub struct AdrenoDriver {
    version: String,
    download_url: String,
    installed: bool,
}

impl AdrenoDriver {
    pub fn new() -> Self {
        Self {
            version: "latest".to_string(),
            download_url: "https://developer.qualcomm.com/download/adreno-sdk".to_string(),
            installed: false,
        }
    }

    pub fn with_version(version: String) -> Self {
        Self { version, download_url: "https://developer.qualcomm.com/download/adreno-sdk".to_string(), installed: false }
    }
}

impl Default for AdrenoDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VulkanDriver for AdrenoDriver {
    fn name(&self) -> &str {
        "Adreno"
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn is_available(&self) -> bool {
        self.installed
    }

    async fn install(&self, dest_dir: &Path) -> LarpResult<()> {
        // Download and extract Adreno Vulkan driver
        // Port from AdrenoDriver.java install()
        log::info!("Installing Adreno driver to {:?}", dest_dir);
        tokio::fs::create_dir_all(dest_dir).await?;
        // Download logic would go here
        Ok(())
    }

    fn get_library_path(&self) -> Option<std::path::PathBuf> {
        if self.installed {
            Some(std::path::PathBuf::from("/vendor/lib64/libvulkan.so"))
        } else {
            None
        }
    }

    fn get_icd_path(&self) -> Option<std::path::PathBuf> {
        if self.installed {
            Some(std::path::PathBuf::from("/vendor/etc/vulkan/icd.d/adreno_icd.json"))
        } else {
            None
        }
    }
}

/// Builtin Driver — Port of BuiltinDriver.java
pub struct BuiltinDriver {
    installed: bool,
}

impl BuiltinDriver {
    pub fn new() -> Self {
        Self { installed: true } // Always available
    }
}

impl Default for BuiltinDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VulkanDriver for BuiltinDriver {
    fn name(&self) -> &str {
        "Builtin (Android)"
    }

    fn version(&self) -> &str {
        "system"
    }

    fn is_available(&self) -> bool {
        true
    }

    async fn install(&self, _dest_dir: &Path) -> LarpResult<()> {
        Ok(()) // No installation needed
    }

    fn get_library_path(&self) -> Option<std::path::PathBuf> {
        Some(std::path::PathBuf::from("/system/lib64/libvulkan.so"))
    }

    fn get_icd_path(&self) -> Option<std::path::PathBuf> {
        None // Uses system loader
    }
}

/// Display Driver — Port of DisplayDriver.java
pub struct DisplayDriver {
    installed: bool,
}

impl DisplayDriver {
    pub fn new() -> Self {
        Self { installed: true }
    }
}

impl Default for DisplayDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VulkanDriver for DisplayDriver {
    fn name(&self) -> &str {
        "Display (Google)"
    }

    fn version(&self) -> &str {
        "system"
    }

    fn is_available(&self) -> bool {
        true
    }

    async fn install(&self, _dest_dir: &Path) -> LarpResult<()> {
        Ok(())
    }

    fn get_library_path(&self) -> Option<std::path::PathBuf> {
        Some(std::path::PathBuf::from("/system/lib64/libvulkan.so"))
    }

    fn get_icd_path(&self) -> Option<std::path::PathBuf> {
        None
    }
}

/// Driver Manager — Port of DriverManager.java
pub struct DriverManager {
    drivers: Vec<Box<dyn VulkanDriver>>,
    selected: Option<usize>,
}

impl DriverManager {
    pub fn new() -> Self {
        let mut drivers: Vec<Box<dyn VulkanDriver>> = Vec::new();
        drivers.push(Box::new(BuiltinDriver::new()));
        drivers.push(Box::new(DisplayDriver::new()));
        drivers.push(Box::new(AdrenoDriver::new()));
        
        Self {
            drivers,
            selected: Some(0), // Default to builtin
        }
    }

    pub fn list_drivers(&self) -> Vec<(&str, &str, bool)> {
        self.drivers.iter().map(|d| (d.name(), d.version(), d.is_available())).collect()
    }

    pub fn select_driver(&mut self, index: usize) -> LarpResult<()> {
        if index < self.drivers.len() {
            self.selected = Some(index);
            Ok(())
        } else {
            Err(crate::error::LarpError::Other("Invalid driver index".to_string()))
        }
    }

    pub fn get_selected(&self) -> Option<&dyn VulkanDriver> {
        self.selected.and_then(|i| self.drivers.get(i)).map(|b| b.as_ref())
    }

    pub fn get_selected_mut(&mut self) -> Option<&mut dyn VulkanDriver> {
        self.selected.and_then(|i| self.drivers.get_mut(i)).map(|b| b.as_mut())
    }

    pub async fn install_selected(&mut self, dest_dir: &Path) -> LarpResult<()> {
        if let Some(driver) = self.get_selected_mut() {
            driver.install(dest_dir).await?;
        }
        Ok(())
    }

    pub fn auto_select_best(&mut self) {
        // Prefer Adreno if available, then Display, then Builtin
        for (i, driver) in self.drivers.iter().enumerate() {
            if driver.name() == "Adreno" && driver.is_available() {
                self.selected = Some(i);
                return;
            }
        }
        for (i, driver) in self.drivers.iter().enumerate() {
            if driver.name() == "Display" && driver.is_available() {
                self.selected = Some(i);
                return;
            }
        }
        self.selected = Some(0); // Builtin
    }
}

impl Default for DriverManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Driver Installer — Port of DriverInstaller.java
pub struct DriverInstaller {
    manager: DriverManager,
    install_dir: std::path::PathBuf,
}

impl DriverInstaller {
    pub fn new(install_dir: &Path) -> Self {
        Self {
            manager: DriverManager::new(),
            install_dir: install_dir.to_path_buf(),
        }
    }

    pub async fn install_all(&mut self) -> LarpResult<()> {
        for driver in &mut self.manager.drivers {
            if !driver.is_available() {
                driver.install(&self.install_dir).await?;
            }
        }
        self.manager.auto_select_best();
        Ok(())
    }

    pub fn get_install_dir(&self) -> &Path {
        &self.install_dir
    }

    pub fn get_manager(&self) -> &DriverManager {
        &self.manager
    }

    pub fn get_manager_mut(&mut self) -> &mut DriverManager {
        &mut self.manager
    }
}