//! Instance model types

use crate::value::{Instance, Resolution, ModInfo};

/// Instance filter for listing
#[derive(Debug, Clone, Default)]
pub struct InstanceFilter {
    pub name_contains: Option<String>,
    pub version: Option<String>,
    pub has_mods: Option<bool>,
}

impl InstanceFilter {
    pub fn matches(&self, instance: &Instance) -> bool {
        if let Some(name) = &self.name_contains {
            if !instance.name.to_lowercase().contains(&name.to_lowercase()) {
                return false;
            }
        }
        if let Some(version) = &self.version {
            if instance.version_id != *version {
                return false;
            }
        }
        if let Some(has_mods) = self.has_mods {
            if (instance.mods.is_empty()) == has_mods {
                return false;
            }
        }
        true
    }
}

/// Instance sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceSort {
    Name,
    CreatedAt,
    LastPlayed,
    PlayTime,
    Version,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_filter() {
        let filter = InstanceFilter::default();
        assert!(filter.matches(&crate::value::Instance {
            id: "1".to_string(),
            name: "Test".to_string(),
            version_id: "1.20.4".to_string(),
            game_dir: "/tmp".to_string(),
            java_path: None,
            jvm_args: vec![],
            game_args: vec![],
            account_id: None,
            icon: None,
            created_at: chrono::Utc::now(),
            last_played: None,
            play_time: 0,
            custom_resolution: None,
            mods: vec![],
            resource_packs: vec![],
            shader_pack: None,
            environment: std::collections::HashMap::new(),
        }));
    }
}
