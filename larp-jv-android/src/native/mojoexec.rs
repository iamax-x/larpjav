//! mojoexec — Pure Rust JVM launcher (replaces jre_launcher C code)

use std::process::Command;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use larp_jv_core::LarpResult;

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
    /// Launch JVM with given configuration
    pub fn launch(config: JvmLaunchConfig) -> LarpResult<std::process::Child> {
        let mut cmd = Command::new(&config.java_exe);
        
        // JVM args
        cmd.args(&config.jvm_args);
        
        // Classpath
        if !config.classpath.is_empty() {
            let cp = config.classpath
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(if cfg!(target_os = "windows") { ";" } else { ":" });
            cmd.arg("-cp").arg(cp);
        }
        
        // Main class
        cmd.arg(&config.main_class);
        
        // Program args
        cmd.args(&config.program_args);
        
        // Working directory
        if let Some(dir) = config.working_dir {
            cmd.current_dir(dir);
        }
        
        // Environment variables
        for (k, v) in config.env_vars {
            cmd.env(k, v);
        }
        
        // Native library path
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
        
        // Set java.library.path JVM arg
        if !config.native_library_path.is_empty() {
            let lib_path = config.native_library_path
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(if cfg!(target_os = "windows") { ";" } else { ":" });
            cmd.arg(format!("-Djava.library.path={}", lib_path));
        }
        
        // Spawn process
        let child = cmd.spawn()
            .map_err(|e| larp_jv_core::LarpError::Jre(larp_jv_core::error::JreError::LaunchFailed(e.to_string())))?;
        
        Ok(child)
    }

    /// Launch and wait for completion
    pub fn launch_and_wait(config: JvmLaunchConfig) -> LarpResult<std::process::ExitStatus> {
        let mut child = Self::launch(config)?;
        let status = child.wait()
            .map_err(|e| larp_jv_core::LarpError::Jre(larp_jv_core::error::JreError::LaunchFailed(e.to_string())))?;
        Ok(status)
    }

    /// Launch with output capture
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
            .map_err(|e| larp_jv_core::LarpError::Jre(larp_jv_core::error::JreError::LaunchFailed(e.to_string())))?;
        
        let output = child.wait_with_output()
            .map_err(|e| larp_jv_core::LarpError::Jre(larp_jv_core::error::JreError::LaunchFailed(e.to_string())))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        Ok((output.status, stdout, stderr))
    }
}

/// ELF parsing utilities (replaces elf_hinter.c)
pub mod elf {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    use std::path::Path;

    /// ELF header
    #[derive(Debug, Clone)]
    pub struct ElfHeader {
        pub class: u8,        // 32-bit or 64-bit
        pub data: u8,         // Endianness
        pub version: u8,
        pub os_abi: u8,
        pub abi_version: u8,
        pub e_type: u16,
        pub machine: u16,
        pub version2: u32,
        pub entry: u64,
        pub phoff: u64,
        pub shoff: u64,
        pub flags: u32,
        pub ehsize: u16,
        pub phentsize: u16,
        pub phnum: u16,
        pub shentsize: u16,
        pub shnum: u16,
        pub shstrndx: u16,
    }

