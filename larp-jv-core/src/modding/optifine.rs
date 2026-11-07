//! OptiFine mod installer

use crate::error::{LarpError, LarpResult, ModdingError};
use crate::modding::{ModLoader, InstalledModLoader};
use crate::value::Version;
use crate::utils::download::download_file;
use reqwest::Client;
use std::path::Path;

/// OptiFine installer
pub struct OptiFineInstaller {
    client: Client,
}

impl OptiFineInstaller {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Get OptiFine versions for Minecraft version
    pub async fn get_versions(&self, mc_version: &str) -> LarpResult<Vec<OptiFineVersion>> {
        // OptiFine doesn't have a public API, so we scrape the website
        let url = format!("https://optifine.net/downloads?version={}", mc_version);
        let resp = self.client.get(&url).send().await?;
        let html = resp.text().await?;

        let mut versions = Vec::new();
        // Parse HTML for versions - simplified
        // In practice, this would use a proper HTML parser

        Ok(versions)
    }

    /// Download OptiFine
    pub async fn download(&self, version: &OptiFineVersion, dest: &Path) -> LarpResult<std::path::PathBuf> {
        let dest_path = dest.join(format!("OptiFine_{}.jar", version.version));
        download_file(&self.client, &version.url, &dest_path, None, false).await?;
        Ok(dest_path)
    }

    /// Install OptiFine (run as library)
    pub async fn install(
        &self,
        optifine_path: &Path,
        dest_dir: &Path,
        mc_version: &str,
    ) -> LarpResult<()> {
        // OptiFine is installed by adding it as a library/tweak
        // This is a placeholder - actual implementation would modify the version JSON
        let output = tokio::process::Command::new("java")
            .arg("-jar")
            .arg(optifine_path)
            .arg("--install")
            .arg(dest_dir)
            .output()
            .await?;

        if !output.status.success() {
            return Err(LarpError::Modding(ModdingError::OptiFine(
                String::from_utf8_lossy(&output.stderr).to_string(),
            )));
        }

        Ok(())
    }
}

impl ModLoader for OptiFineInstaller {
    fn loader_type(&self) -> crate::value::ModLoaderType {
        crate::value::ModLoaderType::Optifine
    }

    fn name(&self) -> &str {
        "OptiFine"
    }

    async fn available_versions(&self, mc_version: &str) -> LarpResult<Vec<String>> {
        let versions = self.get_versions(mc_version).await?;
        Ok(versions.into_iter().map(|v| v.version).collect())
    }

    async fn install(
        &self,
        mc_version: &str,
        loader_version: &str,
        dest_dir: &Path,
        _profile: &Version,
    ) -> LarpResult<InstalledModLoader> {
        let versions = self.get_versions(mc_version).await?;
        let version = versions.into_iter().find(|v| v.version == loader_version)
            .ok_or_else(|| LarpError::Modding(ModdingError::OptiFine("Version not found".to_string())))?;

        let optifine_path = self.download(&version, dest_dir).await?;
        self.install(&optifine_path, dest_dir, mc_version).await?;

        Ok(InstalledModLoader {
            loader: crate::value::ModLoaderType::Optifine,
            version: loader_version.to_string(),
            mc_version: mc_version.to_string(),
            libraries: vec![],
            main_class: "optifine.OptiFine".to_string(),
            tweak_class: Some("optifine.OptiFineTweaker".to_string()),
            arguments: crate::value::Arguments::default(),
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

/// OptiFine version info
#[derive(Debug, Clone)]
pub struct OptiFineVersion {
    pub version: String,
    pub mc_version: String,
    pub url: String,
    pub patch: String,
    pub preview: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optifine_installer_creation() {
        let client = reqwest::Client::new();
        let _installer = OptiFineInstaller::new(client);
    }
}