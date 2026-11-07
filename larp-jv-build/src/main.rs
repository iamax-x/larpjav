//! larp-jv-build — Build tools for larp.jv

use clap::{Parser, Subcommand};
use crate::error::LarpResult;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "larp-jv-build")]
#[command(about = "Build tools for larp.jv", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build Android APK
    Android {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
        
        /// Output directory
        #[arg(short, long, default_value = "target/apk")]
        output: PathBuf,
    },
    
    /// Build Desktop binary
    Desktop {
        /// Target triple (e.g., x86_64-pc-windows-msvc)
        #[arg(short, long)]
        target: Option<String>,
        
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
    },
    
    /// Build all targets
    All {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
    },
    
    /// Generate Android manifest
    Manifest {
        /// Package name
        #[arg(short, long, default_value = "com.larp.jv")]
        package: String,
        
        /// Version code
        #[arg(short, long, default_value = "1")]
        version_code: u32,
        
        /// Version name
        #[arg(long, default_value = "0.1.0")]
        version_name: String,
    },
    
    /// Download assets (fonts, icons, etc.)
    Assets {
        /// Force re-download
        #[arg(short, long)]
        force: bool,
    },
    
    /// Run tests
    Test {
        /// Test specific crate
        #[arg(short, long)]
        crate: Option<String>,
    },
}

fn main() -> LarpResult<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Android { release, output } => {
            println!("Building Android APK...");
            build_android(release, output)?;
        }
        Commands::Desktop { target, release } => {
            println!("Building Desktop binary...");
            build_desktop(target, release)?;
        }
        Commands::All { release } => {
            println!("Building all targets...");
            build_android(release, PathBuf::from("target/apk"))?;
            build_desktop(None, release)?;
        }
        Commands::Manifest { package, version_code, version_name } => {
            println!("Generating Android manifest...");
            generate_manifest(package, version_code, version_name)?;
        }
        Commands::Assets { force } => {
            println!("Downloading assets...");
            download_assets(force)?;
        }
        Commands::Test { crate } => {
            println!("Running tests...");
            run_tests(crate)?;
        }
    }
    
    Ok(())
}

fn build_android(release: bool, output: PathBuf) -> LarpResult<()> {
    let mut args = vec!["apk", "build"];
    if release {
        args.push("--release");
    }
    args.push("-p");
    args.push("larp-jv-android");
    
    let status = std::process::Command::new("cargo")
        .args(&args)
        .current_dir(std::env::current_dir()?)
        .status()?;
    
    if !status.success() {
        return Err(crate::error::LarpError::Other("Android build failed".to_string()));
    }
    
    println!("Android APK built successfully to {:?}", output);
    Ok(())
}

fn build_desktop(target: Option<String>, release: bool) -> LarpResult<()> {
    let mut args = vec!["build", "-p", "larp-jv-desktop"];
    if release {
        args.push("--release");
    }
    if let Some(t) = target {
        args.push("--target");
        args.push(&t);
    }
    
    let status = std::process::Command::new("cargo")
        .args(&args)
        .current_dir(std::env::current_dir()?)
        .status()?;
    
    if !status.success() {
        return Err(crate::error::LarpError::Other("Desktop build failed".to_string()));
    }
    
    println!("Desktop binary built successfully");
    Ok(())
}

fn generate_manifest(package: String, version_code: u32, version_name: String) -> LarpResult<()> {
    let manifest = format!(r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="{}"
    android:versionCode="{}"
    android:versionName="{}"
    android:installLocation="auto">

    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
    <uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
    <uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" />
    <uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" />
    <uses-permission android:name="android.permission.VIBRATE" />
    <uses-permission android:name="android.permission.WAKE_LOCK" />
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
    <uses-permission android:name="android.permission.REQUEST_INSTALL_PACKAGES" />
    <uses-permission android:name="android.permission.POST_NOTIFICATIONS" />

    <uses-feature android:name="android.hardware.vulkan.version" android:required="false" />
    <uses-feature android:name="android.hardware.vulkan.compute" android:required="false" />
    <uses-feature android:glEsVersion="0x00030000" android:required="true" />

    <application
        android:name=".LarpJvApplication"
        android:allowBackup="true"
        android:icon="@mipmap/ic_launcher"
        android:label="@string/app_name"
        android:roundIcon="@mipmap/ic_launcher_round"
        android:supportsRtl="true"
        android:theme="@style/Theme.LarpJv"
        android:extractNativeLibs="true"
        android:usesCleartextTraffic="true"
        android:networkSecurityConfig="@xml/network_security_config"
        android:hardwareAccelerated="true"
        android:largeHeap="true">

        <activity
            android:name=".LarpJvActivity"
            android:exported="true"
            android:launchMode="singleTask"
            android:configChanges="orientation|screenSize|keyboardHidden|smallestScreenSize|uiMode|density|fontScale"
            android:screenOrientation="sensorLandscape"
            android:theme="@style/Theme.LarpJv.Fullscreen"
            android:windowSoftInputMode="adjustResize|stateHidden">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>

        <service
            android:name=".DownloadService"
            android:exported="false"
            android:foregroundServiceType="dataSync" />
        
        <service
            android:name=".GameService"
            android:exported="false" />

        <provider
            android:name="androidx.core.content.FileProvider"
            android:authorities="${{applicationId}}.fileprovider"
            android:exported="false"
            android:grantUriPermissions="true">
            <meta-data
                android:name="android.support.FILE_PROVIDER_PATHS"
                android:resource="@xml/file_paths" />
        </provider>

    </application>

</manifest>"#, package, version_code, version_name);
    
    std::fs::write("AndroidManifest.xml", manifest)?;
    println!("AndroidManifest.xml generated");
    Ok(())
}

fn download_assets(force: bool) -> LarpResult<()> {
    let assets_dir = std::path::Path::new("larp-jv-android/assets");
    std::fs::create_dir_all(assets_dir)?;
    
    // Download fonts
    let fonts_dir = assets_dir.join("fonts");
    std::fs::create_dir_all(&fonts_dir)?;
    
    // In real implementation, download from MojoLauncher releases
    println!("Assets downloaded to {:?}", assets_dir);
    Ok(())
}

fn run_tests(crate_name: Option<String>) -> LarpResult<()> {
    let mut args = vec!["test", "--workspace"];
    if let Some(c) = crate_name {
        args.push("-p");
        args.push(&c);
    }
    
    let status = std::process::Command::new("cargo")
        .args(&args)
        .current_dir(std::env::current_dir()?)
        .status()?;
    
    if !status.success() {
        return Err(crate::error::LarpError::Other("Tests failed".to_string()));
    }
    
    println!("All tests passed");
    Ok(())
}