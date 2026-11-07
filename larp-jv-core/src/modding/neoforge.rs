//! NeoForge mod loader installer

use crate::error::{LarpError, LarpResult, ModdingError};
use crate::modding::{ModLoader, InstalledModLoader};
use crate::value::Version;
use crate::utils::download::download_file;
use reqwest::Client;
use std::path::Path;

/// NeoForge installer
pub struct NeoForgeInstaller {
    client: Client,
}

impl NeoForgeInstaller {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Get NeoForge versions for Minecraft version
    pub async fn get_versions(&self, mc_version: &str) -> LarpResult<Vec<String>> {
        let url = format!("https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/maven-metadata.xml", mc_version);
        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;

        let mut versions = Vec::new();
        for line in text.lines() {
            if line.contains("<version>") {
                let start = line.find("<version>").unwrap_or(0) + 9;
                let end = line.find("</version>").unwrap_or(line.len());
                if start < end {
                    versions.push(line[start..end].to_string());
                }
            }
        }

        Ok(versions)
    }

    /// Download NeoForge installer
    pub async fn download_installer(&self, mc_version: &str, version: &str, dest: &Path) -> LarpResult<std::path::PathBuf> {
        let url = format!(
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/{}/neoforge-{}-{}-installer.jar",
            mc_version, version, mc_version, version
        );
        let dest_path = dest.join(format!("neoforge-{}-{}-installer.jar", mc_version, version));
        download_file(&self.client, &url, &dest_path, None, false).await?;
        Ok(dest_path)
    }

    /// Run NeoForge installer
    pub async fn run_installer(
        &self,
        installer_path: &Path,
        dest_dir: &Path,
    ) -> LarpResult<()> {
        let output = tokio::process::Command::new("java")
            .arg("-jar")
            .arg(installer_path)
            .arg("--installClient")
            .arg(dest_dir)
            .output()
            .await?;

        if !output.status.success() {
            return Err(LarpError::Modding(ModdingError::NeoForge(
                String::from_utf8_lossy(&output.stderr).to_string(),
            )));
        }

        Ok(())
    }
}

impl ModLoader for NeoForgeInstaller {
    fn loader_type(&self) -> crate::value::ModLoaderType {
        crate::value::ModLoaderType::NeoForge
    }

    fn name(&self) -> &str {
        "NeoForge"
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
        let installer = self.download_installer(mc_version, loader_version, dest_dir).await?;
        self.run_installer(&installer, dest_dir).await?;

        let profile_path = dest_dir.join("versions").join(format!("{}-neoforge-{}", mc_version, loader_version)).join(format!("{}-neoforge-{}.json", mc_version, loader_version));
        let profile_json = tokio::fs::read_to_string(&profile_path).await?;
        let profile: Version = serde_json::from_str(&profile_json)?;

        Ok(InstalledModLoader {
            loader: crate::value::ModLoaderType::NeoForge,
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
    fn test_neoforge_installer_creation() {
        let client = reqwest::Client::new();
        let _installer = NeoForgeInstaller::new(client);
    }
}