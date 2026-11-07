//! JRE management for larp.jv

pub mod runner;
pub mod detector;
pub mod downloader;
pub mod classfile;
pub mod error;

use crate::error::{JreError, LarpError, LarpResult};
use crate::utils::download::{create_client, download_file};
use reqwest::Client;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// JRE manager - handles JRE detection, download, and management
pub struct JreManager {
    client: Client,
    cache_dir: PathBuf,
    installed_jres: HashMap<String, JreInstallation>,
}

impl JreManager {
    pub fn new(cache_dir: impl AsRef<Path>) -> LarpResult<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            client: create_client()?,
            cache_dir,
            installed_jres: HashMap::new(),
        })
    }

    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    /// Detect installed JREs
    pub async fn detect_jres(&mut self) -> LarpResult<Vec<JreInstallation>> {
        let mut jres = Vec::new();

        // Check standard locations
        let search_paths = Self::get_search_paths();
        for path in search_paths {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if let Some(jre) = Self::check_jre(&entry.path()).await {
                        jres.push(jre);
                    }
                }
            }
        }

        // Update cache
        for jre in &jres {
            self.installed_jres.insert(jre.version.clone(), jre.clone());
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

    async fn check_jre(path: &Path) -> Option<JreInstallation> {
        let java_exe = if cfg!(target_os = "windows") {
            path.join("bin").join("java.exe")
        } else {
            path.join("bin").join("java")
        };

        if !java_exe.exists() {
            return None;
        }

        // Get version
        let output = tokio::process::Command::new(&java_exe)
            .arg("-version")
            .output()
            .await
            .ok()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let version = Self::parse_version(&stderr)?;

        Some(JreInstallation {
            version,
            path: path.to_path_buf(),
            java_exe,
            vendor: Self::detect_vendor(&stderr),
            architecture: Self::detect_arch(&stderr),
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

    /// Download JRE for version
    pub async fn download_jre(
        &self,
        version: &str,
        dest_dir: &Path,
    ) -> LarpResult<JreInstallation> {
        // Use Adoptium API for JDK downloads
        let url = format!(
            "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/jdk/hotspot/normal/eclipse",
            version,
            Self::current_arch()
        );

        let dest = dest_dir.join(format!("jdk-{}-{}", version, Self::current_arch()));
        std::fs::create_dir_all(&dest)?;

        download_file(&self.client, &url, &dest, None, false).await?;

        // Extract if needed (JDKs are often tar.gz)
        if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
            crate::utils::zip::extract_zip(&dest, &dest.parent().unwrap())?;
        }

        // Verify installation
        let java_exe = if cfg!(target_os = "windows") {
            dest.join("bin").join("java.exe")
        } else {
            dest.join("bin").join("java")
        };

        Ok(JreInstallation {
            version: version.to_string(),
            path: dest,
            java_exe,
            vendor: "Eclipse Adoptium".to_string(),
            architecture: Self::current_arch().to_string(),
        })
    }

    fn current_arch() -> &'static str {
        #[cfg(target_arch = "x86_64")] return "x64";
        #[cfg(target_arch = "aarch64")] return "aarch64";
        #[cfg(target_arch = "x86")] return "x86";
        "x64"
    }

    /// Get or download JRE for version
    pub async fn get_jre(&mut self, version: &str) -> LarpResult<JreInstallation> {
        if let Some(jre) = self.installed_jres.get(version) {
            return Ok(jre.clone());
        }

        // Try to detect
        let detected = self.detect_jres().await?;
        if let Some(jre) = detected.into_iter().find(|j| j.version.starts_with(version)) {
            return Ok(jre);
        }

        // Download
        self.download_jre(version, &self.cache_dir.join("jres")).await
    }
}

/// JRE installation info
#[derive(Debug, Clone)]
pub struct JreInstallation {
    pub version: String,
    pub path: PathBuf,
    pub java_exe: PathBuf,
    pub vendor: String,
    pub architecture: String,
}

impl JreInstallation {
    /// Get java executable path
    pub fn java_path(&self) -> &Path {
        &self.java_exe
    }

    /// Get JRE home path
    pub fn home(&self) -> &Path {
        &self.path
    }

    /// Check if version matches
    pub fn matches(&self, required: &str) -> bool {
        // Simple version matching - would need proper semver
        self.version.starts_with(required)
    }
}

/// Java runner - builds and executes java command
pub struct JavaRunner {
    jre: JreInstallation,
    jvm_args: Vec<String>,
    classpath: Vec<PathBuf>,
    main_class: String,
    program_args: Vec<String>,
    env: HashMap<String, String>,
}

impl JavaRunner {
    pub fn new(jre: JreInstallation) -> Self {
        Self {
            jre,
            jvm_args: Vec::new(),
            classpath: Vec::new(),
            main_class: String::new(),
            program_args: Vec::new(),
            env: HashMap::new(),
        }
    }

    pub fn jvm_args(mut self, args: Vec<String>) -> Self {
        self.jvm_args = args;
        self
    }

    pub fn add_jvm_arg(mut self, arg: String) -> Self {
        self.jvm_args.push(arg);
        self
    }

    pub fn classpath(mut self, cp: Vec<PathBuf>) -> Self {
        self.classpath = cp;
        self
    }

    pub fn add_classpath(mut self, path: PathBuf) -> Self {
        self.classpath.push(path);
        self
    }

    pub fn main_class(mut self, class: String) -> Self {
        self.main_class = class;
        self
    }

    pub fn program_args(mut self, args: Vec<String>) -> Self {
        self.program_args = args;
        self
    }

    pub fn add_program_arg(mut self, arg: String) -> Self {
        self.program_args.push(arg);
        self
    }

    pub fn env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    pub fn set_env(mut self, key: String, value: String) -> Self {
        self.env.insert(key, value);
        self
    }

    /// Build command
    pub fn build_command(&self) -> Vec<String> {
        let mut cmd = Vec::new();
        cmd.push(self.jre.java_exe.to_string_lossy().to_string());
        cmd.extend(self.jvm_args.clone());

        if !self.classpath.is_empty() {
            let cp = self.classpath.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(if cfg!(target_os = "windows") { ";" } else { ":" });
            cmd.push("-cp".to_string());
            cmd.push(cp);
        }

        cmd.push(self.main_class.clone());
        cmd.extend(self.program_args.clone());

        cmd
    }

    /// Run the java process
    pub async fn run(&self) -> LarpResult<std::process::ExitStatus> {
        let mut command = tokio::process::Command::new(&self.jre.java_exe);
        command.args(&self.jvm_args);

        if !self.classpath.is_empty() {
            let cp = self.classpath.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(if cfg!(target_os = "windows") { ";" } else { ":" });
            command.arg("-cp").arg(cp);
        }

        command.arg(&self.main_class);
        command.args(&self.program_args);

        for (k, v) in &self.env {
            command.env(k, v);
        }

        let status = command.status().await?;
        Ok(status)
    }

    /// Run with output capture
    pub async fn run_capture(&self) -> LarpResult<(std::process::ExitStatus, String, String)> {
        let mut command = tokio::process::Command::new(&self.jre.java_exe);
        command.args(&self.jvm_args);

        if !self.classpath.is_empty() {
            let cp = self.classpath.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(if cfg!(target_os = "windows") { ";" } else { ":" });
            command.arg("-cp").arg(cp);
        }

        command.arg(&self.main_class);
        command.args(&self.program_args);

        for (k, v) in &self.env {
            command.env(k, v);
        }

        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        let output = command.output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok((output.status, stdout, stderr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_runner() {
        let jre = JreInstallation {
            version: "17".to_string(),
            path: PathBuf::from("/usr/lib/jvm/java-17"),
            java_exe: PathBuf::from("/usr/lib/jvm/java-17/bin/java"),
            vendor: "OpenJDK".to_string(),
            architecture: "64".to_string(),
        };

        let runner = JavaRunner::new(jre)
            .main_class("com.example.Main".to_string())
            .add_jvm_arg("-Xmx2G".to_string())
            .add_classpath(PathBuf::from("/app/app.jar"));

        let cmd = runner.build_command();
        assert!(cmd[0].ends_with("java"));
        assert!(cmd.contains(&"-Xmx2G".to_string()));
        assert!(cmd.contains(&"-cp".to_string()));
    }
}