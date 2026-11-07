//! Quilt mod loader installer

use crate::error::{LarpError, LarpResult, ModdingError};
use crate::modding::{ModLoader, InstalledModLoader};
use crate::value::Version;
use crate::utils::download::download_file;
use reqwest::Client;
use std::path::Path;

/// Quilt installer
pub struct QuiltInstaller {
    client: Client,
}

impl QuiltInstaller {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Get Quilt loader versions
    pub async fn get_loader_versions(&self) -> LarpResult<Vec<String>> {
        let url = "https://meta.quiltmc.org/v3/versions/loader";
        let resp = self.client.get(url).send().await?;
        let json: serde_json::Value = resp.json().await?;

        let mut versions = Vec::new();
        if let Some(arr) = json.as_array() {
            for item in arr {
                if let Some(version) = item.get("version").and_then(|v| v.as_str()) {
                    versions.push(version.to_string());
                }
            }
        }

        Ok(versions)
    }

    /// Get Minecraft versions supported by Quilt
    pub async fn get_minecraft_versions(&self) -> LarpResult<Vec<String>> {
        let url = "https://meta.quiltmc.org/v3/versions/game";
        let resp = self.client.get(url).send().await?;
        let json: serde_json::Value = resp.json().await?;

        let mut versions = Vec::new();
        if let Some(arr) = json.as_array() {
            for item in arr {
                if let Some(version) = item.get("version").and_then(|v| v.as_str()) {
                    versions.push(version.to_string());
                }
            }
        }

        Ok(versions)
    }

    /// Download Quilt installer
    pub async fn download_installer(&self, loader_version: &str, dest: &Path) -> LarpResult<std::path::PathBuf> {
        let url = format!(
            "https://maven.quiltmc.org/repository/release/org/quiltmc/quilt-installer/{}/quilt-installer-{}.jar",
            loader_version, loader_version
        );
        let dest_path = dest.join(format!("quilt-installer-{}.jar", loader_version));
        download_file(&self.client, &url, &dest_path, None, false).await?;
        Ok(dest_path)
    }

    /// Run Quilt installer
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
            .arg("install")
            .arg("client")
            .arg(dest_dir)
            .arg("--mc-version")
            .arg(mc_version)
            .arg("--loader-version")
            .arg(loader_version)
            .arg("--no-profile")
            .output()
            .await?;

        if !output.status.success() {
            return Err(LarpError::Modding(ModdingError::Quilt(
                String::from_utf8_lossy(&output.stderr).to_string(),
            )));
        }

        Ok(())
    }
}

impl ModLoader for QuiltInstaller {
    fn loader_type(&self) -> crate::value::ModLoaderType {
        crate::value::ModLoaderType::Quilt
    }

    fn name(&self) -> &str {
        "Quilt"
    }

    async fn available_versions(&self, mc_version: &str) -> LarpResult<Vec<String>> {
        let loaders = self.get_loader_versions().await?;
        Ok(loaders)
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

        let profile_path = dest_dir.join("versions").join(format!("quilt-loader-{}-{}", loader_version, mc_version)).join(format!("quilt-loader-{}-{}.json", loader_version, mc_version));
        let profile_json = tokio::fs::read_to_string(&profile_path).await?;
        let profile: Version = serde_json::from_str(&profile_json)?;

        Ok(InstalledModLoader {
            loader: crate::value::ModLoaderType::Quilt,
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
    fn test_quilt_installer_creation() {
        let client = reqwest::Client::new();
        let _installer = QuiltInstaller::new(client);
    }
}