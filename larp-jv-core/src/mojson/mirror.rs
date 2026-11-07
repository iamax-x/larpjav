//! Mirror handling for MoJson

use crate::value::{DownloadMirror, MirrorTamperedException};
use crate::error::{LarpError, LarpResult, MoJsonError};
use reqwest::Client;
use std::collections::HashMap;

/// Mirror manager
pub struct MirrorManager {
    mirrors: HashMap<String, Vec<Mirror>>,
    client: Client,
}

#[derive(Debug, Clone)]
pub struct Mirror {
    pub url: String,
    pub priority: i32,
    pub verified: bool,
}

impl MirrorManager {
    pub fn new() -> Self {
        Self {
            mirrors: HashMap::new(),
            client: Client::new(),
        }
    }

    pub fn with_client(client: Client) -> Self {
        Self {
            mirrors: HashMap::new(),
            client,
        }
    }

    /// Add mirror for a domain
    pub fn add_mirror(&mut self, domain: String, mirror: Mirror) {
        self.mirrors.entry(domain).or_default().push(mirror);
    }

    /// Get mirrors for URL
    pub fn get_mirrors(&self, url: &str) -> Vec<&Mirror> {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(domain) = parsed.domain() {
                if let Some(mirrors) = self.mirrors.get(domain) {
                    let mut result: Vec<_> = mirrors.iter().collect();
                    result.sort_by_key(|m| m.priority);
                    return result;
                }
            }
        }
        vec![]
    }

    /// Try download from mirrors
    pub async fn try_download(
        &self,
        original_url: &str,
        dest: &std::path::Path,
    ) -> LarpResult<u64> {
        let mirrors = self.get_mirrors(original_url);

        if mirrors.is_empty() {
            // Try original URL
            return crate::utils::download::download_file(&self.client, original_url, dest, None, false).await;
        }

        for mirror in mirrors {
            let mirror_url = original_url.replacen(
                &url::Url::parse(original_url).unwrap().domain().unwrap_or(""),
                &mirror.url,
            );
            let result = crate::utils::download::download_file(&self.client, &mirror_url, dest, None, false).await;
            if result.is_ok() {
                return result;
            }
        }

        Err(LarpError::MoJson(MoJsonError::AllMirrorsFailed))
    }
}

impl Default for MirrorManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Default mirrors for common repositories
pub fn default_mirrors() -> HashMap<String, Vec<Mirror>> {
    let mut mirrors = HashMap::new();

    mirrors.insert("repo1.maven.org".to_string(), vec![
        Mirror { url: "https://repo1.maven.org/maven2".to_string(), priority: 0, verified: true },
        Mirror { url: "https://maven-central.storage-download.googleapis.com/maven2".to_string(), priority: 1, verified: true },
    ]);

    mirrors.insert("libraries.minecraft.net".to_string(), vec![
        Mirror { url: "https://libraries.minecraft.net".to_string(), priority: 0, verified: true },
    ]);

    mirrors.insert("maven.minecraftforge.net".to_string(), vec![
        Mirror { url: "https://maven.minecraftforge.net".to_string(), priority: 0, verified: true },
    ]);

    mirrors.insert("maven.fabricmc.net".to_string(), vec![
        Mirror { url: "https://maven.fabricmc.net".to_string(), priority: 0, verified: true },
    ]);

    mirrors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirror_manager() {
        let mut mm = MirrorManager::new();
        mm.add_mirror("example.com".to_string(), Mirror { url: "https://mirror.example.com".to_string(), priority: 0, verified: true });
        let mirrors = mm.get_mirrors("https://example.com/path/file.jar");
        assert_eq!(mirrors.len(), 1);
    }

    #[test]
    fn test_default_mirrors() {
        let mirrors = default_mirrors();
        assert!(mirrors.contains_key("repo1.maven.org"));
        assert!(mirrors.contains_key("libraries.minecraft.net"));
    }
}