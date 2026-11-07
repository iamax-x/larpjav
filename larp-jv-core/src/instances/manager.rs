//! Instance manager extensions

use crate::instances::InstanceManager;
use crate::error::LarpResult;
use crate::value::Instance;
use std::path::Path;

impl InstanceManager {
    /// Export instance to file
    pub fn export_instance(&self, id: &str, dest: &Path) -> LarpResult<()> {
        if let Some(instance) = self.get_instance(id) {
            let content = toml::to_string_pretty(instance)?;
            std::fs::write(dest, content)?;
            Ok(())
        } else {
            Err(crate::error::LarpError::Instance(crate::error::InstanceError::NotFound(id.to_string())))
        }
    }

    /// Import instance from file
    pub fn import_instance(&mut self, path: &Path) -> LarpResult<Instance> {
        let content = std::fs::read_to_string(path)?;
        let instance: Instance = toml::from_str(&content)?;
        let new_id = uuid::Uuid::new_v4().to_string();
        let mut instance = instance;
        instance.id = new_id;
        self.save_instance(&instance)?;
        self.instances.insert(instance.id.clone(), instance.clone());
        Ok(instance)
    }

    /// Duplicate instance
    pub fn duplicate_instance(&mut self, id: &str, new_name: String) -> LarpResult<Instance> {
        if let Some(instance) = self.get_instance(id).cloned() {
            let mut new_instance = instance;
            new_instance.id = uuid::Uuid::new_v4().to_string();
            new_instance.name = new_name;
            new_instance.created_at = chrono::Utc::now();
            new_instance.last_played = None;
            new_instance.play_time = 0;
            self.save_instance(&new_instance)?;
            self.instances.insert(new_instance.id.clone(), new_instance.clone());
            Ok(new_instance)
        } else {
            Err(crate::error::LarpError::Instance(crate::error::InstanceError::NotFound(id.to_string())))
        }
    }
}
