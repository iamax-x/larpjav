//! Instance management

pub mod model;
pub mod manager;
pub mod version;

use crate::error::{LarpError, LarpResult, InstanceError};
use crate::value::{Instance, Resolution, ModInfo, ModLoaderType};
use crate::utils::file::{read_to_string, write_string, atomic_write};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Instance manager
pub struct InstanceManager {
    instances_dir: PathBuf,
    instances: HashMap<String, Instance>,
}

impl InstanceManager {
    pub fn new(instances_dir: impl AsRef<Path>) -> LarpResult<Self> {
        let instances_dir = instances_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&instances_dir)?;

        let mut manager = Self {
            instances_dir,
            instances: HashMap::new(),
        };

        manager.load_all()?;
        Ok(manager)
    }

    fn load_all(&mut self) -> LarpResult<()> {
        for entry in std::fs::read_dir(&self.instances_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                if let Ok(instance) = self.load_instance(&path) {
                    self.instances.insert(instance.id.clone(), instance);
                }
            }
        }
        Ok(())
    }

    fn load_instance(&self, path: &Path) -> LarpResult<Instance> {
        let content = read_to_string(path)?;
        let instance: Instance = toml::from_str(&content)?;
        Ok(instance)
    }

    fn instance_path(&self, id: &str) -> PathBuf {
        self.instances_dir.join(format!("{}.toml", id))
    }

    pub fn create_instance(
        &mut self,
        name: String,
        version_id: String,
        game_dir: PathBuf,
    ) -> LarpResult<Instance> {
        let id = uuid::Uuid::new_v4().to_string();
        let instance = Instance {
            id: id.clone(),
            name,
            version_id,
            game_dir: game_dir.to_string_lossy().to_string(),
            java_path: None,
            jvm_args: vec!["-Xmx2G".to_string(), "-Xms1G".to_string()],
            game_args: Vec::new(),
            account_id: None,
            icon: None,
            created_at: chrono::Utc::now(),
            last_played: None,
            play_time: 0,
            custom_resolution: None,
            mods: Vec::new(),
            resource_packs: Vec::new(),
            shader_pack: None,
            environment: HashMap::new(),
        };

        self.save_instance(&instance)?;
        self.instances.insert(id, instance.clone());
        Ok(instance)
    }

    pub fn save_instance(&self, instance: &Instance) -> LarpResult<()> {
        let path = self.instance_path(&instance.id);
        let content = toml::to_string_pretty(instance)?;
        atomic_write(&path, content.as_bytes())?;
        Ok(())
    }

    pub fn get_instance(&self, id: &str) -> Option<&Instance> {
        self.instances.get(id)
    }

    pub fn get_instance_mut(&mut self, id: &str) -> Option<&mut Instance> {
        self.instances.get_mut(id)
    }

    pub fn delete_instance(&mut self, id: &str) -> LarpResult<bool> {
        if self.instances.remove(id).is_some() {
            let path = self.instance_path(id);
            if path.exists() {
                std::fs::remove_file(path)?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn list_instances(&self) -> Vec<&Instance> {
        self.instances.values().collect()
    }

    pub fn add_mod(&mut self, instance_id: &str, mod_info: ModInfo) -> LarpResult<()> {
        if let Some(instance) = self.instances.get_mut(instance_id) {
            instance.mods.push(mod_info);
            self.save_instance(instance)?;
            Ok(())
        } else {
            Err(LarpError::Instance(InstanceError::NotFound(instance_id.to_string())))
        }
    }

    pub fn remove_mod(&mut self, instance_id: &str, mod_id: &str) -> LarpResult<bool> {
        if let Some(instance) = self.instances.get_mut(instance_id) {
            let len = instance.mods.len();
            instance.mods.retain(|m| m.id != mod_id);
            let removed = instance.mods.len() != len;
            if removed {
                self.save_instance(instance)?;
            }
            Ok(removed)
        } else {
            Err(LarpError::Instance(InstanceError::NotFound(instance_id.to_string())))
        }
    }

    pub fn set_resolution(&mut self, instance_id: &str, resolution: Resolution) -> LarpResult<()> {
        if let Some(instance) = self.instances.get_mut(instance_id) {
            instance.custom_resolution = Some(resolution);
            self.save_instance(instance)?;
            Ok(())
        } else {
            Err(LarpError::Instance(InstanceError::NotFound(instance_id.to_string())))
        }
    }

    pub fn update_play_time(&mut self, instance_id: &str, milliseconds: u64) -> LarpResult<()> {
        if let Some(instance) = self.instances.get_mut(instance_id) {
            instance.play_time += milliseconds;
            instance.last_played = Some(chrono::Utc::now());
            self.save_instance(instance)?;
            Ok(())
        } else {
            Err(LarpError::Instance(InstanceError::NotFound(instance_id.to_string())))
        }
    }
}

impl Default for InstanceManager {
    fn default() -> Self {
        Self::new("instances").expect("Failed to create InstanceManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_instance_manager() {
        let tmp = TempDir::new().unwrap();
        let mut manager = InstanceManager::new(tmp.path()).unwrap();

        let instance = manager.create_instance(
            "Test Instance".to_string(),
            "1.20.4".to_string(),
            tmp.path().join("game"),
        ).unwrap();

        assert_eq!(instance.name, "Test Instance");
        assert_eq!(manager.list_instances().len(), 1);
    }
}