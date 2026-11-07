//! Library handling for MoJson

use crate::value::{Library, LibraryArtifact, LibraryDownloads, ArtifactInfo, LibrarySubstitution, SubstitutionMap};
use crate::utils::maven::MavenCoordinate;
use std::collections::HashMap;

/// Library resolver
pub struct LibraryResolver {
    substitutions: SubstitutionMap,
}

impl LibraryResolver {
    pub fn new(substitutions: SubstitutionMap) -> Self {
        Self { substitutions }
    }

    /// Apply substitutions to library
    pub fn apply_substitution(&self, library: &mut Library) {
        for sub in &self.substitutions.libraries {
            if library.name == sub.name {
                library.name = sub.target.clone();
            }
        }
    }

    /// Get library artifact info
    pub fn get_artifact(&self, library: &Library) -> Option<&LibraryArtifact> {
        library.downloads.as_ref()?.artifact.as_ref()
    }

    /// Get native classifier for OS
    pub fn get_native_classifier(&self, library: &Library, os: &str) -> Option<&String> {
        library.natives.as_ref()?.get(os)
    }

    /// Check if library is native
    pub fn is_native(&self, library: &Library) -> bool {
        library.natives.is_some()
    }
}

/// Library downloader
pub struct LibraryDownloader {
    client: reqwest::Client,
}

impl LibraryDownloader {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// Download library artifact
    pub async fn download_artifact(
        &self,
        artifact: &LibraryArtifact,
        dest: &std::path::Path,
    ) -> crate::error::LarpResult<()> {
        crate::utils::download::download_file(&self.client, &artifact.url, dest, None, false).await?;
        Ok(())
    }

    /// Download all artifacts for library
    pub async fn download_library(
        &self,
        library: &Library,
        dest_dir: &std::path::Path,
    ) -> crate::error::LarpResult<Vec<std::path::PathBuf>> {
        let mut downloaded = Vec::new();

        if let Some(artifact) = self.get_artifact(library) {
            let filename = artifact.path.as_deref().unwrap_or_else(|| {
                let maven = MavenCoordinate::parse(&library.name).unwrap();
                maven.filename()
            });
            let dest = dest_dir.join(filename);
            self.download_artifact(artifact, &dest).await?;
            downloaded.push(dest);
        }

        // Download classifiers (natives)
        if let Some(classifiers) = library.downloads.as_ref().map(|d| &d.classifiers) {
            for (classifier, artifact) in classifiers {
                let filename = format!("{}-{}.jar", library.name.split(':').nth(1).unwrap_or("lib"), classifier);
                let dest = dest_dir.join(filename);
                self.download_artifact(artifact, &dest).await?;
                downloaded.push(dest);
            }
        }

        Ok(downloaded)
    }
}

impl Default for LibraryDownloader {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse maven coordinate from library name
pub fn parse_maven_coordinate(name: &str) -> crate::error::LarpResult<MavenCoordinate> {
    MavenCoordinate::parse(name)
}

/// Build library path in Maven repository
pub fn library_repo_path(coord: &MavenCoordinate) -> String {
    coord.repo_path()
}

/// Build library URL for repository
pub fn library_repo_url(coord: &MavenCoordinate, base_url: &str) -> String {
    coord.repo_url(base_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_resolver() {
        let resolver = LibraryResolver::new(SubstitutionMap { libraries: vec![] });
    }

    #[test]
    fn test_parse_maven() {
        let coord = parse_maven_coordinate("com.mojang:authlib:1.0.0").unwrap();
        assert_eq!(coord.group, "com.mojang");
        assert_eq!(coord.artifact, "authlib");
        assert_eq!(coord.version, "1.0.0");
    }
}