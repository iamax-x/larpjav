//! Forge mod loader installer

use crate::error::{LarpError, LarpResult, ModdingError};
use crate::modding::{ModLoader, InstalledModLoader};
use crate::value::Version;
use crate::utils::download::{create_client, download_file};
use reqwest::Client;
use std::path::Path;

/// Forge installer
pub struct ForgeInstaller {
    client: Client,
}

impl ForgeInstaller {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Get Forge versions for Minecraft version
    pub async fn get_versions(&self, mc_version: &str) -> LarpResult<Vec<ForgeVersion>> {
        let url = format!("https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml");
        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;

        // Parse XML for versions
        let mut versions = Vec::new();
        for line in text.lines() {
            if line.contains("<version>") && line.contains(mc_version) {
                let start = line.find("<version>").unwrap_or(0) + 9;
                let end = line.find("</version>").unwrap_or(line.len());
                if start < end {
                    versions.push(ForgeVersion {
                        version: line[start..end].to_string(),
                        mc_version: mc_version.to_string(),
                    });
                }
            }
        }

        Ok(versions)
    }

    /// Download Forge installer
    pub async fn download_installer(&self, version: &str, dest: &Path) -> LarpResult<std::path::PathBuf> {
        let url = format!(
            "https://maven.minecraftforge.net/net/minecraftforge/forge/{}/forge-{}-installer.jar",
            version, version
        );
        let dest_path = dest.join(format!("forge-{}-installer.jar", version));
        download_file(&self.client, &url, &dest_path, None, false).await?;
        Ok(dest_path)
    }

    /// Run Forge installer
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
            return Err(LarpError::Modding(ModdingError::Forge(
                String::from_utf8_lossy(&output.stderr).to_string(),
            )));
        }

        Ok(())
    }
}

impl ModLoader for ForgeInstaller {
    fn loader_type(&self) -> crate::value::ModLoaderType {
        crate::value::ModLoaderType::Forge
    }

    fn name(&self) -> &str {
        "Forge"
    }

    async fn available_versions(&self, mc_version: &str) -> LarpResult<Vec<String>> {
        self.get_versions(mc_version).await.map(|v| v.into_iter().map(|fv| fv.version).collect())
    }

    async fn install(
        &self,
        mc_version: &str,
        loader_version: &str,
        dest_dir: &Path,
        _profile: &Version,
    ) -> LarpResult<InstalledModLoader> {
        let installer = self.download_installer(loader_version, dest_dir).await?;
        self.run_installer(&installer, dest_dir).await?;

        // Read generated profile
        let profile_path = dest_dir.join("versions").join(format!("{}-forge-{}", mc_version, loader_version)).join(format!("{}-forge-{}.json", mc_version, loader_version));
        let profile_json = tokio::fs::read_to_string(&profile_path).await?;
        let profile: Version = serde_json::from_str(&profile_json)?;

        Ok(InstalledModLoader {
            loader: crate::value::ModLoaderType::Forge,
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
        let installer = self.download_installer(loader_version, std::path::Path::new("/tmp")).await?;
        // Extract installer profile
        // This is a placeholder - actual implementation would extract the profile from the installer JAR
        Ok(serde_json::json!({}))
    }
}

/// Forge version info
#[derive(Debug, Clone)]
pub struct ForgeVersion {
    pub version: String,
    pub mc_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forge_installer_creation() {
        let client = reqwest::Client::new();
        let _installer = ForgeInstaller::new(client);
    }
}