//! Build script for larp-jv-android — embeds assets and configures build

use std::fs;
use std::path::Path;

fn main() {
    // Tell cargo to re-run if assets change
    println!("cargo:rerun-if-changed=assets/");
    println!("cargo:rerun-if-changed=AndroidManifest.xml");
    
    // Embed assets
    embed_assets();
    
    // Generate Android manifest if needed
    // generate_manifest();
    
    // Link native libraries
    link_native_libs();
}

fn embed_assets() {
    let assets_dir = Path::new("assets");
    if !assets_dir.exists() {
        fs::create_dir_all(assets_dir).ok();
        return;
    }
    
    // Create assets list for android-assets crate
    let mut assets_list = Vec::new();
    
    for entry in walkdir::WalkDir::new(assets_dir) {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                let relative = path.strip_prefix(assets_dir).unwrap();
                assets_list.push(relative.to_string_lossy().to_string());
            }
        }
    }
    
    // Write assets list for android-assets
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let list_path = Path::new(&out_dir).join("assets_list.txt");
    fs::write(list_path, assets_list.join("\n")).ok();
    
    println!("cargo:rustc-env=ASSETS_DIR={}", assets_dir.display());
}

fn link_native_libs() {
    // Link native libraries for Android
    let target = std::env::var("TARGET").unwrap_or_default();
    
    if target.contains("android") {
        println!("cargo:rustc-link-search=native=../larp-jv-native/target/{}/release", 
                 std::env::var("PROFILE").unwrap_or_else(|_| "release".to_string()));
        println!("cargo:rustc-link-lib=static=mojoexec");
        println!("cargo:rustc-link-lib=static=jvm_hooks");
        
        // System libraries
        println!("cargo:rustc-link-lib=log");
        println!("cargo:rustc-link-lib=android");
        println!("cargo:rustc-link-lib=EGL");
        println!("cargo:rustc-link-lib=GLESv3");
        println!("cargo:rustc-link-lib=vulkan");
    }
}

#[cfg(feature = "generate-manifest")]
fn generate_manifest() {
    // Generate AndroidManifest.xml from template
    let manifest_template = include_str!("AndroidManifest.xml.template");
    let manifest = manifest_template
        .replace("${PACKAGE}", "com.larp.jv")
        .replace("${VERSION_CODE}", "1")
        .replace("${VERSION_NAME}", "0.1.0")
        .replace("${APP_NAME}", "larp.jv");
    
    fs::write("AndroidManifest.xml", manifest).ok();
}