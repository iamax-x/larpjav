//! Mod manager - unified interface for all mod loaders

use crate::error::{LarpError, LarpResult};
use crate::modding::{ModLoader, InstalledModLoader, ModManager as ModManagerImpl};
use crate::value::{ModLoaderType, ModInfo, Version};
use crate::utils::download::create_client;
use reqwest::Client;
use std::path::Path;

/// High-level mod manager
pub struct ModManager {
    inner: ModManagerImpl,
}

impl ModManager {
    pub fn new() -> LarpResult<Self> {
        Ok(Self {
            inner: ModManagerImpl::new()?,
        })
    }

    /// Install mod loader
    pub async fn install_loader(
        &self,
        loader: ModLoaderType,
        mc_version: &str,
        loader_version: &str,
        dest_dir: &Path,
        profile: &Version,
    ) -> LarpResult<InstalledModLoader> {
        self.inner.install_loader(loader, mc_version, loader_version, dest_dir, profile).await
    }

    /// Get available versions for loader
    pub async fn available_versions(&self, loader: ModLoaderType, mc_version: &str) -> LarpResult<Vec<String>> {
        self.inner.available_versions(loader, mc_version).await
    }

    /// Install mod
    pub async fn install_mod(
        &self,
        mod_info: &ModInfo,
        dest_dir: &Path,
    ) -> LarpResult<std::path::PathBuf> {
        self.inner.install_mod(mod_info, dest_dir).await
    }

    /// Search mods
    pub async fn search_mods(
        &self,
        query: &str,
        loader: ModLoaderType,
        mc_version: &str,
    ) -> LarpResult<Vec<ModInfo>> {
        self.inner.search_mods(query, loader, mc_version).await
    }

    /// Get loader instance
    pub fn get_loader(&self, loader: ModLoaderType) -> &dyn ModLoader {
        self.inner.get_loader(loader)
    }

    /// Install multiple mods
    pub async fn install_mods(
        &self,
        mods: &[ModInfo],
        dest_dir: &Path,
    ) -> LarpResult<Vec<std::path::PathBuf>> {
        let mut results = Vec::new();
        for mod_info in mods {
            let path = self.install_mod(mod_info, dest_dir).await?;
            results.push(path);
        }
        Ok(results)
    }

    /// Check mod compatibility
    pub async fn check_compatibility(
        &self,
        mods: &[ModInfo],
        mc_version: &str,
        loader: ModLoaderType,
    ) -> LarpResult<CompatibilityReport> {
        // Simplified compatibility check
        let mut report = CompatibilityReport {
            compatible: true,
            conflicts: Vec::new(),
            missing_dependencies: Vec::new(),
        };

        // Check for duplicate mods
        let mut seen = std::collections::HashSet::new();
        for mod_info in mods {
            if !seen.insert(mod_info.id.clone()) {
                report.conflicts.push(format!("Duplicate mod: {}", mod_info.id));
                report.compatible = false;
            }
        }

        Ok(report)
    }
}

/// Compatibility report
#[derive(Debug, Clone)]
pub struct CompatibilityReport {
    pub compatible: bool,
    pub conflicts: Vec<String>,
    pub missing_dependencies: Vec<String>,
}

impl Default for ModManager {
    fn default() -> Self {
        Self::new().expect("Failed to create ModManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_manager_creation() {
        let _manager = ModManager::new().unwrap();
    }
}