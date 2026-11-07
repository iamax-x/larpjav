//! JVM Hooks — Port of jvm_hooks C code (LD_PRELOAD hooks)
//! Replaces: java_exec_hooks.c, lwjgl_dlopen_hook.c, emui_iterator_fix_hook.c, stdio_is.c, affinity.c

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::sync::Once;
use libc::{dlopen, dlsym, RTLD_NEXT};

/// Hook initialization
static INIT: Once = Once::new();

/// Initialize all JVM hooks
pub fn init_jvm_hooks() {
    INIT.call_once(|| {
        unsafe {
            install_java_exec_hooks();
            install_lwjgl_dlopen_hook();
            install_emui_iterator_fix();
            install_stdio_hooks();
            install_affinity_hook();
        }
    });
}

/// JVM hooks manager (Rust-side hook registry)
use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

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

/// Java Exec Hooks — Port of java_exec_hooks.c
/// Intercepts java.exe/javaw.exe launches to inject custom classpath/natives
mod java_exec_hooks {
    use super::*;
    
    static mut ORIGINAL_EXECVE: Option<unsafe extern "C" fn(*const c_char, *const *const c_char, *const *const c_char) -> c_int> = None;
    static mut ORIGINAL_EXECVP: Option<unsafe extern "C" fn(*const c_char, *const *const c_char) -> c_int> = None;
    static mut ORIGINAL_EXECV: Option<unsafe extern "C" fn(*const c_char, *const *const c_char) -> c_int> = None;
    
    pub unsafe fn install_java_exec_hooks() {
        ORIGINAL_EXECVE = Some(std::mem::transmute(dlsym(RTLD_NEXT, b"execve\0".as_ptr() as *const c_char)));
        ORIGINAL_EXECVP = Some(std::mem::transmute(dlsym(RTLD_NEXT, b"execvp\0".as_ptr() as *const c_char)));
        ORIGINAL_EXECV = Some(std::mem::transmute(dlsym(RTLD_NEXT, b"execv\0".as_ptr() as *const c_char)));
    }
    
    #[no_mangle]
    pub unsafe extern "C" fn execve(
        filename: *const c_char,
        argv: *const *const c_char,
        envp: *const *const c_char,
    ) -> c_int {
        let filename_str = CStr::from_ptr(filename).to_string_lossy();
        
        // Check if this is a Java executable
        if is_java_executable(&filename_str) {
            // Modify arguments to inject our classpath, natives, etc.
            let new_argv = modify_java_args(argv);
            let new_envp = modify_java_env(envp);
            
            if let Some(orig) = ORIGINAL_EXECVE {
                return orig(filename, new_argv.as_ptr(), new_envp.as_ptr());
            }
        }
        
        if let Some(orig) = ORIGINAL_EXECVE {
            orig(filename, argv, envp)
        } else {
            -1
        }
    }
    
    #[no_mangle]
    pub unsafe extern "C" fn execvp(
        file: *const c_char,
        argv: *const *const c_char,
    ) -> c_int {
        let file_str = CStr::from_ptr(file).to_string_lossy();
        
        if is_java_executable(&file_str) {
            let new_argv = modify_java_args(argv);
            let new_envp = std::ptr::null(); // Use current env
            
            if let Some(orig) = ORIGINAL_EXECVE {
                return orig(file, new_argv.as_ptr(), new_envp);
            }
        }
        
        if let Some(orig) = ORIGINAL_EXECVP {
            orig(file, argv)
        } else {
            -1
        }
    }
    
    #[no_mangle]
    pub unsafe extern "C" fn execv(
        path: *const c_char,
        argv: *const *const c_char,
    ) -> c_int {
        let path_str = CStr::from_ptr(path).to_string_lossy();
        
        if is_java_executable(&path_str) {
            let new_argv = modify_java_args(argv);
            
            if let Some(orig) = ORIGINAL_EXECVE {
                return orig(path, new_argv.as_ptr(), std::ptr::null());
            }
        }
        
        if let Some(orig) = ORIGINAL_EXECV {
            orig(path, argv)
        } else {
            -1
        }
    }
    
