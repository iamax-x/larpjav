//! Mod loader support for larp.jv

pub mod forge;
pub mod fabric;
pub mod quilt;
pub mod neoforge;
pub mod optifine;
pub mod legacy_fabric;
pub mod bta;
pub mod modpack;
pub mod search;
pub mod manager;

use crate::error::{LarpError, LarpResult, ModdingError};
use crate::value::{ModLoaderType, ModInfo, ModLoaderMetadata};
use crate::utils::download::{create_client, download_file};
use reqwest::Client;
use std::path::Path;
use std::collections::HashMap;

/// Mod loader trait
#[async_trait::async_trait]
pub trait ModLoader: Send + Sync {
    fn loader_type(&self) -> ModLoaderType;
    fn name(&self) -> &str;

    /// Get available versions for Minecraft version
    async fn available_versions(&self, mc_version: &str) -> LarpResult<Vec<String>>;

    /// Install mod loader
    async fn install(
        &self,
        mc_version: &str,
        loader_version: &str,
        dest_dir: &Path,
        profile: &crate::value::Version,
    ) -> LarpResult<InstalledModLoader>;

    /// Get installer profile JSON
    async fn get_installer_profile(
        &self,
        mc_version: &str,
        loader_version: &str,
    ) -> LarpResult<serde_json::Value>;
}

/// Installed mod loader info
#[derive(Debug, Clone)]
pub struct InstalledModLoader {
    pub loader: ModLoaderType,
    pub version: String,
    pub mc_version: String,
    pub libraries: Vec<std::path::PathBuf>,
    pub main_class: String,
    pub tweak_class: Option<String>,
    pub arguments: crate::value::Arguments,
}

/// Mod installer manager
pub struct ModManager {
    client: Client,
    forge: forge::ForgeInstaller,
    fabric: fabric::FabricInstaller,
    quilt: quilt::QuiltInstaller,
    neoforge: neoforge::NeoForgeInstaller,
    optifine: optifine::OptiFineInstaller,
    legacy_fabric: legacy_fabric::LegacyFabricInstaller,
    bta: bta::BTAInstaller,
}

impl ModManager {
    pub fn new() -> LarpResult<Self> {
        let client = create_client()?;
        Ok(Self {
            client: client.clone(),
            forge: forge::ForgeInstaller::new(client.clone()),
            fabric: fabric::FabricInstaller::new(client.clone()),
            quilt: quilt::QuiltInstaller::new(client.clone()),
            neoforge: neoforge::NeoForgeInstaller::new(client.clone()),
            optifine: optifine::OptiFineInstaller::new(client.clone()),
            legacy_fabric: legacy_fabric::LegacyFabricInstaller::new(client.clone()),
            bta: bta::BTAInstaller::new(client.clone()),
        })
    }

    pub fn get_loader(&self, loader: ModLoaderType) -> &dyn ModLoader {
        match loader {
            ModLoaderType::Forge => &self.forge,
            ModLoaderType::Fabric => &self.fabric,
            ModLoaderType::Quilt => &self.quilt,
            ModLoaderType::NeoForge => &self.neoforge,
            ModLoaderType::Optifine => &self.optifine,
            ModLoaderType::LegacyFabric => &self.legacy_fabric,
            ModLoaderType::BTA => &self.bta,
            _ => &self.fabric, // fallback
        }
    }

    /// Install mod loader
    pub async fn install_loader(
        &self,
        loader: ModLoaderType,
        mc_version: &str,
        loader_version: &str,
        dest_dir: &Path,
        profile: &crate::value::Version,
    ) -> LarpResult<InstalledModLoader> {
        let loader_impl = self.get_loader(loader);
        loader_impl.install(mc_version, loader_version, dest_dir, profile).await
    }

    /// Get available versions for loader
    pub async fn available_versions(&self, loader: ModLoaderType, mc_version: &str) -> LarpResult<Vec<String>> {
        let loader_impl = self.get_loader(loader);
        loader_impl.available_versions(mc_version).await
    }

    /// Install mod
    pub async fn install_mod(
        &self,
        mod_info: &ModInfo,
        dest_dir: &Path,
    ) -> LarpResult<std::path::PathBuf> {
        let dest = dest_dir.join(format!("{}-{}.jar", mod_info.id, mod_info.version));
        if let Some(url) = &mod_info.url {
            download_file(&self.client, url, &dest, None, false).await?;
        }
        Ok(dest)
    }

    /// Search mods
    pub async fn search_mods(&self, query: &str, loader: ModLoaderType, mc_version: &str) -> LarpResult<Vec<ModInfo>> {
        search::search_mods(&self.client, query, loader, mc_version).await
    }
}

impl Default for ModManager {
    fn default() -> Self {
        Self::new().expect("Failed to create ModManager")
    }
}

/// Common mod loader functions
pub mod common {
    use crate::error::LarpResult;
    use crate::utils::download::download_file;
    use reqwest::Client;
    use std::path::Path;

    /// Download and run Forge installer
    pub async fn run_forge_installer(
        client: &Client,
        installer_url: &str,
        dest_dir: &Path,
        mc_version: &str,
    ) -> LarpResult<()> {
        let installer_path = dest_dir.join("forge-installer.jar");
        download_file(client, installer_url, &installer_path, None, false).await?;

        // Run Forge installer
        let output = tokio::process::Command::new("java")
            .arg("-jar")
            .arg(&installer_path)
            .arg("--installClient")
            .arg(dest_dir)
            .output()
            .await?;

        if !output.status.success() {
            return Err(crate::error::LarpError::Modding(crate::error::ModdingError::Forge(
                String::from_utf8_lossy(&output.stderr).to_string(),
            )));
        }

        Ok(())
    }

    /// Download Fabric installer
    pub async fn download_fabric_installer(
        client: &Client,
        mc_version: &str,
        loader_version: &str,
        dest_dir: &Path,
    ) -> LarpResult<std::path::PathBuf> {
        let url = format!(
            "https://maven.fabricmc.net/net/fabricmc/fabric-installer/{}/fabric-installer-{}.jar",
            loader_version, loader_version
        );
        let dest = dest_dir.join(format!("fabric-installer-{}.jar", loader_version));
        download_file(client, &url, &dest, None, false).await?;
        Ok(dest)
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