//! JAR signature verification

use crate::error::LarpResult;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

/// Verify JAR signature (basic check for META-INF/*.SF, *.DSA, *.RSA)
pub fn verify_jar_signature(path: &Path) -> LarpResult<bool> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut has_sf = false;
    let mut has_dsa_or_rsa = false;

    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        let name = entry.name();
        if name.starts_with("META-INF/") && name.ends_with(".SF") {
            has_sf = true;
        }
        if name.starts_with("META-INF/") && (name.ends_with(".DSA") || name.ends_with(".RSA") || name.ends_with(".EC")) {
            has_dsa_or_rsa = true;
        }
    }

    Ok(has_sf && has_dsa_or_rsa)
}

/// Get signature files from JAR
pub fn get_signature_files(path: &Path) -> LarpResult<Vec<String>> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut files = Vec::new();

    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        let name = entry.name();
        if name.starts_with("META-INF/") &&
           (name.ends_with(".SF") || name.ends_with(".DSA") || name.ends_with(".RSA") || name.ends_with(".EC") || name.ends_with(".MF")) {
            files.push(name.to_string());
        }
    }

    Ok(files)
}

/// Read MANIFEST.MF
pub fn read_manifest(path: &Path) -> LarpResult<String> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut entry = archive.by_name("META-INF/MANIFEST.MF")?;
    let mut content = String::new();
    entry.read_to_string(&mut content)?;
    Ok(content)
}

/// Parse MANIFEST.MF
pub fn parse_manifest(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut current_key = String::new();
    let mut current_value = String::new();

    for line in content.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            // Continuation line
            current_value.push_str(line.trim_start());
        } else if let Some((key, value)) = line.split_once(':') {
            if !current_key.is_empty() {
                map.insert(current_key, current_value);
            }
            current_key = key.trim().to_string();
            current_value = value.trim().to_string();
        }
    }
    if !current_key.is_empty() {
        map.insert(current_key, current_value);
    }
    map
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use zip::write::FileOptions;
    use zip::ZipWriter;

    #[test]
    fn test_parse_manifest() {
        let manifest = r#"Manifest-Version: 1.0
Created-By: 1.8.0_292 (Oracle Corporation)
Main-Class: com.example.Main
"#;
        let parsed = parse_manifest(manifest);
        assert_eq!(parsed.get("Manifest-Version"), Some(&"1.0".to_string()));
        assert_eq!(parsed.get("Main-Class"), Some(&"com.example.Main".to_string()));
    }
}
