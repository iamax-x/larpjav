//! FreeType binding — font rendering (replaces libfreetype.so)

use std::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;

/// FreeType wrapper - binds to system freetype library
pub struct FreeType {
    lib: Mutex<Option<libloading::Library>>,
    face_cache: Mutex<HashMap<String, Face>>,
}

#[derive(Debug, Clone)]
pub struct Face {
    pub family_name: String,
    pub style_name: String,
    pub size: u32,
}

impl FreeType {
    pub fn new() -> Self {
        Self {
            lib: Mutex::new(None),
            face_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Load freetype library
    pub fn load(&self) -> crate::error::LarpResult<()> {
        let lib = unsafe {
            #[cfg(target_os = "linux")]
            libloading::Library::new("libfreetype.so.6")
                .or_else(|_| libloading::Library::new("libfreetype.so"))?;
            
            #[cfg(target_os = "android")]
            libloading::Library::new("libfreetype.so")?;
            
            #[cfg(target_os = "windows")]
            libloading::Library::new("freetype.dll")?;
            
            #[cfg(target_os = "macos")]
            libloading::Library::new("libfreetype.dylib")?;
        };
        
        *self.lib.lock().unwrap() = Some(lib);
        log::info!("FreeType loaded successfully");
        Ok(())
    }

    /// Load a font face
    pub fn load_face(&self, path: &str, size: u32) -> crate::error::LarpResult<Face> {
        // In real implementation, use FreeType C API via libloading
        // For now, create a placeholder
        let face = Face {
            family_name: "Unknown".to_string(),
            style_name: "Regular".to_string(),
            size,
        };
        
        self.face_cache.lock().unwrap().insert(
            format!("{}@{}", path, size),
            face.clone(),
        );
        
        Ok(face)
    }

    /// Get cached face
    pub fn get_face(&self, path: &str, size: u32) -> Option<Face> {
        self.face_cache.lock().unwrap()
            .get(&format!("{}@{}", path, size))
            .cloned()
    }

    /// Check if loaded
    pub fn is_loaded(&self) -> bool {
        self.lib.lock().unwrap().is_some()
    }
}

impl Default for FreeType {
    fn default() -> Self {
        Self::new()
    }
}

/// Global instance
lazy_static::lazy_static! {
    pub static ref FREETYPE: FreeType = FreeType::new();
}

/// Initialize FreeType
pub fn init() -> crate::error::LarpResult<()> {
    FREETYPE.load()?;
    log::info!("FreeType initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freetype_init() {
        let _ = init();
    }
}