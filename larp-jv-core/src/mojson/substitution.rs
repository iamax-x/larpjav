//! Substitution handling for MoJson

use crate::value::{LibrarySubstitution, SubstitutionMap};
use std::collections::HashMap;

/// Substitution manager
pub struct SubstitutionManager {
    substitutions: HashMap<String, LibrarySubstitution>,
}

impl SubstitutionManager {
    pub fn new(substitutions: SubstitutionMap) -> Self {
        let mut subs = HashMap::new();
        for sub in substitutions.libraries {
            subs.insert(sub.name.clone(), sub);
        }
        Self { substitutions: subs }
    }

    /// Apply substitution to library name
    pub fn apply(&self, name: &str) -> String {
        if let Some(sub) = self.substitutions.get(name) {
            sub.target.clone()
        } else {
            name.to_string()
        }
    }

    /// Check if substitution exists
    pub fn has_substitution(&self, name: &str) -> bool {
        self.substitutions.contains_key(name)
    }

    /// Get all substitutions
    pub fn all(&self) -> Vec<&LibrarySubstitution> {
        self.substitutions.values().collect()
    }
}

impl Default for SubstitutionManager {
    fn default() -> Self {
        Self::new(SubstitutionMap { libraries: vec![] })
    }
}

/// Default substitutions for common libraries
pub fn default_substitutions() -> SubstitutionMap {
    SubstitutionMap {
        libraries: vec![
            LibrarySubstitution {
                name: "com.mojang:authlib".to_string(),
                target: "com.mojang:authlib:1.0.0".to_string(),
                rules: vec![],
            },
            LibrarySubstitution {
                name: "org.lwjgl:lwjgl".to_string(),
                target: "org.lwjgl:lwjgl:3.3.1".to_string(),
                rules: vec![],
            },
            LibrarySubstitution {
                name: "org.lwjgl:lwjgl-glfw".to_string(),
                target: "org.lwjgl:lwjgl-glfw:3.3.1".to_string(),
                rules: vec![],
            },
            LibrarySubstitution {
                name: "org.lwjgl:lwjgl-opengl".to_string(),
                target: "org.lwjgl:lwjgl-opengl:3.3.1".to_string(),
                rules: vec![],
            },
            LibrarySubstitution {
                name: "org.lwjgl:lwjgl-openal".to_string(),
                target: "org.lwjgl:lwjgl-openal:3.3.1".to_string(),
                rules: vec![],
            },
            LibrarySubstitution {
                name: "org.lwjgl:lwjgl-stb".to_string(),
                target: "org.lwjgl:lwjgl-stb:3.3.1".to_string(),
                rules: vec![],
            },
            LibrarySubstitution {
                name: "org.lwjgl:lwjgl-tinyfd".to_string(),
                target: "org.lwjgl:lwjgl-tinyfd:3.3.1".to_string(),
                rules: vec![],
            },
            LibrarySubstitution {
                name: "org.lwjgl:lwjgl-jemalloc".to_string(),
                target: "org.lwjgl:lwjgl-jemalloc:3.3.1".to_string(),
                rules: vec![],
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitution_manager() {
        let subs = default_substitutions();
        let manager = SubstitutionManager::new(subs);
        assert_eq!(manager.apply("com.mojang:authlib"), "com.mojang:authlib:1.0.0");
        assert_eq!(manager.apply("non.existent:lib"), "non.existent:lib");
    }
}