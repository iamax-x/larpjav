//! MoJson (Mojang JSON) parsing and handling

pub mod manifest;
pub mod rules;
pub mod library;
pub mod mirror;
pub mod substitution;
pub mod native;
pub mod client;

use crate::error::{LarpError, LarpResult, MoJsonError};
use crate::value::{Version, VersionManifest, Library, Rule, LibraryArtifact, DependentLibrary, LibrarySubstitution, SubstitutionMap, ClientInfo, NativeLibraryExtractable, ExtractSettings, MoJsonRule};
use crate::utils::download::{create_client, download_file};
use reqwest::Client;
use std::path::Path;
use std::collections::HashMap;

/// MoJson manager - handles version manifest, libraries, rules
pub struct MoJsonManager {
    client: Client,
    cache_dir: std::path::PathBuf,
    mirror_manager: MirrorManager,
}

impl MoJsonManager {
    pub fn new(cache_dir: impl AsRef<Path>) -> LarpResult<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            client: create_client()?,
            cache_dir,
            mirror_manager: MirrorManager::default(),
        })
    }

    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    /// Fetch version manifest
    pub async fn fetch_version_manifest(&self) -> LarpResult<VersionManifest> {
        const MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
        let resp = self.client.get(MANIFEST_URL).send().await?;
        if !resp.status().is_success() {
            return Err(LarpError::MoJson(MoJsonError::InvalidManifest("Failed to fetch manifest".to_string())));
        }
        Ok(resp.json().await?)
    }

    /// Fetch version JSON
    pub async fn fetch_version(&self, version_url: &str) -> LarpResult<Version> {
        let resp = self.client.get(version_url).send().await?;
        if !resp.status().is_success() {
            return Err(LarpError::MoJson(MoJsonError::InvalidManifest("Failed to fetch version".to_string())));
        }
        Ok(resp.json().await?)
    }

    /// Resolve libraries for a version
    pub async fn resolve_libraries(
        &self,
        version: &Version,
        os: &str,
        arch: &str,
        features: &HashMap<String, bool>,
    ) -> LarpResult<Vec<ResolvedLibrary>> {
        let mut resolved = Vec::new();

        for library in &version.libraries {
            if rules::evaluate_rules(&library.rules, os, arch, features) {
                let artifact = library.downloads.as_ref().and_then(|d| d.artifact.clone());
                let native_classifier = library.natives.as_ref().and_then(|n| n.get(os).cloned());

                if let Some(artifact) = artifact {
                    resolved.push(ResolvedLibrary {
                        name: library.name.clone(),
                        url: artifact.url,
                        sha1: artifact.sha1,
                        size: artifact.size,
                        path: artifact.path,
                        native_classifier: native_classifier.clone(),
                        extract_rules: library.extract_rules.clone(),
                    });
                }

                // Handle classifiers (natives)
                if let Some(classifiers) = library.downloads.as_ref().map(|d| &d.classifiers) {
                    for (classifier, artifact) in classifiers {
                        if native_classifier.as_ref() == Some(classifier) || classifier.contains(os) {
                            resolved.push(ResolvedLibrary {
                                name: library.name.clone(),
                                url: artifact.url,
                                sha1: artifact.sha1,
                                size: artifact.size,
                                path: artifact.path,
                                native_classifier: Some(classifier.clone()),
                                extract_rules: library.extract_rules.clone(),
                            });
                        }
                    }
                }
            }
        }

        Ok(resolved)
    }

    /// Download all libraries
    pub async fn download_libraries(
        &self,
        libraries: &[ResolvedLibrary],
        dest_dir: &Path,
        progress: Option<&mut dyn FnMut(u64, u64)>,
    ) -> LarpResult<Vec<std::path::PathBuf>> {
        let mut downloaded = Vec::new();
        let total = libraries.len() as u64;
        let mut completed = 0u64;

        for lib in libraries {
            let filename = lib.path.as_deref().unwrap_or_else(|| {
                let maven = crate::utils::maven::MavenCoordinate::parse(&lib.name).unwrap();
                maven.filename()
            });
            let dest = dest_dir.join(&filename);

            download_file(&self.client, &lib.url, &dest, progress, false).await?;

            // Verify hash
            crate::utils::hash::verify_hash(&dest, &lib.sha1, crate::utils::hash::HashAlgorithm::Sha1).await?;

            // Extract if needed
            if lib.extract_rules.is_some() {
                crate::utils::zip::extract_zip(&dest, dest_dir).await?;
            }

            downloaded.push(dest);
            completed += 1;
            if let Some(p) = progress {
                p(completed, total);
            }
        }

        Ok(downloaded)
    }

    /// Download version JSON and libraries
    pub async fn prepare_version(
        &self,
        version: &Version,
        dest_dir: &Path,
        os: &str,
        arch: &str,
        features: &HashMap<String, bool>,
    ) -> LarpResult<PreparedVersion> {
        let libraries = self.resolve_libraries(version, os, arch, features).await?;
        let lib_paths = self.download_libraries(&libraries, dest_dir, None).await?;

        // Download client JAR
        let client_jar = if let Some(download) = &version.download {
            let dest = dest_dir.join(format!("{}.jar", version.id));
            download_file(&self.client, &download.url, &dest, None, false).await?;
            crate::utils::hash::verify_hash(&dest, &download.sha1, crate::utils::hash::HashAlgorithm::Sha1).await?;
            Some(dest)
        } else {
            None
        };

        // Download asset index
        let asset_index = self.download_asset_index(&version.asset_index, dest_dir).await?;

        Ok(PreparedVersion {
            version: version.clone(),
            libraries: lib_paths,
            client_jar,
            asset_index,
            native_libraries: libraries.into_iter().filter(|l| l.native_classifier.is_some()).collect(),
        })
    }

    async fn download_asset_index(&self, asset_index: &crate::value::AssetIndex, dest_dir: &Path) -> LarpResult<std::path::PathBuf> {
        let dest = dest_dir.join("assets").join("indexes").join(format!("{}.json", asset_index.id));
        std::fs::create_dir_all(dest.parent().unwrap())?;
        download_file(&self.client, &asset_index.url, &dest, None, false).await?;
        crate::utils::hash::verify_hash(&dest, &asset_index.sha1, crate::utils::hash::HashAlgorithm::Sha1).await?;
        Ok(dest)
    }
}

