//! Minecraft options.txt parsing

use crate::error::LarpResult;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Parse options.txt into key-value pairs
pub fn parse_options(path: &Path) -> LarpResult<HashMap<String, String>> {
    let content = fs::read_to_string(path)?;
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if let Some((key, value)) = line.split_once(':') {
            map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    Ok(map)
}

/// Write options to file
pub fn write_options(path: &Path, options: &HashMap<String, String>) -> LarpResult<()> {
    let mut lines = Vec::new();
    for (key, value) in options {
        lines.push(format!("{}:{}", key, value));
    }
    fs::write(path, lines.join("\n"))?;
    Ok(())
}

/// Get option with default
pub fn get_option(options: &HashMap<String, String>, key: &str, default: &str) -> String {
    options.get(key).cloned().unwrap_or_else(|| default.to_string())
}

/// Get option as bool
pub fn get_option_bool(options: &HashMap<String, String>, key: &str, default: bool) -> bool {
    options.get(key)
        .map(|v| v.parse().unwrap_or(default))
        .unwrap_or(default)
}

/// Get option as int
pub fn get_option_int(options: &HashMap<String, String>, key: &str, default: i32) -> i32 {
    options.get(key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Get option as float
pub fn get_option_float(options: &HashMap<String, String>, key: &str, default: f32) -> f32 {
    options.get(key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Set option
pub fn set_option(options: &mut HashMap<String, String>, key: &str, value: &str) {
    options.insert(key.to_string(), value.to_string());
}

/// Common Minecraft option keys
pub mod keys {
    pub const MOUSE_SENSITIVITY: &str = "mouseSensitivity";
    pub const FOV: &str = "fov";
    pub const GAMMA: &str = "gamma";
    pub const RENDER_DISTANCE: &str = "renderDistance";
    pub const VIEW_BOBBING: &str = "viewBobbing";
    pub const AUTO_JUMP: &str = "autoJump";
    pub const INVERT_MOUSE: &str = "invertMouse";
    pub const TOUCHSCREEN: &str = "touchscreen";
    pub const CHAT_VISIBILITY: &str = "chatVisibility";
    pub const CHAT_COLORS: &str = "chatColors";
    pub const CHAT_LINKS: &str = "chatLinks";
    pub const CHAT_OPACITY: &str = "chatOpacity";
    pub const CHAT_LINKS_PROMPT: &str = "chatLinksPrompt";
    pub const CHAT_SCALE: &str = "chatScale";
    pub const CHAT_WIDTH: &str = "chatWidth";
    pub const CHAT_HEIGHT_FOCUSED: &str = "chatHeightFocused";
    pub const CHAT_HEIGHT_UNFOCUSED: &str = "chatHeightUnfocused";
    pub const MIPMAP_LEVELS: &str = "mipmapLevels";
    pub const USE_VSYNC: &str = "useVSync";
    pub const FULLSCREEN: &str = "fullscreen";
    pub const ENABLE_VSYNC: &str = "enableVsync";
    pub const USE_FBO: &str = "useFbo";
    pub const ENTITY_SHADOWS: &str = "entityShadows";
    pub const FORCE_UNICODE_FONT: &str = "forceUnicodeFont";
    pub const CLOUDS: &str = "clouds";
    pub const GRAPHICS: &str = "graphics";
    pub const AMBIENT_OCCLUSION: &str = "ambientOcclusion";
    pub const PARTICLES: &str = "particles";
    pub const RENDER_CLOUDS: &str = "renderClouds";
    pub const RESOURCE_PACKS: &str = "resourcePacks";
    pub const INCOMPATIBLE_RESOURCE_PACKS: &str = "incompatibleResourcePacks";
    pub const LANGUAGE: &str = "lang";
    pub const LAST_SERVER: &str = "lastServer";
    pub const HIDE_SERVER_ADDRESS: &str = "hideServerAddress";
    pub const ADVANCED_ITEM_TOOLTIPS: &str = "advancedItemTooltips";
    pub const PAUSE_ON_LOST_FOCUS: &str = "pauseOnLostFocus";
    pub const TOUCHSCREEN_MODE: &str = "touchscreenMode";
    pub const OVERRIDE_WIDTH: &str = "overrideWidth";
    pub const OVERRIDE_HEIGHT: &str = "overrideHeight";
    pub const CUSTOM_SKIN: &str = "customSkin";
    pub const SKIN_URL: &str = "skinUrl";
    pub const CAPE_URL: &str = "capeUrl";
    pub const SKIN_MODEL: &str = "skinModel";
    pub const NARRATOR: &str = "narrator";
    pub const ENABLE_GL_ERROR_CHECK: &str = "enableGlErrorCheck";
    pub const USE_GL_DEBUG: &str = "useGlDebug";
    pub const RENDER_DISTANCE_CHUNKS: &str = "renderDistanceChunks";
    pub const SIMULATION_DISTANCE: &str = "simulationDistance";
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_parse_write_options() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "mouseSensitivity:0.5").unwrap();
        writeln!(file, "fov:90").unwrap();
        writeln!(file, "# comment").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "renderDistance:12").unwrap();

        let opts = parse_options(file.path()).unwrap();
        assert_eq!(opts.get("mouseSensitivity"), Some(&"0.5".to_string()));
        assert_eq!(opts.get("fov"), Some(&"90".to_string()));
        assert_eq!(opts.get("renderDistance"), Some(&"12".to_string()));
        assert!(!opts.contains_key("# comment"));
    }
}
