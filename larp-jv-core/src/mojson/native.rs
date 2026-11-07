//! Native library handling for MoJson

use crate::value::{NativeLibraryExtractable, ExtractSettings, ExtractRules};
use crate::error::LarpResult;
use std::path::Path;
use crate::utils::zip::extract_zip;

/// Native library extractor
pub struct NativeExtractor;

impl NativeExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract native libraries from JAR
    pub async fn extract(
        &self,
        native: &NativeLibraryExtractable,
        jar_path: &Path,
        dest_dir: &Path,
    ) -> LarpResult<Vec<std::path::PathBuf>> {
        let mut extracted = Vec::new();

        if let Some(natives) = &native.natives {
            for (classifier, pattern) in natives {
                // Extract files matching pattern
                let rules = native.extract_rules.as_ref();
                let include = rules.as_ref().and_then(|r| r.exclude.as_ref());

                // Use zip extraction with filtering
                let entries = crate::utils::zip::list_zip_entries(jar_path).await?;
                for entry in entries {
                    if entry.starts_with(&format!("{}/", pattern)) {
                        if let Some(excludes) = include {
                            let skip = excludes.iter().any(|e| entry.contains(e));
                            if skip { continue; }
                        }

                        let dest = dest_dir.join(&entry);
                        crate::utils::zip::extract_zip_entry(jar_path, &entry, &dest).await?;
                        extracted.push(dest);
                    }
                }
            }
        }

        Ok(extracted)
    }

    /// Extract all natives for current OS
    pub async fn extract_for_os(
        &self,
        native: &NativeLibraryExtractable,
        jar_path: &Path,
        dest_dir: &Path,
        os: &str,
    ) -> LarpResult<Vec<std::path::PathBuf>> {
        if let Some(classifier) = native.natives.as_ref().and_then(|n| n.get(os)) {
            let mut modified = native.clone();
            modified.natives = Some(std::collections::HashMap::from([(os.to_string(), classifier.clone())]));
            self.extract(&modified, jar_path, dest_dir).await
        } else {
            Ok(vec![])
        }
    }
}

impl Default for NativeExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract settings helper
pub fn should_extract_file(entry: &str, rules: &ExtractSettings) -> bool {
    if let Some(excludes) = &rules.exclude {
        !excludes.iter().any(|e| entry.contains(e))
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_settings() {
        let settings = ExtractSettings { exclude: vec!["META-INF".to_string()] };
        assert!(!should_extract_file("META-INF/MANIFEST.MF", &settings));
        assert!(should_extract_file("com/example/Class.class", &settings));
    }
}