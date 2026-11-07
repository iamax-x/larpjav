//! Instance Installer — Port of InstanceInstaller.java
//! Profile Compatibility — Port of Profiles, ProfileBody, ProfileWatcher

use crate::value::{Instance, Version, Library, Rule, ExtractSettings};
use crate::error::LarpResult;
use crate::modding::ModLoaderInstaller;
use crate::mojson::MoJsonManager;
use crate::jre::JreManager;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Instance Installer — Port of InstanceInstaller.java
pub struct InstanceInstaller {
    mojson: Arc<MoJsonManager>,
    jre_manager: Arc<JreManager>,
    mod_loader_installers: Vec<Box<dyn ModLoaderInstaller>>,
}

impl InstanceInstaller {
    pub fn new(
        mojson: Arc<MoJsonManager>,
        jre_manager: Arc<JreManager>,
    ) -> Self {
        Self {
            mojson,
            jre_manager,
            mod_loader_installers: Vec::new(),
        }
    }

    pub fn register_mod_loader(&mut self, installer: Box<dyn ModLoaderInstaller>) {
        self.mod_loader_installers.push(installer);
    }

    /// Install a version (vanilla or modded)
    pub async fn install_version(&self, instance: &mut Instance, version_id: &str) -> LarpResult<()> {
        // 1. Resolve version manifest
        let version = self.mojson.get_version(version_id).await?;
        
        // 2. Download libraries
        for lib in &version.libraries {
            if self.check_rules(&lib.rules)? {
                let artifact = lib.downloads.artifact.as_ref()
                    .ok_or_else(|| crate::error::LarpError::Other("Missing artifact".to_string()))?;
                let dest = instance.get_libraries_dir().join(&lib.name.to_path());
                self.mojson.download_file(&artifact.url, &dest, artifact.sha1.as_deref()).await?;
                
                // Extract natives if needed
                if let Some(natives) = &lib.downloads.classifiers {
                    for (classifier, download) in natives {
                        if classifier.contains("natives") {
                            let native_dest = instance.get_natives_dir().join(format!("{}-{}.jar", lib.name, classifier));
                            self.mojson.download_file(&download.url, &native_dest, download.sha1.as_deref()).await?;
                            // Extract natives
                            crate::utils::zip::extract_jar(&native_dest, instance.get_natives_dir())?;
                        }
                    }
                }
            }
        }

        // 3. Download client jar
        if let Some(client) = &version.downloads.client {
            let dest = instance.get_versions_dir().join(version_id).join(format!("{}.jar", version_id));
            tokio::fs::create_dir_all(dest.parent().unwrap()).await?;
            self.mojson.download_file(&client.url, &dest, client.sha1.as_deref()).await?;
        }

        // 4. Download assets
        if let Some(asset_index) = &version.asset_index {
            let dest = instance.get_assets_dir().join("indexes").join(format!("{}.json", asset_index.id));
            tokio::fs::create_dir_all(dest.parent().unwrap()).await?;
            self.mojson.download_file(&asset_index.url, &dest, asset_index.sha1.as_deref()).await?;
            
            // Download asset files (would parse index and download each)
        }

        instance.version = Some(version_id.to_string());
        Ok(())
    }

    /// Install mod loader on top of vanilla
    pub async fn install_mod_loader(
        &self,
        instance: &mut Instance,
        loader_name: &str,
        loader_version: &str,
        mc_version: &str,
    ) -> LarpResult<()> {
        let installer = self.mod_loader_installers.iter()
            .find(|i| i.name() == loader_name)
            .ok_or_else(|| crate::error::LarpError::Other(format!("Loader not found: {}", loader_name)))?;
        
        installer.install(mc_version, loader_version, instance.get_instance_dir(), &mut |_| {}).await?;
        
        instance.mod_loader = Some(loader_name.to_string());
        instance.mod_loader_version = Some(loader_version.to_string());
        Ok(())
    }

    fn check_rules(&self, rules: &[Rule]) -> LarpResult<bool> {
        for rule in rules {
            let action = rule.action.as_deref().unwrap_or("allow");
            let os_match = rule.os.as_ref().map(|os| {
                os.name.as_deref().map(|n| n == std::env::consts::OS).unwrap_or(true)
            }).unwrap_or(true);
            
            if !os_match {
                return Ok(action == "disallow");
            }
        }
        Ok(true)
    }
}

