//! Version management for instances

use crate::error::LarpResult;
use crate::mojson::MoJsonManager;
use crate::value::Version;
use std::path::Path;

/// Version resolver for instances
pub struct VersionResolver {
    mojson: MoJsonManager,
}

impl VersionResolver {
    pub fn new(mojson: MoJsonManager) -> Self {
        Self { mojson }
    }

    /// Resolve version for instance
    pub async fn resolve(&self, version_id: &str) -> LarpResult<Version> {
        // Try to get from cache or download
        let manifest = self.mojson.fetch_version_manifest().await?;
        let version_info = manifest.versions.iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| crate::error::LarpError::Instance(crate::error::InstanceError::VersionNotFound(version_id.to_string())))?;

        self.mojson.fetch_version(&version_info.url).await
    }

    /// Get compatible mod loaders for version
    pub async fn compatible_loaders(&self, version: &Version) -> LarpResult<Vec<crate::value::ModLoaderType>> {
        let mut loaders = Vec::new();

        // Check if Forge is available
        // This would check Forge's maven metadata
        loaders.push(crate::value::ModLoaderType::Forge);
        loaders.push(crate::value::ModLoaderType::Fabric);
        loaders.push(crate::value::ModLoaderType::Quilt);
        loaders.push(crate::value::ModLoaderType::NeoForge);

        Ok(loaders)
    }
}