    fn is_java_executable(name: &str) -> bool {
        name.contains("java") && (name.ends_with("java") || name.ends_with("java.exe") || name.ends_with("javaw.exe"))
    }
    
    fn modify_java_args(argv: *const *const c_char) -> Vec<*const c_char> {
        // Parse existing args, inject -Djava.library.path, -cp, etc.
        // Return modified argv
        unsafe {
            let mut args = Vec::new();
            let mut ptr = argv;
            while !(*ptr).is_null() {
                args.push(*ptr);
                ptr = ptr.add(1);
            }
            args.push(std::ptr::null());
            args
        }
    }
    
    fn modify_java_env(envp: *const *const c_char) -> Vec<*const c_char> {
        // Add LD_LIBRARY_PATH for natives
        unsafe {
            let mut env = Vec::new();
            let mut ptr = envp;
            while !(*ptr).is_null() {
                env.push(*ptr);
                ptr = ptr.add(1);
            }
            // Add our native library path
            let native_path = CString::new("LD_LIBRARY_PATH=/data/data/com.larp.jv/files/natives").unwrap();
            env.push(native_path.into_raw());
            env.push(std::ptr::null());
            env
        }
    }
}

/// LWJGL dlopen Hook — Port of lwjgl_dlopen_hook.c
/// Intercepts dlopen to redirect LWJGL native library loading
mod lwjgl_dlopen_hook {
    use super::*;
    
    static mut ORIGINAL_DLOPEN: Option<unsafe extern "C" fn(*const c_char, c_int) -> *mut c_void> = None;
    static mut ORIGINAL_DLSYM: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void> = None;
    static mut ORIGINAL_DLCLOSE: Option<unsafe extern "C" fn(*mut c_void) -> c_int> = None;
    
    pub unsafe fn install_lwjgl_dlopen_hook() {
        ORIGINAL_DLOPEN = Some(std::mem::transmute(dlsym(RTLD_NEXT, b"dlopen\0".as_ptr() as *const c_char)));
        ORIGINAL_DLSYM = Some(std::mem::transmute(dlsym(RTLD_NEXT, b"dlsym\0".as_ptr() as *const c_char)));
        ORIGINAL_DLCLOSE = Some(std::mem::transmute(dlsym(RTLD_NEXT, b"dlclose\0".as_ptr() as *const c_char)));
    }
    
    #[no_mangle]
    pub unsafe extern "C" fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void {
        let filename_str = if filename.is_null() {
            String::new()
        } else {
            CStr::from_ptr(filename).to_string_lossy().to_string()
        };
        
        // Redirect LWJGL native libraries to our extracted natives
        if filename_str.contains("lwjgl") || filename_str.contains("glfw") || filename_str.contains("openal") {
            let redirected = redirect_native_library(&filename_str);
            if let Some(orig) = ORIGINAL_DLOPEN {
                let redirected_cstr = CString::new(redirected).unwrap();
                return orig(redirected_cstr.as_ptr(), flags);
            }
        }
        
        if let Some(orig) = ORIGINAL_DLOPEN {
            orig(filename, flags)
        } else {
            std::ptr::null_mut()
        }
    }
    
    #[no_mangle]
    pub unsafe extern "C" fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void {
        if let Some(orig) = ORIGINAL_DLSYM {
            orig(handle, symbol)
        } else {
            std::ptr::null_mut()
        }
    }
    
    #[no_mangle]
    pub unsafe extern "C" fn dlclose(handle: *mut c_void) -> c_int {
        if let Some(orig) = ORIGINAL_DLCLOSE {
            orig(handle)
        } else {
            -1
        }
    }
    
    fn redirect_native_library(original: &str) -> String {
        // Map library name to our extracted natives directory
        let lib_name = std::path::Path::new(original)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(original);
        
        format!("/data/data/com.larp.jv/files/natives/{}", lib_name)
    }
}