    /// Parse ELF header
    pub fn parse_elf_header(path: &Path) -> std::io::Result<ElfHeader> {
        let mut file = File::open(path)?;
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        
        if magic != [0x7f, b'E', b'L', b'F'] {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Not an ELF file"));
        }
        
        let class = file.read_u8()?;
        let data = file.read_u8()?;
        let version = file.read_u8()?;
        let os_abi = file.read_u8()?;
        let abi_version = file.read_u8()?;
        let _pad = [0u8; 7]; // padding
        file.read_exact(&mut [0u8; 7])?;
        
        let e_type = if class == 1 { file.read_u16::<byteorder::LittleEndian>()? } else { file.read_u16::<byteorder::LittleEndian>()? };
        let machine = file.read_u16::<byteorder::LittleEndian>()?;
        let version2 = file.read_u32::<byteorder::LittleEndian>()?;
        
        let (entry, phoff, shoff) = if class == 1 {
            (file.read_u32::<byteorder::LittleEndian>()? as u64,
             file.read_u32::<byteorder::LittleEndian>()? as u64,
             file.read_u32::<byteorder::LittleEndian>()? as u64)
        } else {
            (file.read_u64::<byteorder::LittleEndian>()?,
             file.read_u64::<byteorder::LittleEndian>()?,
             file.read_u64::<byteorder::LittleEndian>()?)
        };
        
        let flags = file.read_u32::<byteorder::LittleEndian>()?;
        let ehsize = file.read_u16::<byteorder::LittleEndian>()?;
        let phentsize = file.read_u16::<byteorder::LittleEndian>()?;
        let phnum = file.read_u16::<byteorder::LittleEndian>()?;
        let shentsize = file.read_u16::<byteorder::LittleEndian>()?;
        let shnum = file.read_u16::<byteorder::LittleEndian>()?;
        let shstrndx = file.read_u16::<byteorder::LittleEndian>()?;
        
        Ok(ElfHeader {
            class, data, version, os_abi, abi_version,
            e_type, machine, version2, entry, phoff, shoff,
            flags, ehsize, phentsize, phnum, shentsize, shnum, shstrndx,
        })
    }

    /// Check if ELF is 64-bit
    pub fn is_64bit(header: &ElfHeader) -> bool {
        header.class == 2
    }

    /// Check if ELF is 32-bit
    pub fn is_32bit(header: &ElfHeader) -> bool {
        header.class == 1
    }
}

/// Native library hook utilities (replaces native_library_hook.c)
pub mod hooks {
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::Mutex;

    /// Native library hook manager
    pub struct NativeLibraryHook {
        hooks: Mutex<HashMap<String, HookFunc>>,
    }

    type HookFunc = Box<dyn Fn(&str) -> Option<String> + Send + Sync>;

    impl NativeLibraryHook {
        pub fn new() -> Self {
            Self {
                hooks: Mutex::new(HashMap::new()),
            }
        }

        /// Register a hook for library loading
        pub fn register_hook<F>(&self, library_name: &str, f: F)
        where
            F: Fn(&str) -> Option<String> + Send + Sync + 'static,
        {
            self.hooks.lock().unwrap().insert(library_name.to_string(), Box::new(f));
        }

        /// Resolve library path using hooks
        pub fn resolve_library(&self, name: &str) -> Option<String> {
            let hooks = self.hooks.lock().unwrap();
            if let Some(hook) = hooks.get(name) {
                return hook(name);
            }
            None
        }

        /// Default library path resolution
        pub fn default_library_path(name: &str, search_paths: &[&str]) -> Option<String> {
            for path in search_paths {
                let full_path = format!("{}/lib{}.so", path, name);
                if std::path::Path::new(&full_path).exists() {
                    return Some(full_path);
                }
            }
            None
        }
    }

    impl Default for NativeLibraryHook {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// Abort wait utilities (replaces abort_wait.c)
pub mod abort_wait {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    /// Abort waiter for graceful shutdown
    pub struct AbortWaiter {
        aborted: Arc<AtomicBool>,
    }

    impl AbortWaiter {
        pub fn new() -> Self {
            Self {
                aborted: Arc::new(AtomicBool::new(false)),
            }
        }

        pub fn abort(&self) {
            self.aborted.store(true, Ordering::SeqCst);
        }

        pub fn is_aborted(&self) -> bool {
            self.aborted.load(Ordering::SeqCst)
        }

        pub fn wait(&self, timeout: Duration) -> bool {
            let start = std::time::Instant::now();
            while !self.is_aborted() {
                if start.elapsed() >= timeout {
                    return false;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            true
        }

        pub fn get_token(&self) -> AbortToken {
            AbortToken(self.aborted.clone())
        }
    }

    #[derive(Clone)]
    pub struct AbortToken(Arc<AtomicBool>);

    impl AbortToken {
        pub fn abort(&self) {
            self.0.store(true, Ordering::SeqCst);
        }

        pub fn is_aborted(&self) -> bool {
            self.0.load(Ordering::SeqCst)
        }
    }
}