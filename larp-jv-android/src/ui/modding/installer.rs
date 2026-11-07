//! Unified Mod Installer — trait and implementations

use crate::error::LarpResult;
use larp_jv_core::value::{InstalledModLoader, ModLoaderType};
use async_trait::async_trait;
use std::path::Path;

/// Unified installer trait for all mod loaders
#[async_trait]
pub trait ModLoaderInstaller: Send + Sync {
    fn loader_type(&self) -> ModLoaderType;
    fn name(&self) -> &str;

    /// Get available versions for a Minecraft version
    async fn fetch_versions(&self, mc_version: &str) -> LarpResult<Vec<String>>;

    /// Install the mod loader
    async fn install(
        &self,
        mc_version: &str,
        loader_version: &str,
        dest_dir: &std::path::Path,
        profile: &crate::value::Version,
    ) -> LarpResult<InstalledModLoader>;

    /// Get installer profile JSON
    async fn get_installer_profile(
        &self,
        mc_version: &str,
        loader_version: &str,
    ) -> LarpResult<serde_json::Value>;
}

/// Forge installer implementation
pub mod forge {
    use super::*;
    use std::sync::Arc;

    pub struct ForgeInstaller {
        client: Arc<reqwest::Client>,
    }

    impl ForgeInstaller {
        pub fn new(client: Arc<reqwest::Client>) -> Self {
            Self { client }
        }
    }

    #[async_trait]
    impl ModLoaderInstaller for ForgeInstaller {
        fn loader_type(&self) -> ModLoaderType {
            ModLoaderType::Forge
        }

        fn name(&self) -> &str {
            "Forge"
        }

        async fn fetch_versions(&self, mc_version: &str) -> LarpResult<Vec<String>> {
            let url = format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{}/maven-metadata.xml", mc_version);
            let resp = self.client.get(&url).send().await?;
            let text = resp.text().await?;

            let mut versions = Vec::new();
            for line in text.lines() {
                if line.contains("<version>") && line.contains(mc_version) {
                    let start = line.find("<version>").unwrap_or(0) + 9;
                    let end = line.find("</version>").unwrap_or(line.len());
                    if start < end {
                        versions.push(line[start..end].to_string());
                    }
                }
            }
            Ok(versions)
        }

