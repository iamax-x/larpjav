//! JRE detector

use crate::jre::JreInstallation;
use crate::error::LarpResult;
use std::path::PathBuf;

/// Detect JRE installations
pub async fn detect_jres() -> LarpResult<Vec<JreInstallation>> {
    let mut jres = Vec::new();
    let search_paths = get_search_paths();

    for path in search_paths {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(jre) = check_jre(&entry.path()).await {
                    jres.push(jre);
                }
            }
        }
    }

    Ok(jres)
}

fn get_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // JAVA_HOME
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        paths.push(PathBuf::from(java_home));
    }

    // Standard locations
    #[cfg(target_os = "windows")]
    {
        paths.push(PathBuf::from("C:\\Program Files\\Java"));
        paths.push(PathBuf::from("C:\\Program Files (x86)\\Java"));
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            paths.push(PathBuf::from(program_files).join("Java"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        paths.push(PathBuf::from("/usr/lib/jvm"));
        paths.push(PathBuf::from("/usr/java"));
        paths.push(PathBuf::from("/opt/java"));
    }

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Library/Java/JavaVirtualMachines"));
        paths.push(PathBuf::from("/System/Library/Java/JavaVirtualMachines"));
    }

    paths
}

async fn check_jre(path: &std::path::Path) -> Option<JreInstallation> {
    let java_exe = if cfg!(target_os = "windows") {
        path.join("bin").join("java.exe")
    } else {
        path.join("bin").join("java")
    };

    if !java_exe.exists() {
        return None;
    }

    let output = tokio::process::Command::new(&java_exe)
        .arg("-version")
        .output()
        .await
        .ok()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let version = parse_version(&stderr)?;
    let vendor = detect_vendor(&stderr);
    let architecture = detect_arch(&stderr);

    Some(JreInstallation {
        version,
        path: path.to_path_buf(),
        java_exe,
        vendor,
        architecture,
    })
}

fn parse_version(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("version") {
            let parts: Vec<&str> = line.split('"').collect();
            if parts.len() >= 2 {
                return Some(parts[1].to_string());
            }
        }
    }
    None
}

fn detect_vendor(output: &str) -> String {
    if output.contains("OpenJDK") { "OpenJDK".to_string() }
    else if output.contains("Oracle") { "Oracle".to_string() }
    else if output.contains("AdoptOpenJDK") || output.contains("Eclipse Adoptium") { "Eclipse Adoptium".to_string() }
    else if output.contains("Azul") { "Azul".to_string() }
    else if output.contains("Amazon Corretto") { "Amazon".to_string() }
    else if output.contains("Microsoft") { "Microsoft".to_string() }
    else { "Unknown".to_string() }
}

fn detect_arch(output: &str) -> String {
    if output.contains("64-Bit") || output.contains("x86_64") || output.contains("aarch64") {
        "64".to_string()
    } else {
        "32".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let output = r#"openjdk version "17.0.5" 2022-10-18
OpenJDK Runtime Environment (build 17.0.5+8)
OpenJDK 64-Bit Server VM (build 17.0.5+8, mixed mode)"#;
        assert_eq!(parse_version(output), Some("17.0.5".to_string()));
    }
}