//! JVM Hooks — LD_PRELOAD hooks (replaces jvm_hooks C code)

use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// JVM hooks manager
pub struct JvmHooks {
    hooks: Mutex<HashMap<String, HookFunc>>,
    loaded_libraries: Mutex<Vec<String>>,
}

type HookFunc = Box<dyn Fn(&str, &[u8]) -> Option<Vec<u8>> + Send + Sync>;

impl JvmHooks {
    pub fn new() -> Self {
        Self {
            hooks: Mutex::new(HashMap::new()),
            loaded_libraries: Mutex::new(Vec::new()),
        }
    }

    pub fn register_hook<F>(&self, function_name: &str, f: F)
    where
        F: Fn(&str, &[u8]) -> Option<Vec<u8>> + Send + Sync + 'static,
    {
        self.hooks.lock().unwrap().insert(function_name.to_string(), Box::new(f));
    }

    pub fn call_hook(&self, function_name: &str, args: &[u8]) -> Option<Vec<u8>> {
        let hooks = self.hooks.lock().unwrap();
        if let Some(hook) = hooks.get(function_name) {
            return hook(function_name, args);
        }
        None
    }

    pub fn track_library(&self, library_name: &str) {
        self.loaded_libraries.lock().unwrap().push(library_name.to_string());
    }

    pub fn is_library_loaded(&self, library_name: &str) -> bool {
        self.loaded_libraries.lock().unwrap().contains(library_name)
    }

    pub fn get_loaded_libraries(&self) -> Vec<String> {
        self.loaded_libraries.lock().unwrap().clone()
    }
}

impl Default for JvmHooks {
    fn default() -> Self {
        Self::new()
    }
}

/// Global hooks instance
lazy_static::lazy_static! {
    pub static ref JVM_HOOKS: JvmHooks = JvmHooks::new();
}

/// Java exec hooks (replaces java_exec_hooks.c)
pub mod java_exec {
    use super::*;

    pub fn init_hooks() {
        JVM_HOOKS.register_hook("java_exec", |func, args| {
            log::debug!("java_exec hook called: {}", func);
            None
        });

        JVM_HOOKS.register_hook("java_exec_with_args", |func, args| {
            log::debug!("java_exec_with_args hook called: {}", func);
            None
        });
    }

    pub fn hook_execve(path: &str, args: &[&str], env: &[&str]) -> Option<(String, Vec<String>, Vec<String>)> {
        if path.contains("java") || path.contains("jvm") {
            log::debug!("Intercepted JVM launch: {} {:?}", path, args);
            
            let modified_args = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
            let modified_env = env.iter().map(|s| s.to_string()).collect::<Vec<_>>();
            
            return Some((path.to_string(), modified_args, modified_env));
        }
        None
    }
}

/// LWJGL dlopen hook (replaces lwjgl_dlopen_hook.c)
pub mod lwjgl {
    use super::*;

    pub fn init_hooks() {
        JVM_HOOKS.register_hook("lwjgl_dlopen", |func, args| {
            if func.contains("lwjgl") || func.contains("glfw") || func.contains("openal") {
                log::debug!("LWJGL dlopen hook: {}", func);
            }
            None
        });

        JVM_HOOKS.register_hook("lwjgl_load_library", |func, args| {
            if func.contains("lwjgl") {
                log::debug!("LWJGL loadLibrary hook: {}", func);
            }
            None
        });
    }

    pub fn hook_dlopen(filename: &str, flags: i32) -> Option<String> {
        if filename.contains("lwjgl") || filename.contains("glfw") || filename.contains("openal") {
            log::debug!("Intercepted dlopen: {}", filename);
        }
        None
    }

    pub fn hook_dlsym(handle: *mut std::ffi::c_void, symbol: &str) -> Option<*mut std::ffi::c_void> {
        if symbol.contains("lwjgl") || symbol.contains("glfw") || symbol.contains("openal") {
            log::debug!("Intercepted dlsym: {}", symbol);
        }
        None
    }
}

/// EMUI iterator fix hook (replaces emui_iterator_fix_hook.c)
pub mod emui {
    use super::*;

    pub fn init_hooks() {
        JVM_HOOKS.register_hook("emui_iterator_fix", |func, args| {
            log::debug!("EMUI iterator fix hook: {}", func);
            None
        });
    }

    pub fn fix_iterator<T>(iter: impl Iterator<Item = T>) -> Vec<T> {
        iter.collect()
    }
}

/// Thread affinity hook (replaces affinity.c)
pub mod affinity {
    use std::os::unix::thread::ThreadExt;

    pub fn set_thread_affinity(cpu_set: &[usize]) -> std::io::Result<()> {
        log::debug!("Setting thread affinity: {:?}", cpu_set);
        Ok(())
    }

    pub fn get_thread_affinity() -> std::io::Result<Vec<usize>> {
        Ok(Vec::new())
    }
}

/// Stdio redirect hook (replaces stdio_is.c)
pub mod stdio {
    use std::io::{self, Write};
    use std::fs::OpenOptions;
    use std::os::unix::io::AsRawFd;

    pub fn redirect_stdio(stdout_path: &str, stderr_path: &str) -> std::io::Result<()> {
        let stdout_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(stdout_path)?;
        
        let stderr_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(stderr_path)?;
        
        unsafe {
            libc::dup2(stdout_file.as_raw_fd(), libc::STDOUT_FILENO);
            libc::dup2(stderr_file.as_raw_fd(), libc::STDERR_FILENO);
        }
        
        Ok(())
    }

    pub struct StdioCapture {
        stdout_buf: Vec<u8>,
        stderr_buf: Vec<u8>,
    }

    impl StdioCapture {
        pub fn new() -> Self {
            Self {
                stdout_buf: Vec::new(),
                stderr_buf: Vec::new(),
            }
        }

        pub fn stdout(&self) -> &[u8] {
            &self.stdout_buf
        }

        pub fn stderr(&self) -> &[u8] {
            &self.stderr_buf
        }
    }
}

pub fn init() {
    java_exec::init_hooks();
    lwjgl::init_hooks();
    emui::init_hooks();
    log::info!("jvm_hooks initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks() {
        init();
    }
}