        async fn install(
            &self,
            mc_version: &str,
            loader_version: &str,
            dest_dir: &std::path::Path,
            _profile: &crate::value::Version,
        ) -> LarpResult<crate::value::InstalledModLoader> {
            let installer_url = format!(
                "https://maven.minecraftforge.net/net/minecraftforge/forge/{}/{}/forge-{}-{}-installer.jar",
                mc_version, loader_version, mc_version, loader_version
            );
            
            let dest = dest_dir.join(format!("forge-{}-{}-installer.jar", mc_version, loader_version));
            // Download installer
            // Run installer with --installClient
            
            Ok(crate::value::InstalledModLoader {
                loader: crate::value::ModLoaderType::Forge,
                version: loader_version.to_string(),
                mc_version: mc_version.to_string(),
                libraries: vec![],
                main_class: "net.minecraft.client.main.Main".to_string(),
                tweak_class: Some("net.minecraftforge.fml.common.launcher.FMLTweaker".to_string()),
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
}

/// Fabric installer implementation
pub mod fabric {
    use super::*;
    use std::sync::Arc;

    pub struct FabricInstaller {
        client: Arc<reqwest::Client>,
    }

    impl FabricInstaller {
        pub fn new(client: Arc<reqwest::Client>) -> Self {
            Self { client }
        }
    }

    #[async_trait]
    impl ModLoaderInstaller for FabricInstaller {
        fn loader_type(&self) -> ModLoaderType {
            ModLoaderType::Fabric
        }

        fn name(&self) -> &str {
            "Fabric"
        }

        async fn fetch_versions(&self, _mc_version: &str) -> LarpResult<Vec<String>> {
            let url = "https://meta.fabricmc.net/v2/versions/loader";
            let resp = self.client.get(url).send().await?;
            let json: serde_json::Value = resp.json().await?;

            let mut versions = Vec::new();
            if let Some(arr) = json.as_array() {
                for item in arr {
                    if let Some(version) = item.get("version").and_then(|v| v.as_str()) {
                        versions.push(version.to_string());
                    }
                }
            }
            Ok(versions)
        }

        async fn install(
            &self,
            mc_version: &str,
            loader_version: &str,
            dest_dir: &std::path::Path,
            _profile: &crate::value::Version,
        ) -> LarpResult<crate::value::InstalledModLoader> {
            let url = format!(
                "https://maven.fabricmc.net/net/fabricmc/fabric-installer/{}/fabric-installer-{}.jar",
                loader_version, loader_version
            );
            
            let dest = dest_dir.join(format!("fabric-installer-{}.jar", loader_version));
            // Download and run installer
            
            Ok(crate::value::InstalledModLoader {
                loader: crate::value::ModLoaderType::Fabric,
                version: loader_version.to_string(),
                mc_version: mc_version.to_string(),
                libraries: vec![],
                main_class: "com.mojang.blaze3d.platform.GlStateManager".to_string(),
                tweak_class: None,
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
}

/// Unified installer factory
pub struct InstallerFactory;

impl InstallerFactory {
    pub fn get_installer(loader: ModLoaderType, client: Arc<reqwest::Client>) -> Box<dyn ModLoaderInstaller> {
        match loader {
            ModLoaderType::Forge => Box::new(forge::ForgeInstaller::new(client.clone())),
            ModLoaderType::Fabric => Box::new(fabric::FabricInstaller::new(client.clone())),
            ModLoaderType::Quilt => Box::new(quilt::QuiltInstaller::new(client.clone())),
            ModLoaderType::NeoForge => Box::new(neoforge::NeoForgeInstaller::new(client.clone())),
            ModLoaderType::Optifine => Box::new(optifine::OptiFineInstaller::new(client.clone())),
            ModLoaderType::LegacyFabric => Box::new(legacy_fabric::LegacyFabricInstaller::new(client.clone())),
            ModLoaderType::BTA => Box::new(bta::BTAInstaller::new(client.clone())),
            _ => Box::new(fabric::FabricInstaller::new(client.clone())),
        }
    }
}

// Placeholder modules for other loaders
pub mod quilt {
    use super::*;
    use std::sync::Arc;

    pub struct QuiltInstaller { client: Arc<reqwest::Client> }
    impl QuiltInstaller { pub fn new(client: Arc<reqwest::Client>) -> Self { Self { client } } }
    #[async_trait] impl super::ModLoaderInstaller for QuiltInstaller {
        fn loader_type(&self) -> ModLoaderType { ModLoaderType::Quilt }
        fn name(&self) -> &str { "Quilt" }
        async fn fetch_versions(&self, _mc: &str) -> LarpResult<Vec<String>> { Ok(vec![]) }
        async fn install(&self, mc: &str, lv: &str, dest: &std::path::Path, _p: &crate::value::Version) -> LarpResult<crate::value::InstalledModLoader> { Ok(crate::value::InstalledModLoader { loader: ModLoaderType::Quilt, version: lv.to_string(), mc_version: mc.to_string(), libraries: vec![], main_class: "".to_string(), tweak_class: None, arguments: crate::value::Arguments::default() }) }
        async fn get_installer_profile(&self, _mc: &str, _lv: &str) -> LarpResult<serde_json::Value> { Ok(serde_json::json!({})) }
    }
}

pub mod neoforge {
    use super::*;
    use std::sync::Arc;
    pub struct NeoForgeInstaller { client: Arc<reqwest::Client> }
    impl NeoForgeInstaller { pub fn new(client: Arc<reqwest::Client>) -> Self { Self { client } } }
    #[async_trait] impl super::ModLoaderInstaller for NeoForgeInstaller {
        fn loader_type(&self) -> ModLoaderType { ModLoaderType::NeoForge }
        fn name(&self) -> &str { "NeoForge" }
        async fn fetch_versions(&self, mc: &str) -> LarpResult<Vec<String>> { Ok(vec![]) }
        async fn install(&self, mc: &str, lv: &str, dest: &std::path::Path, _p: &crate::value::Version) -> LarpResult<crate::value::InstalledModLoader> { Ok(crate::value::InstalledModLoader { loader: ModLoaderType::NeoForge, version: lv.to_string(), mc_version: mc.to_string(), libraries: vec![], main_class: "".to_string(), tweak_class: None, arguments: crate::value::Arguments::default() }) }
        async fn get_installer_profile(&self, _mc: &str, _lv: &str) -> LarpResult<serde_json::Value> { Ok(serde_json::json!({})) }
    }
}

pub mod optifine {
    use super::*;
    use std::sync::Arc;
    pub struct OptiFineInstaller { client: Arc<reqwest::Client> }
    impl OptiFineInstaller { pub fn new(client: Arc<reqwest::Client>) -> Self { Self { client } } }
    #[async_trait] impl super::ModLoaderInstaller for OptiFineInstaller {
        fn loader_type(&self) -> ModLoaderType { ModLoaderType::Optifine }
        fn name(&self) -> &str { "OptiFine" }
        async fn fetch_versions(&self, _mc: &str) -> LarpResult<Vec<String>> { Ok(vec![]) }
        async fn install(&self, mc: &str, lv: &str, dest: &std::path::Path, _p: &crate::value::Version) -> LarpResult<crate::value::InstalledModLoader> { Ok(crate::value::InstalledModLoader { loader: ModLoaderType::Optifine, version: lv.to_string(), mc_version: mc.to_string(), libraries: vec![], main_class: "".to_string(), tweak_class: None, arguments: crate::value::Arguments::default() }) }
        async fn get_installer_profile(&self, _mc: &str, _lv: &str) -> LarpResult<serde_json::Value> { Ok(serde_json::json!({})) }
    }
}

pub mod legacy_fabric {
    use super::*;
    use std::sync::Arc;
    pub struct LegacyFabricInstaller { client: Arc<reqwest::Client> }
    impl LegacyFabricInstaller { pub fn new(client: Arc<reqwest::Client>) -> Self { Self { client } } }
    #[async_trait] impl super::ModLoaderInstaller for LegacyFabricInstaller {
        fn loader_type(&self) -> ModLoaderType { ModLoaderType::LegacyFabric }
        fn name(&self) -> &str { "Legacy Fabric" }
        async fn fetch_versions(&self, _mc: &str) -> LarpResult<Vec<String>> { Ok(vec![]) }
        async fn install(&self, mc: &str, lv: &str, dest: &std::path::Path, _p: &crate::value::Version) -> LarpResult<crate::value::InstalledModLoader> { Ok(crate::value::InstalledModLoader { loader: ModLoaderType::LegacyFabric, version: lv.to_string(), mc_version: mc.to_string(), libraries: vec![], main_class: "".to_string(), tweak_class: None, arguments: crate::value::Arguments::default() }) }
        async fn get_installer_profile(&self, _mc: &str, _lv: &str) -> LarpResult<serde_json::Value> { Ok(serde_json::json!({})) }
    }
}

pub mod bta {
    use super::*;
    use std::sync::Arc;
    pub struct BTAInstaller { client: Arc<reqwest::Client> }
    impl BTAInstaller { pub fn new(client: Arc<reqwest::Client>) -> Self { Self { client } } }
    #[async_trait] impl super::ModLoaderInstaller for BTAInstaller {
        fn loader_type(&self) -> ModLoaderType { ModLoaderType::BTA }
        fn name(&self) -> &str { "BTA" }
        async fn fetch_versions(&self, _mc: &str) -> LarpResult<Vec<String>> { Ok(vec![]) }
        async fn install(&self, mc: &str, lv: &str, dest: &std::path::Path, _p: &crate::value::Version) -> LarpResult<crate::value::InstalledModLoader> { Ok(crate::value::InstalledModLoader { loader: ModLoaderType::BTA, version: lv.to_string(), mc_version: mc.to_string(), libraries: vec![], main_class: "".to_string(), tweak_class: None, arguments: crate::value::Arguments::default() }) }
        async fn get_installer_profile(&self, _mc: &str, _lv: &str) -> LarpResult<serde_json::Value> { Ok(serde_json::json!({})) }
    }
}