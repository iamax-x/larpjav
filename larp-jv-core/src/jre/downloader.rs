//! JRE downloader

use crate::jre::JreInstallation;
use crate::error::LarpResult;
use crate::utils::download::{create_client, download_file};
use reqwest::Client;
use std::path::Path;

/// Download JRE from Adoptium
pub async fn download_jre_adoptium(
    version: &str,
    dest_dir: &Path,
    client: &Client,
) -> LarpResult<JreInstallation> {
    let arch = current_arch();
    let url = format!(
        "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/jdk/hotspot/normal/eclipse",
        version, arch
    );

    let dest = dest_dir.join(format!("jdk-{}-{}", version, arch));
    std::fs::create_dir_all(&dest)?;

    download_file(client, &url, &dest, None, false).await?;

    // Extract if tar.gz
    if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
        // Use tar extraction
        let file = tokio::fs::File::open(&dest).await?;
        let mut archive = tar::Archive::new(file);
        archive.unpack(dest.parent().unwrap())?;
    }

    let java_exe = if cfg!(target_os = "windows") {
        dest.join("bin").join("java.exe")
    } else {
        dest.join("bin").join("java")
    };

    Ok(JreInstallation {
        version: version.to_string(),
        path: dest,
        java_exe,
        vendor: "Eclipse Adoptium".to_string(),
        architecture: arch.to_string(),
    })
}

/// Download JRE from Oracle (requires license acceptance)
pub async fn download_jre_oracle(
    version: &str,
    dest_dir: &Path,
    client: &Client,
) -> LarpResult<JreInstallation> {
    // Oracle downloads require cookie acceptance
    // This is a placeholder - actual implementation would need to handle Oracle's license
    Err(crate::error::LarpError::Jre(crate::error::JreError::NotFound { version: version.to_string() }))
}

/// Get available JRE versions from Adoptium
pub async fn available_versions(client: &Client) -> crate::error::LarpResult<Vec<String>> {
    let url = "https://api.adoptium.net/v3/info/release_types";
    let resp = client.get(url).send().await?;
    let json: serde_json::Value = resp.json().await?;

    let mut versions = Vec::new();
    if let Some(arr) = json.as_array() {
        for item in arr {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                versions.push(name.to_string());
            }
        }
    }

    Ok(versions)
}

fn current_arch() -> &'static str {
    #[cfg(target_arch = "x86_64")] return "x64";
    #[cfg(target_arch = "aarch64")] return "aarch64";
    #[cfg(target_arch = "x86")] return "x86";
    "x64"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_arch() {
        assert!(!current_arch().is_empty());
    }
}