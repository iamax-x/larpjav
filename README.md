# larp.jv

**Pure Rust Minecraft Java Edition Launcher for Android and Desktop**

A complete rewrite of MojoLauncher/PojavLauncher in pure Rust, providing a fast, modern, and cross-platform Minecraft Java Edition launcher.

## Features

- **Pure Rust** - No JVM dependency in the launcher itself
- **Cross-platform** - Android (via NDK) and Desktop (Linux, Windows, macOS)
- **Modern UI** - Built with egui/wgpu for hardware-accelerated rendering
- **Full Mod Support** - Forge, Fabric, Quilt, NeoForge, OptiFine, Legacy Fabric, BTA
- **Microsoft & Ely.by Auth** - OAuth2 device code flow + XSTS authentication
- **Offline Mode** - Play without internet
- **Modpack Support** - MRPACK format support
- **Instance Management** - Multiple Minecraft instances with custom configs
- **JRE Management** - Automatic detection, download, and management of Java runtimes

## Architecture

```
larp.jv/
├── larp-jv-core/       # Pure Rust core library (no platform deps)
│   ├── auth/           # Microsoft, Ely.by, Offline auth
│   ├── downloader/     # Async download engine with verification
│   ├── instances/      # Instance management
│   ├── modding/        # Forge, Fabric, Quilt, NeoForge, OptiFine, etc.
│   ├── jre/            # JRE detection, download, classfile parsing
│   ├── mojson/         # Mojang JSON parsing (version manifests, rules)
│   ├── utils/          # Hash, ZIP, JSON, file, math, GPU, memory utils
│   └── value/          # Core data models
├── larp-jv-android/    # Android frontend (egui/wgpu + JNI)
├── larp-jv-desktop/    # Desktop frontend (egui/wgpu/winit)
├── larp-jv-native/     # Native libraries (mojoexec, JVM hooks, JNI dispatch)
└── larp-jv-build/      # Build tools
```

## Building

### Prerequisites

- Rust 1.75+ (2024 edition)
- Android NDK r25+ (for Android builds)
- CMake, Ninja (for native libraries)

### Desktop

```bash
cargo build --release -p larp-jv-desktop
```

### Android

```bash
# Install cargo-apk
cargo install cargo-apk

# Build APK
cd larp-jv-android
cargo apk build --release
```

## Configuration

The launcher stores configuration in:
- **Desktop**: `~/.config/larp.jv/`
- **Android**: App-private storage

## License

GPL-3.0-only (compatible with MojoLauncher/PojavLauncher)

## Credits

- Based on [MojoLauncher](https://github.com/MojoLauncher/MojoLauncher) (GPL-3.0)
- Inspired by [PojavLauncher](https://github.com/PojavLauncherTeam/PojavLauncher)
- Uses [egui](https://github.com/emilk/egui) for UI
- Uses [wgpu](https://github.com/gfx-rs/wgpu) for rendering

## Contributing

Contributions are welcome! Please read the SPEC.md for the architecture overview before contributing.