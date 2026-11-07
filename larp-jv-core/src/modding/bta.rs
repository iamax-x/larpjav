//! BTA (Better Than Adventure) mod installer

use crate::error::{LarpError, LarpResult, ModdingError};
use crate::modding::{ModLoader, InstalledModLoader};
use crate::value::Version;
use crate::utils::download::download_file;
use reqwest::Client;
use std::path::Path;

/// BTA installer
pub struct BTAInstaller {
    client: Client,
}

impl BTAInstaller {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Get BTA versions
    pub async fn get_versions(&self) -> LarpResult<Vec<BTAVersion>> {
        // BTA versions from GitHub releases
        let url = "https://api.github.com/repos/Better-Than-Adventure/Better-Than-Adventure/releases";
        let resp = self.client.get(url).send().await?;
        let json: serde_json::Value = resp.json().await?;

        let mut versions = Vec::new();
        if let Some(arr) = json.as_array() {
            for item in arr {
                if let Some(tag) = item.get("tag_name").and_then(|v| v.as_str()) {
                    versions.push(BTAVersion {
                        version: tag.trim_start_matches('v').to_string(),
                        url: item.get("assets").and_then(|a| a.as_array()).and_then(|arr| arr.first()).and_then(|a| a.get("browser_download_url")).and_then(|u| u.as_str()).unwrap_or("").to_string(),
                    });
                }
            }
        }

        Ok(versions)
    }

    /// Download BTA
    pub async fn download(&self, version: &BTAVersion, dest: &Path) -> LarpResult<std::path::PathBuf> {
        let dest_path = dest.join(format!("BTA-{}.jar", version.version));
        download_file(&self.client, &version.url, &dest_path, None, false).await?;
        Ok(dest_path)
    }

    /// Install BTA
    pub async fn install(
        &self,
        bta_path: &Path,
        dest_dir: &Path,
        mc_version: &str,
    ) -> LarpResult<()> {
        // BTA is installed as a mod/tweak
        let output = tokio::process::Command::new("java")
            .arg("-jar")
            .arg(bta_path)
            .arg("--install")
            .arg(dest_dir)
            .output()
            .await?;

        if !output.status.success() {
            return Err(LarpError::Modding(ModdingError::BTA(
                String::from_utf8_lossy(&output.stderr).to_string(),
            )));
        }

        Ok(())
    }
}

impl ModLoader for BTAInstaller {
    fn loader_type(&self) -> crate::value::ModLoaderType {
        crate::value::ModLoaderType::BTA
    }

    fn name(&self) -> &str {
        "Better Than Adventure"
    }

    async fn available_versions(&self, _mc_version: &str) -> LarpResult<Vec<String>> {
        let versions = self.get_versions().await?;
        Ok(versions.into_iter().map(|v| v.version).collect())
    }

    async fn install(
        &self,
        mc_version: &str,
        loader_version: &str,
        dest_dir: &Path,
        _profile: &Version,
    ) -> LarpResult<InstalledModLoader> {
        let versions = self.get_versions().await?;
        let version = versions.into_iter().find(|v| v.version == loader_version)
            .ok_or_else(|| LarpError::Modding(ModdingError::BTA("Version not found".to_string())))?;

        let bta_path = self.download(&version, dest_dir).await?;
        self.install(&bta_path, dest_dir, mc_version).await?;

        Ok(InstalledModLoader {
            loader: crate::value::ModLoaderType::BTA,
            version: loader_version.to_string(),
            mc_version: mc_version.to_string(),
            libraries: vec![],
            main_class: "bta.BTA".to_string(),
            tweak_class: Some("bta.BTATweaker".to_string()),
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

/// BTA version info
#[derive(Debug, Clone)]
pub struct BTAVersion {
    pub version: String,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bta_installer_creation() {
        let client = reqwest::Client::new();
        let _installer = BTAInstaller::new(client);
    }
}