//! Legacy Fabric mod loader installer

use crate::error::{LarpError, LarpResult, ModdingError};
use crate::modding::{ModLoader, InstalledModLoader};
use crate::value::Version;
use crate::utils::download::download_file;
use reqwest::Client;
use std::path::Path;

/// Legacy Fabric installer (for older Minecraft versions)
pub struct LegacyFabricInstaller {
    client: Client,
}

impl LegacyFabricInstaller {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Get Legacy Fabric versions
    pub async fn get_versions(&self, mc_version: &str) -> LarpResult<Vec<String>> {
        let url = "https://meta.fabricmc.net/v2/versions/loader";
        let resp = self.client.get(url).send().await?;
        let json: serde_json::Value = resp.json().await?;

        let mut versions = Vec::new();
        if let Some(arr) = json.as_array() {
            for item in arr {
                if let Some(version) = item.get("version").and_then(|v| v.as_str()) {
                    // Legacy Fabric versions typically start with 0.x
                    if version.starts_with("0.") {
                        versions.push(version.to_string());
                    }
                }
            }
        }

        Ok(versions)
    }

    /// Download Legacy Fabric installer
    pub async fn download_installer(&self, loader_version: &str, dest: &Path) -> LarpResult<std::path::PathBuf> {
        let url = format!(
            "https://maven.fabricmc.net/net/fabricmc/fabric-installer/{}/fabric-installer-{}.jar",
            loader_version, loader_version
        );
        let dest_path = dest.join(format!("legacy-fabric-installer-{}.jar", loader_version));
        download_file(&self.client, &url, &dest_path, None, false).await?;
        Ok(dest_path)
    }

    /// Run Legacy Fabric installer
    pub async fn run_installer(
        &self,
        installer_path: &Path,
        dest_dir: &Path,
        mc_version: &str,
        loader_version: &str,
    ) -> LarpResult<()> {
        let output = tokio::process::Command::new("java")
            .arg("-jar")
            .arg(installer_path)
            .arg("client")
            .arg("-dir")
            .arg(dest_dir)
            .arg("-mcversion")
            .arg(mc_version)
            .arg("-loader")
            .arg(loader_version)
            .arg("-noprofile")
            .output()
            .await?;

        if !output.status.success() {
            return Err(LarpError::Modding(ModdingError::LegacyFabric(
                String::from_utf8_lossy(&output.stderr).to_string(),
            )));
        }

        Ok(())
    }
}

impl ModLoader for LegacyFabricInstaller {
    fn loader_type(&self) -> crate::value::ModLoaderType {
        crate::value::ModLoaderType::LegacyFabric
    }

    fn name(&self) -> &str {
        "Legacy Fabric"
    }

    async fn available_versions(&self, mc_version: &str) -> LarpResult<Vec<String>> {
        self.get_versions(mc_version).await
    }

    async fn install(
        &self,
        mc_version: &str,
        loader_version: &str,
        dest_dir: &Path,
        _profile: &Version,
    ) -> LarpResult<InstalledModLoader> {
        let installer = self.download_installer(loader_version, dest_dir).await?;
        self.run_installer(&installer, dest_dir, mc_version, loader_version).await?;

        let profile_path = dest_dir.join("versions").join(format!("legacy-fabric-loader-{}-{}", loader_version, mc_version)).join(format!("legacy-fabric-loader-{}-{}.json", loader_version, mc_version));
        let profile_json = tokio::fs::read_to_string(&profile_path).await?;
        let profile: Version = serde_json::from_str(&profile_json)?;

        Ok(InstalledModLoader {
            loader: crate::value::ModLoaderType::LegacyFabric,
            version: loader_version.to_string(),
            mc_version: mc_version.to_string(),
            libraries: vec![],
            main_class: profile.main_class,
            tweak_class: None,
            arguments: profile.arguments,
        })
    }

    async fn get_installer_profile(
        &self,
        mc_version: &str,
        loader_version: &str,
    ) -> LarpResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_fabric_installer_creation() {
        let client = reqwest::Client::new();
        let _installer = LegacyFabricInstaller::new(client);
    }
}