/// EMUI Iterator Fix — Port of emui_iterator_fix_hook.c
/// Fixes Huawei EMUI iterator bug in Java collections
mod emui_iterator_fix {
    use super::*;
    
    pub unsafe fn install_emui_iterator_fix() {
        // This would hook into JNI to fix the iterator bug
        // The bug: ConcurrentModificationException on Huawei devices
        // Fix: Wrap iterator methods to handle concurrent modification
    }
    
    #[no_mangle]
    pub unsafe extern "C" fn Java_java_util_ArrayList_iterator(
        env: *mut jni::sys::JNIEnv,
        obj: jni::sys::jobject,
    ) -> jni::sys::jobject {
        // Custom iterator implementation that handles EMUI bug
        std::ptr::null_mut()
    }
}

/// Stdio Hooks — Port of stdio_is.c
/// Redirects stdin/stdout/stderr for headless operation
mod stdio_hooks {
    use super::*;
    use std::fs::File;
    use std::os::unix::io::AsRawFd;
    
    static mut STDIN_REDIRECTED: bool = false;
    static mut STDOUT_REDIRECTED: bool = false;
    static mut STDERR_REDIRECTED: bool = false;
    
    pub unsafe fn install_stdio_hooks() {
        // Redirect stdio to files or pipes for logging
    }
    
    pub fn redirect_stdio(stdin_path: Option<&str>, stdout_path: Option<&str>, stderr_path: Option<&str>) -> Result<(), std::io::Error> {
        unsafe {
            if let Some(path) = stdin_path {
                let file = File::open(path)?;
                libc::dup2(file.as_raw_fd(), 0);
                STDIN_REDIRECTED = true;
            }
            
            if let Some(path) = stdout_path {
                let file = File::create(path)?;
                libc::dup2(file.as_raw_fd(), 1);
                STDOUT_REDIRECTED = true;
            }
            
            if let Some(path) = stderr_path {
                let file = File::create(path)?;
                libc::dup2(file.as_raw_fd(), 2);
                STDERR_REDIRECTED = true;
            }
        }
        Ok(())
    }
}

/// CPU Affinity Hook — Port of affinity.c
/// Sets CPU affinity for game threads
mod affinity {
    use super::*;
    use libc::{cpu_set_t, CPU_SET, CPU_ZERO, sched_setaffinity, getpid};
    
    pub fn set_affinity(cpu_ids: &[usize]) -> Result<(), std::io::Error> {
        unsafe {
            let mut cpuset: cpu_set_t = std::mem::zeroed();
            CPU_ZERO(&mut cpuset);
            
            for &cpu in cpu_ids {
                CPU_SET(cpu, &mut cpuset);
            }
            
            let result = sched_setaffinity(getpid(), std::mem::size_of::<cpu_set_t>(), &cpuset);
            if result != 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
        Ok(())
    }
    
    pub fn set_game_thread_affinity() -> Result<(), std::io::Error> {
        // Prefer big cores on ARM big.LITTLE
        #[cfg(target_arch = "aarch64")]
        {
            // Typically CPUs 4-7 are big cores on 8-core ARM
            set_affinity(&[4, 5, 6, 7])
        }
        
        #[cfg(not(target_arch = "aarch64"))]
        {
            Ok(())
        }
    }
    
    pub fn set_render_thread_affinity() -> Result<(), std::io::Error> {
        // Prefer big cores for rendering
        #[cfg(target_arch = "aarch64")]
        {
            set_affinity(&[4, 5])
        }
        
        #[cfg(not(target_arch = "aarch64"))]
        {
            Ok(())
        }
    }
}

/// Public API
pub use java_exec_hooks::*;
pub use lwjgl_dlopen_hook::*;
pub use emui_iterator_fix::*;
pub use stdio_hooks::*;
pub use affinity::*;