/// Profile Compatibility — Port of Profiles.java, ProfileBody.java
pub mod profcompat {
    use crate::value::Instance;
    use crate::error::LarpResult;
    use std::path::Path;
    use std::collections::HashMap;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct ProfileBody {
        pub name: String,
        pub version: String,
        pub game_dir: Option<String>,
        pub java_args: Option<String>,
        pub memory: Option<u64>,
        pub mod_loader: Option<String>,
        pub mod_loader_version: Option<String>,
        pub custom_resolution: Option<(u32, u32)>,
        pub icon: Option<String>,
        pub last_played: Option<u64>,
        pub created: u64,
        pub extra: HashMap<String, serde_json::Value>,
    }

    impl ProfileBody {
        pub fn new(name: String, version: String) -> Self {
            Self {
                name,
                version,
                game_dir: None,
                java_args: None,
                memory: None,
                mod_loader: None,
                mod_loader_version: None,
                custom_resolution: None,
                icon: None,
                last_played: None,
                created: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                extra: HashMap::new(),
            }
        }

        pub fn to_instance(&self, instance_dir: &Path) -> Instance {
            Instance {
                name: self.name.clone(),
                version: Some(self.version.clone()),
                game_dir: self.game_dir.as_ref().map(Path::new).map(|p| p.to_path_buf()),
                java_args: self.java_args.clone(),
                memory: self.memory,
                mod_loader: self.mod_loader.clone(),
                mod_loader_version: self.mod_loader_version.clone(),
                custom_resolution: self.custom_resolution,
                icon: self.icon.clone(),
                last_played: self.last_played,
                created: self.created,
                ..Default::default()
            }
        }

        pub fn from_instance(instance: &Instance) -> Self {
            Self {
                name: instance.name.clone(),
                version: instance.version.clone().unwrap_or_default(),
                game_dir: instance.game_dir.as_ref().map(|p| p.to_string_lossy().to_string()),
                java_args: instance.java_args.clone(),
                memory: instance.memory,
                mod_loader: instance.mod_loader.clone(),
                mod_loader_version: instance.mod_loader_version.clone(),
                custom_resolution: instance.custom_resolution,
                icon: instance.icon.clone(),
                last_played: instance.last_played,
                created: instance.created,
                extra: instance.extra.clone(),
            }
        }
    }

    /// Profiles Manager — Port of Profiles.java
    pub struct Profiles {
        profiles_dir: std::path::PathBuf,
        profiles: HashMap<String, ProfileBody>,
    }

    impl Profiles {
        pub fn new(profiles_dir: &Path) -> Self {
            Self {
                profiles_dir: profiles_dir.to_path_buf(),
                profiles: HashMap::new(),
            }
        }

        pub fn load(&mut self) -> LarpResult<()> {
            tokio::fs::create_dir_all(&self.profiles_dir).await?;
            let entries = tokio::fs::read_dir(&self.profiles_dir).await?;
            // Load each profile.json
            Ok(())
        }

        pub fn save(&self) -> LarpResult<()> {
            for (name, profile) in &self.profiles {
                let path = self.profiles_dir.join(name).join("profile.json");
                tokio::fs::create_dir_all(path.parent().unwrap()).await?;
                let json = serde_json::to_string_pretty(profile)?;
                tokio::fs::write(path, json).await?;
            }
            Ok(())
        }

        pub fn add(&mut self, profile: ProfileBody) {
            self.profiles.insert(profile.name.clone(), profile);
        }

        pub fn remove(&mut self, name: &str) -> Option<ProfileBody> {
            self.profiles.remove(name)
        }

        pub fn get(&self, name: &str) -> Option<&ProfileBody> {
            self.profiles.get(name)
        }

        pub fn list(&self) -> Vec<&ProfileBody> {
            self.profiles.values().collect()
        }
    }

    /// Profile Watcher — Port of ProfileWatcher.java
    pub struct ProfileWatcher {
        // Would use notify crate to watch for changes
    }

    impl ProfileWatcher {
        pub fn new() -> Self {
            Self {}
        }

        pub fn watch(&mut self, _profiles_dir: &Path) -> LarpResult<()> {
            Ok(())
        }
    }
}