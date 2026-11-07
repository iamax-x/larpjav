//! Maven coordinate utilities

use crate::error::LarpResult;
use std::path::Path;

/// Maven coordinate (group:artifact:version[:classifier][:type])
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MavenCoordinate {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub classifier: Option<String>,
    pub extension: String, // default "jar"
}

impl MavenCoordinate {
    pub fn new(group: impl Into<String>, artifact: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            group: group.into(),
            artifact: artifact.into(),
            version: version.into(),
            classifier: None,
            extension: "jar".to_string(),
        }
    }

    pub fn with_classifier(mut self, classifier: impl Into<String>) -> Self {
        self.classifier = Some(classifier.into());
        self
    }

    pub fn with_extension(mut self, extension: impl Into<String>) -> Self {
        self.extension = extension.into();
        self
    }

    /// Parse from "group:artifact:version[:classifier][:type]"
    pub fn parse(s: &str) -> LarpResult<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        match parts.len() {
            3 => Ok(Self::new(parts[0], parts[1], parts[2])),
            4 => Ok(Self::new(parts[0], parts[1], parts[2]).with_classifier(parts[3])),
            5 => Ok(Self::new(parts[0], parts[1], parts[2])
                .with_classifier(parts[3])
                .with_extension(parts[4])),
            _ => Err(crate::error::LarpError::MoJson(
                crate::error::MoJsonError::InvalidLibraryFormat(s.to_string()),
            )),
        }
    }

    /// Convert to "group:artifact:version[:classifier][:type]"
    pub fn to_string(&self) -> String {
        match &self.classifier {
            Some(c) => format!("{}:{}:{}:{}:{}", self.group, self.artifact, self.version, c, self.extension),
            None => format!("{}:{}:{}", self.group, self.artifact, self.version),
        }
    }

    /// Get artifact filename (e.g., artifact-version-classifier.jar)
    pub fn filename(&self) -> String {
        match &self.classifier {
            Some(c) => format!("{}-{}-{}.{}", self.artifact, self.version, c, self.extension),
            None => format!("{}-{}.{}", self.artifact, self.version, self.extension),
        }
    }

    /// Get relative path in Maven repo (group/artifact/version/filename)
    pub fn repo_path(&self) -> String {
        let group_path = self.group.replace('.', "/");
        format!("{}/{}/{}/{}", group_path, self.artifact, self.version, self.filename())
    }

    /// Get URL for Maven Central
    pub fn maven_central_url(&self) -> String {
        format!("https://repo1.maven.org/maven2/{}", self.repo_path())
    }

    /// Get URL for custom repository
    pub fn repo_url(&self, base_url: &str) -> String {
        let base = base_url.trim_end_matches('/');
        format!("{}/{}", base, self.repo_path())
    }
}

impl std::fmt::Display for MavenCoordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::str::FromStr for MavenCoordinate {
    type Err = crate::error::LarpError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Resolve library from multiple repositories
pub async fn resolve_library(
    coord: &MavenCoordinate,
    repositories: &[String],
    client: &reqwest::Client,
) -> LarpResult<Option<String>> {
    for repo in repositories {
        let url = coord.repo_url(repo);
        let resp = client.head(&url).send().await?;
        if resp.status().is_success() {
            return Ok(Some(url));
        }
    }
    Ok(None)
}

/// Download library to local Maven cache
pub async fn download_to_cache(
    coord: &MavenCoordinate,
    cache_dir: &Path,
    client: &reqwest::Client,
    repositories: &[String],
) -> LarpResult<PathBuf> {
    let local_path = cache_dir.join(coord.repo_path());
    if local_path.exists() {
        return Ok(local_path);
    }

    if let Some(url) = resolve_library(coord, repositories, client).await? {
        use crate::utils::download::download_file;
        download_file(client, &url, &local_path, None, false).await?;
        return Ok(local_path);
    }

    Err(crate::error::LarpError::MoJson(
        crate::error::MoJsonError::LibraryNotFound {
            group: coord.group.clone(),
            artifact: coord.artifact.clone(),
            version: coord.version.clone(),
        },
    ))
}

use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_maven_coordinate() {
        let coord = MavenCoordinate::parse("com.mojang:authlib:1.0.0").unwrap();
        assert_eq!(coord.group, "com.mojang");
        assert_eq!(coord.artifact, "authlib");
        assert_eq!(coord.version, "1.0.0");
        assert_eq!(coord.classifier, None);
        assert_eq!(coord.extension, "jar");
    }

    #[test]
    fn test_parse_with_classifier() {
        let coord = MavenCoordinate::parse("org.lwjgl:lwjgl:3.3.1:natives-windows").unwrap();
        assert_eq!(coord.group, "org.lwjgl");
        assert_eq!(coord.artifact, "lwjgl");
        assert_eq!(coord.version, "3.3.1");
        assert_eq!(coord.classifier, Some("natives-windows".to_string()));
    }

    #[test]
    fn test_filename() {
        let coord = MavenCoordinate::new("com.mojang", "authlib", "1.0.0");
        assert_eq!(coord.filename(), "authlib-1.0.0.jar");

        let coord = coord.with_classifier("natives-windows");
        assert_eq!(coord.filename(), "authlib-1.0.0-natives-windows.jar");
    }

    #[test]
    fn test_repo_path() {
        let coord = MavenCoordinate::new("com.mojang", "authlib", "1.0.0");
        assert_eq!(coord.repo_path(), "com/mojang/authlib/1.0.0/authlib-1.0.0.jar");
    }
}
