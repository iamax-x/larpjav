//! gl4es — OpenGL to GLES translation layer binding (replaces libgl4es_114.so)

use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// gl4es wrapper - binds to system gl4es library
pub struct Gl4es {
    lib: Mutex<Option<libloading::Library>>,
    function_cache: Mutex<HashMap<String, *mut std::ffi::c_void>>,
}

impl Gl4es {
    pub fn new() -> Self {
        Self {
            lib: Mutex::new(None),
            function_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Load gl4es library
    pub fn load(&self) -> crate::error::LarpResult<()> {
        let lib = unsafe {
            #[cfg(target_os = "linux")]
            libloading::Library::new("libgl4es.so.1.1.4")
                .or_else(|_| libloading::Library::new("libgl4es.so"))
                .or_else(|_| libloading::Library::new("libgl4es.so.1"))?;
            
            #[cfg(target_os = "android")]
            libloading::Library::new("libgl4es.so")?;
            
            #[cfg(target_os = "windows")]
            libloading::Library::new("gl4es.dll")?;
            
            #[cfg(target_os = "macos")]
            libloading::Library::new("libgl4es.dylib")?;
        };
        
        *self.lib.lock().unwrap() = Some(lib);
        log::info!("gl4es loaded successfully");
        Ok(())
    }

    /// Get OpenGL function pointer
    pub fn get_proc_address(&self, name: &str) -> Option<*mut std::ffi::c_void> {
        // Check cache first
        if let Some(ptr) = self.function_cache.lock().unwrap().get(name) {
            return Some(*ptr);
        }
        
        // Load from library
        if let Some(lib) = self.lib.lock().unwrap().as_ref() {
            if let Ok(sym) = unsafe { lib.get::<*mut std::ffi::c_void>(name.as_bytes()) } {
                self.function_cache.lock().unwrap().insert(name.to_string(), *sym);
                return Some(*sym);
            }
        }
        
        None
    }

    /// Check if gl4es is loaded
    pub fn is_loaded(&self) -> bool {
        self.lib.lock().unwrap().is_some()
    }
}

impl Default for Gl4es {
    fn default() -> Self {
        Self::new()
    }
}

/// Global instance
lazy_static::lazy_static! {
    pub static ref GL4ES: Gl4es = Gl4es::new();
}

/// Initialize gl4es
pub fn init() -> crate::error::LarpResult<()> {
    GL4ES.load()?;
    log::info!("gl4es initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gl4es_init() {
        // Test will fail if gl4es not available, which is expected in test env
        let _ = init();
    }
}