//! Android MultiRT — port of MultiRTConfigDialog.java + MultiRTUtils.java

use larp_jv_core::LarpResult;
use std::path::Path;
use std::collections::HashMap;

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub name: String,
    pub version: String,
    pub path: std::path::PathBuf,
    pub jvm_args: Vec<String>,
    pub is_default: bool,
}

/// MultiRT manager
pub struct MultiRTManager {
    runtimes: HashMap<String, RuntimeConfig>,
    default_runtime: Option<String>,
    config_dir: std::path::PathBuf,
}

impl MultiRTManager {
    pub fn new(config_dir: impl AsRef<Path>) -> LarpResult<Self> {
        let config_dir = config_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&config_dir)?;
        
        let mut manager = Self {
            runtimes: HashMap::new(),
            default_runtime: None,
            config_dir,
        };
        
        manager.load()?;
        Ok(manager)
    }

    fn load(&mut self) -> LarpResult<()> {
        // Load from config file
        let config_file = self.config_dir.join("multirt.json");
        if config_file.exists() {
            let content = std::fs::read_to_string(&config_file)?;
            let data: serde_json::Value = serde_json::from_str(&content)?;
            
            if let Some(runtimes) = data.get("runtimes").and_then(|v| v.as_array()) {
                for rt in runtimes {
                    if let (Some(name), Some(version), Some(path)) = (
                        rt.get("name").and_then(|v| v.as_str()),
                        rt.get("version").and_then(|v| v.as_str()),
                        rt.get("path").and_then(|v| v.as_str()),
                    ) {
                        let config = RuntimeConfig {
                            name: name.to_string(),
                            version: version.to_string(),
                            path: std::path::PathBuf::from(path),
                            jvm_args: rt.get("jvm_args")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                                .unwrap_or_default(),
                            is_default: rt.get("is_default").and_then(|v| v.as_bool()).unwrap_or(false),
                        };
                        
                        if config.is_default {
                            self.default_runtime = Some(name.to_string());
                        }
                        
                        self.runtimes.insert(name.to_string(), config);
                    }
                }
            }
        }
        
        Ok(())
    }

    fn save(&self) -> LarpResult<()> {
        let config_file = self.config_dir.join("multirt.json");
        let mut data = serde_json::Map::new();
        
        let mut runtimes = Vec::new();
        for (name, config) in &self.runtimes {
            let mut rt = serde_json::Map::new();
            rt.insert("name".to_string(), serde_json::Value::String(name.clone()));
            rt.insert("version".to_string(), serde_json::Value::String(config.version.clone()));
            rt.insert("path".to_string(), serde_json::Value::String(config.path.to_string_lossy().to_string()));
            rt.insert("jvm_args".to_string(), serde_json::Value::Array(
                config.jvm_args.iter().map(|s| serde_json::Value::String(s.clone())).collect()
            ));
            rt.insert("is_default".to_string(), serde_json::Value::Bool(config.is_default));
            runtimes.push(serde_json::Value::Object(rt));
        }
        
        data.insert("runtimes".to_string(), serde_json::Value::Array(runtimes));
        
        let content = serde_json::to_string_pretty(&serde_json::Value::Object(data))?;
        std::fs::write(config_file, content)?;
        Ok(())
    }

    /// Add a runtime
    pub fn add_runtime(&mut self, config: RuntimeConfig) -> LarpResult<()> {
        if config.is_default {
            // Unset previous default
            for rt in self.runtimes.values_mut() {
                rt.is_default = false;
            }
            self.default_runtime = Some(config.name.clone());
        }
        
        self.runtimes.insert(config.name.clone(), config);
        self.save()?;
        Ok(())
    }

    /// Remove a runtime
    pub fn remove_runtime(&mut self, name: &str) -> LarpResult<bool> {
        let removed = self.runtimes.remove(name).is_some();
        if removed {
            if self.default_runtime.as_deref() == Some(name) {
                self.default_runtime = None;
                // Set first runtime as default if any
                if let Some(first) = self.runtimes.keys().next().cloned() {
                    if let Some(rt) = self.runtimes.get_mut(&first) {
                        rt.is_default = true;
                        self.default_runtime = Some(first);
                    }
                }
            }
            self.save()?;
        }
        Ok(removed)
    }

    /// Get runtime by name
    pub fn get_runtime(&self, name: &str) -> Option<&RuntimeConfig> {
        self.runtimes.get(name)
    }

    /// Get default runtime
    pub fn get_default_runtime(&self) -> Option<&RuntimeConfig> {
        self.default_runtime.as_ref().and_then(|name| self.runtimes.get(name))
    }

    /// List all runtimes
    pub fn list_runtimes(&self) -> Vec<&RuntimeConfig> {
        self.runtimes.values().collect()
    }

    /// Set default runtime
    pub fn set_default_runtime(&mut self, name: &str) -> LarpResult<bool> {
        if self.runtimes.contains_key(name) {
            // Unset previous default
            for rt in self.runtimes.values_mut() {
                rt.is_default = false;
            }
            
            if let Some(rt) = self.runtimes.get_mut(name) {
                rt.is_default = true;
                self.default_runtime = Some(name.to_string());
                self.save()?;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// Detect installed Java runtimes
    pub fn detect_runtimes(&mut self) -> LarpResult<Vec<RuntimeConfig>> {
        let mut detected = Vec::new();
        
        // Check standard locations
        let search_paths = [
            "/usr/lib/jvm",
            "/usr/java",
            "/opt/java",
            "/Library/Java/JavaVirtualMachines",
            "C:\\Program Files\\Java",
            "C:\\Program Files (x86)\\Java",
        ];
        
        for path in &search_paths {
            let path = std::path::Path::new(path);
            if path.exists() {
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let entry_path = entry.path();
                        if entry_path.is_dir() {
                            // Check if it's a Java runtime
                            let java_exe = if cfg!(target_os = "windows") {
                                entry_path.join("bin").join("java.exe")
                            } else {
                                entry_path.join("bin").join("java")
                            };
                            
                            if java_exe.exists() {
                                // Try to get version
                                if let Ok(output) = std::process::Command::new(&java_exe).arg("-version").output() {
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    let version = Self::parse_java_version(&stderr).unwrap_or_else(|| "Unknown".to_string());
                                    
                                    let config = RuntimeConfig {
                                        name: entry_path.file_name().unwrap().to_string_lossy().to_string(),
                                        version,
                                        path: entry_path,
                                        jvm_args: Vec::new(),
                                        is_default: false,
                                    };
                                    
                                    detected.push(config);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(detected)
    }

    fn parse_java_version(output: &str) -> Option<String> {
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
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: String::new(),
            path: std::path::PathBuf::new(),
            jvm_args: Vec::new(),
            is_default: false,
        }
    }
}