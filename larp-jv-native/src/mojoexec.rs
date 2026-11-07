//! mojoexec — Pure Rust JVM launcher (replaces jre_launcher C code)

use std::process::Command;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::error::LarpResult;

/// JVM launcher configuration
#[derive(Debug, Clone)]
pub struct JvmLaunchConfig {
    pub java_exe: PathBuf,
    pub jvm_args: Vec<String>,
    pub classpath: Vec<PathBuf>,
    pub main_class: String,
    pub program_args: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub env_vars: HashMap<String, String>,
    pub native_library_path: Vec<PathBuf>,
}

impl JvmLaunchConfig {
    pub fn new(java_exe: PathBuf, main_class: String) -> Self {
        Self {
            java_exe,
            jvm_args: Vec::new(),
            classpath: Vec::new(),
            main_class,
            program_args: Vec::new(),
            working_dir: None,
            env_vars: HashMap::new(),
            native_library_path: Vec::new(),
        }
    }

    pub fn add_jvm_arg(mut self, arg: String) -> Self {
        self.jvm_args.push(arg);
        self
    }

    pub fn add_classpath(mut self, path: PathBuf) -> Self {
        self.classpath.push(path);
        self
    }

    pub fn add_program_arg(mut self, arg: String) -> Self {
        self.program_args.push(arg);
        self
    }

    pub fn set_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    pub fn set_env_var(mut self, key: String, value: String) -> Self {
        self.env_vars.insert(key, value);
        self
    }

    pub fn add_native_library_path(mut self, path: PathBuf) -> Self {
        self.native_library_path.push(path);
        self
    }
}

/// JVM launcher
pub struct JvmLauncher;

impl JvmLauncher {
    pub fn launch(config: JvmLaunchConfig) -> LarpResult<std::process::Child> {
        let mut cmd = Command::new(&config.java_exe);
        
        cmd.args(&config.jvm_args);
        
        if !config.classpath.is_empty() {
            let cp = config.classpath
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(if cfg!(target_os = "windows") { ";" } else { ":" });
            cmd.arg("-cp").arg(cp);
        }
        
        cmd.arg(&config.main_class);
        cmd.args(&config.program_args);
        
        if let Some(dir) = config.working_dir {
            cmd.current_dir(dir);
        }
        
        for (k, v) in config.env_vars {
            cmd.env(k, v);
        }
        
        if !config.native_library_path.is_empty() {
            let lib_path = config.native_library_path
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(if cfg!(target_os = "windows") { ";" } else { ":" });
            
            #[cfg(target_os = "linux")]
            cmd.env("LD_LIBRARY_PATH", lib_path);
            #[cfg(target_os = "macos")]
            cmd.env("DYLD_LIBRARY_PATH", lib_path);
            #[cfg(target_os = "windows")]
            cmd.env("PATH", format!("{};{}", lib_path, std::env::var("PATH").unwrap_or_default()));
            #[cfg(target_os = "android")]
            cmd.env("LD_LIBRARY_PATH", lib_path);
        }
        
        if !config.native_library_path.is_empty() {
            let lib_path = config.native_library_path
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(if cfg!(target_os = "windows") { ";" } else { ":" });
            cmd.arg(format!("-Djava.library.path={}", lib_path));
        }
        
        let child = cmd.spawn()
            .map_err(|e| crate::error::LarpError::Jre(crate::error::JreError::LaunchFailed(e.to_string())))?;
        
        Ok(child)
    }

    pub fn launch_and_wait(config: JvmLaunchConfig) -> LarpResult<std::process::ExitStatus> {
        let mut child = Self::launch(config)?;
        let status = child.wait()
            .map_err(|e| crate::error::LarpError::Jre(crate::error::JreError::LaunchFailed(e.to_string())))?;
        Ok(status)
    }

    pub fn launch_capture(config: JvmLaunchConfig) -> LarpResult<(std::process::ExitStatus, String, String)> {
        use std::process::Stdio;
        
        let mut cmd = Command::new(&config.java_exe);
        
        cmd.args(&config.jvm_args);
        
        if !config.classpath.is_empty() {
            let cp = config.classpath
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(if cfg!(target_os = "windows") { ";" } else { ":" });
            cmd.arg("-cp").arg(cp);
        }
        
        cmd.arg(&config.main_class);
        cmd.args(&config.program_args);
        
        if let Some(dir) = config.working_dir {
            cmd.current_dir(dir);
        }
        
        for (k, v) in config.env_vars {
            cmd.env(k, v);
        }
        
        if !config.native_library_path.is_empty() {
            let lib_path = config.native_library_path
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(if cfg!(target_os = "windows") { ";" } else { ":" });
            
            #[cfg(target_os = "linux")]
            cmd.env("LD_LIBRARY_PATH", lib_path);
            #[cfg(target_os = "macos")]
            cmd.env("DYLD_LIBRARY_PATH", lib_path);
            #[cfg(target_os = "windows")]
            cmd.env("PATH", format!("{};{}", lib_path, std::env::var("PATH").unwrap_or_default()));
            #[cfg(target_os = "android")]
            cmd.env("LD_LIBRARY_PATH", lib_path);
        }
        
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        let mut child = cmd.spawn()
            .map_err(|e| crate::error::LarpError::Jre(crate::error::JreError::LaunchFailed(e.to_string())))?;
        
        let output = child.wait_with_output()
            .map_err(|e| crate::error::LarpError::Jre(crate::error::JreError::LaunchFailed(e.to_string())))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        Ok((output.status, stdout, stderr))
    }
}

pub fn init() {
    log::info!("mojoexec initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = JvmLaunchConfig::new(
            PathBuf::from("java"),
            "com.example.Main".to_string()
        )
        .add_jvm_arg("-Xmx2G".to_string())
        .add_classpath(PathBuf::from("app.jar"));
        
        assert_eq!(config.jvm_args.len(), 1);
        assert_eq!(config.classpath.len(), 1);
    }
}