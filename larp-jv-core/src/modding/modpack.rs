//! Modpack (MRPACK) support

use crate::error::{LarpError, LarpResult};
use crate::utils::download::download_file;
use crate::utils::zip::extract_zip;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Modpack index (MRPACK format)
#[derive(Debug, Deserialize, Serialize)]
pub struct ModpackIndex {
    pub format_version: u32,
    pub game: String,
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<ModpackFile>,
    pub dependencies: ModpackDependencies,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModpackFile {
    pub path: String,
    pub downloads: Vec<String>,
    pub file_size: u64,
    pub hashes: ModpackHashes,
    pub env: ModpackEnv,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModpackHashes {
    pub sha1: String,
    pub sha256: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModpackEnv {
    pub client: ModpackEnvSide,
    pub server: ModpackEnvSide,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModpackEnvSide {
    pub required: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModpackDependencies {
    pub minecraft: String,
    pub forge: Option<String>,
    pub fabric_loader: Option<String>,
    pub quilt_loader: Option<String>,
    pub neoforge: Option<String>,
}

/// Modpack installer
pub struct ModpackInstaller {
    client: Client,
}

impl ModpackInstaller {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Install modpack from MRPACK file
    pub async fn install_mrpack(
        &self,
        mrpack_path: &Path,
        dest_dir: &Path,
    ) -> LarpResult<ModpackInstallResult> {
        // Extract MRPACK
        let extract_dir = dest_dir.join("modpack_extract");
        extract_zip(mrpack_path, &extract_dir).await?;

        // Read index.json
        let index_path = extract_dir.join("index.json");
        let index_json = tokio::fs::read_to_string(&index_path).await?;
        let index: ModpackIndex = serde_json::from_str(&index_json)?;

        // Download files
        let mut downloaded = Vec::new();
        for file in &index.files {
            let file_dest = dest_dir.join(&file.path);
            std::fs::create_dir_all(file_dest.parent().unwrap())?;

            // Try each download URL
            let mut success = false;
            for url in &file.downloads {
                let result = download_file(&self.client, url, &file_dest, None, false).await;
                if result.is_ok() {
                    success = true;
                    break;
                }
            }

            if !success {
                return Err(LarpError::Other(format!("Failed to download {}", file.path)));
            }

            // Verify hashes
            crate::utils::hash::verify_hash(&file_dest, &file.hashes.sha1, crate::utils::hash::HashAlgorithm::Sha1).await?;

            downloaded.push(file_dest);
        }

        Ok(ModpackInstallResult {
            index,
            files: downloaded,
        })
    }

    /// Install modpack from URL
    pub async fn install_from_url(
        &self,
        url: &str,
        dest_dir: &Path,
    ) -> LarpResult<ModpackInstallResult> {
        let mrpack_path = dest_dir.join("modpack.mrpack");
        download_file(&self.client, url, &mrpack_path, None, false).await?;
        self.install_mrpack(&mrpack_path, dest_dir).await
    }
}

#[derive(Debug, Clone)]
pub struct ModpackInstallResult {
    pub index: ModpackIndex,
    pub files: Vec<std::path::PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modpack_installer_creation() {
        let client = reqwest::Client::new();
        let _installer = ModpackInstaller::new(client);
    }
}