/// Resolved library with download info
#[derive(Debug, Clone)]
pub struct ResolvedLibrary {
    pub name: String,
    pub url: String,
    pub sha1: String,
    pub size: u64,
    pub path: Option<String>,
    pub native_classifier: Option<String>,
    pub extract_rules: Option<crate::value::ExtractRules>,
}

/// Prepared version with all assets downloaded
#[derive(Debug, Clone)]
pub struct PreparedVersion {
    pub version: Version,
    pub libraries: Vec<std::path::PathBuf>,
    pub client_jar: Option<std::path::PathBuf>,
    pub asset_index: std::path::PathBuf,
    pub native_libraries: Vec<ResolvedLibrary>,
}

/// Mirror manager for alternative download sources
#[derive(Debug, Clone, Default)]
pub struct MirrorManager {
    mirrors: HashMap<String, Vec<String>>,
}

impl MirrorManager {
    pub fn add_mirror(&mut self, domain: String, urls: Vec<String>) {
        self.mirrors.insert(domain, urls);
    }

    pub fn get_mirrors(&self, url: &str) -> Vec<String> {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(domain) = parsed.domain() {
                return self.mirrors.get(domain).cloned().unwrap_or_default();
            }
        }
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_mojson_manager_creation() {
        let tmp = TempDir::new().unwrap();
        let manager = MoJsonManager::new(tmp.path()).unwrap();
    }

    #[test]
    fn test_mirror_manager() {
        let mut mm = MirrorManager::default();
        mm.add_mirror("repo1.maven.org".to_string(), vec!["https://mirror.example.com".to_string()]);
        let mirrors = mm.get_mirrors("https://repo1.maven.org/maven2/com/example/test.jar");
        assert_eq!(mirrors.len(), 1);
    }
}