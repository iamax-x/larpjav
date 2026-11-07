# larp.jv — Rust Port of MojoLauncher

**Original**: MojoLauncher (Java/Kotlin + JNI C/C++) — Minecraft Java Edition launcher for Android
**Target**: Pure Rust, cross-platform (Android, iOS, Desktop), zero-JVM dependency
**Name**: `larp.jv` (package: `com.larp.jv`)

---

## Architecture Overview

```
larp.jv/
├── larp-jv-core/           # Pure Rust core library (no platform deps)
│   ├── auth/               # Microsoft, Ely.by, Offline auth
│   ├── downloader/         # Async download + verification (SHA1, SHA256)
│   ├── instances/          # Instance management, profiles, versions
│   ├── modding/            # Forge, Fabric, Quilt, NeoForge, OptiFine, LegacyFabric
│   ├── jre/                # JRE management, Java runner, classfile parsing
│   ├── mojson/             # Mojang JSON parsing (version manifests, rules)
│   ├── utils/              # Hash, ZIP, JSON, File, Math, GPU, Memory utils
│   └── value/              # Core data models (Library, Rule, Substitution, etc.)
├── larp-jv-android/        # Android frontend (egui/wgpu + JNI glue)
│   ├── ui/                 # egui-based UI (port of all fragments/activities)
│   ├── platform/           # Android-specific: clipboard, vibration, permissions
│   ├── input/              # Touch, gamepad, mouse/keyboard mapping
│   ├── render/             # wgpu/GLES backend for game rendering
│   └── jni/                # Minimal JNI bridge for Android APIs
├── larp-jv-desktop/        # Desktop frontend (egui/wgpu)
│   ├── ui/                 # Desktop UI (egui)
│   ├── platform/           # Desktop-specific: file dialogs, system tray
│   └── render/             # wgpu/Vulkan/Metal/DX12 backend
├── larp-jv-native/         # Native libraries (replacing JNI .so files)
│   ├── gl4es/              # OpenGL → GLES translation (port from C)
│   ├── freetype/           # Font rendering (bind to system freetype)
│   ├── jnidispatch/        # JNA replacement (pure Rust FFI)
│   ├── unpack200/          # Pack200 unpacker (pure Rust)
│   ├── mojoexec/           # JVM launcher (replaces mojoexec C code)
│   └── jvm-hooks/          # LD_PRELOAD hooks (port from C)
├── larp-jv-build/          # Build system (cargo xtask, bindgen, cbindgen)
└── assets/                 # Fonts, icons, shaders, default configs
```

---

## Module-by-Module Porting Map

### 1. Core Library (`larp-jv-core`)

#### `auth/` — Authentication
| Java File | Rust Module | Notes |
|-----------|-------------|-------|
| `AuthType.java` | `auth::types` | Enum: Microsoft, ElyBy, Offline |
| `Account.java` / `Accounts.java` | `auth::account` | Account storage, encryption (AES-GCM) |
| `BackgroundLogin.java` | `auth::background` | Async trait + implementations |
| `MicrosoftBackgroundLogin.java` | `auth::microsoft` | OAuth2 device code flow + XSTS |
| `ElyByBackgroundLogin.java` | `auth::elyby` | Ely.by API |
| `CommonLoginUtils.java` | `auth::common` | Shared utilities |
| `OAuthTokenResponse.java` | `auth::oauth` | Token models |
| `LoginListener.java` | `auth::listener` | Callback trait |
| `SkinHeadRenderer.java` | `auth::skin` | Skin/cape rendering (egui texture) |

#### `downloader/` — Download Engine
| Java File | Rust Module | Notes |
|-----------|-------------|-------|
| `Downloader.java` | `downloader::engine` | Async tokio-based, resumable |
| `DownloaderTask.java` / `TaskMetadata.java` | `downloader::task` | Task state machine |
| `DownloadFileTask.java` | `downloader::file` | File download with progress |
| `CheckFileOnDiskTask.java` | `downloader::verify` | Hash verification (SHA1/256) |
| `CompleteMetadataTask.java` | `downloader::metadata` | Post-download processing |
| `AcquireableTaskMetadata.java` | `downloader::acquire` | Streaming metadata |
| `BytesCopiedListener.java` / `ProgressListener.java` | `downloader::progress` | Progress callbacks |
| `VerificationException.java` | `downloader::error` | Custom error types |

#### `instances/` — Instance Management
| Java File | Rust Module | Notes |
|-----------|-------------|-------|
| `Instance.java` / `DisplayInstance.java` | `instances::model` | Instance data, serialization |
| `InstanceAdapter.java` / `InstanceAdapterExtra.java` | `instances::adapter` | UI adapters (egui) |
| `VersionListAdapter.java` / `VersionSelectorDialog.java` | `instances::version` | Version listing/selection |

#### `modding/` — Mod Loader Support
| Java File | Rust Module | Notes |
|-----------|-------------|-------|
| `ForgeInstallFragment.java` / `ForgeInstaller.java` | `modding::forge` | Forge installer (port from forge_installer module) |
| `FabricInstallFragment.java` / `FabriclikeInstallFragment.java` | `modding::fabric` | Fabric/Quilt/NeoForge unified |
| `QuiltInstallFragment.java` | `modding::quilt` | Quilt-specific |
| `NeoforgeInstallFragment.java` | `modding::neoforge` | NeoForge-specific |
| `LegacyFabricInstallFragment.java` | `modding::legacy_fabric` | Legacy Fabric |
| `OptiFineInstallFragment.java` / `OptiFine*.java` | `modding::optifine` | OptiFine scraper + installer |
| `BTAInstallFragment.java` | `modding::bta` | Better Than Adventure |
| `ModVersionListFragment.java` / `SearchModFragment.java` | `modding::search` | Modrinth/CurseForge API |
| `ModpackInstallFragment.java` / `ModpackManager.java` | `modding::modpack` | Modpack (MRPACK) support |

#### `jre/` — Java Runtime Environment
| Java File | Rust Module | Notes |
|-----------|-------------|-------|
| `JavaRunner.java` / `GameRunner.java` | `jre::runner` | Process spawning, arg building |
| `RuntimeSelectionException.java` / `VMLoadException.java` | `jre::error` | Error types |
| `JREUtils.java` / `NewJREUtil.java` | `jre::utils` | JRE detection, download, extraction |
| `ClassVersionReader.java` / `ConstantFieldReader.java` | `jre::classfile` | Class file parsing (version, constants) |
| `GameRunner.java` | `jre::game` | Game launch orchestration |

#### `mojson/` — Mojang JSON Parsing
| Java File | Rust Module | Notes |
|-----------|-------------|-------|
| `MoJsonDownloader.java` / `MoJsonExtras.java` | `mojson::downloader` | Version manifest downloading |
| `MoJsonRule.java` | `mojson::rule` | Rule evaluation (os, features, has_custom_resolution) |
| `DownloadMirror.java` / `MirrorTamperedException.java` | `mojson::mirror` | Mirror selection, tamper detection |
| `LibraryArtifact.java` / `DependentLibrary.java` | `mojson::library` | Library models |
| `LibrarySubstitution.java` / `SubstitutionMap.java` | `mojson::substitution` | Library substitutions |
| `NativeLibraryExtractable.java` / `ExtractSettings.java` | `mojson::native` | Native extraction config |
| `ClientInfo.java` | `mojson::client` | Client metadata |

#### `utils/` — Utilities
| Java File | Rust Module | Notes |
|-----------|-------------|-------|
| `HashUtils.java` | `utils::hash` | SHA1, SHA256, MD5 (ring crate) |
| `ZipUtils.java` | `utils::zip` | ZIP extraction (zip-rs) |
| `FileUtils.java` | `utils::file` | File operations |
| `JSONUtils.java` / `GsonJsonUtils.java` | `utils::json` | serde_json + custom serializers |
| `DownloadUtils.java` | `utils::download` | HTTP utilities (reqwest) |
| `GpuUtils.java` | `utils::gpu` | GPU detection (vulkan/gles) |
| `MemoryHoleFinder.java` / `SelfMapsParser.java` | `utils::memory` | /proc/self/maps parsing |
| `MathUtils.java` / `MatrixUtils.java` | `utils::math` | Math helpers |
| `KeycodeUtils.java` | `utils::keycode` | Keycode mapping (Android ↔ GLFW) |
| `MCOptionUtils.java` / `GameOptionsUtils.java` | `utils::options` | options.txt parsing |
| `NotificationUtils.java` | `utils::notification` | System notifications |
| `SignatureCheckUtil.java` | `utils::signature` | JAR signature verification |
| `DateUtils.java` / `LocaleUtils.java` | `utils::locale` | Date/locale helpers |
| `FilteredSubList.java` | `utils::collections` | Collection utilities |

#### `value/` — Core Value Types
| Java File | Rust Module | Notes |
|-----------|-------------|-------|
| `LibraryArtifact.java` | `value::library` | |
| `DependentLibrary.java` | `value::dependent` | |
| `LibrarySubstitution.java` | `value::substitution` | |
| `MoJsonRule.java` | `value::rule` | |
| `NativeLibraryExtractable.java` | `value::native` | |
| `ExtractSettings.java` | `value::extract` | |
| `ClientInfo.java` | `value::client` | |
| `SubstitutionMap.java` | `value::subst_map` | |

---

### 2. Android Frontend (`larp-jv-android`)

#### UI Components (egui port)
| Java Fragment/Activity | Rust Module | Notes |
|------------------------|-------------|-------|
| `LauncherActivity.java` | `android::ui::launcher` | Main entry |
| `MainMenuFragment.java` | `android::ui::main_menu` | Home screen |
| `InstanceEditorFragment.java` | `android::ui::instance_editor` | Create/edit instances |
| `MicrosoftLoginFragment.java` / `ElyByLoginFragment.java` / `LocalLoginFragment.java` | `android::ui::auth` | Login screens |
| `SelectAuthFragment.java` | `android::ui::auth_select` | Auth method picker |
| `FabricInstallFragment.java` / `ForgeInstallFragment.java` / etc. | `android::ui::mod_install` | Mod installer UIs |
| `GamepadMapperFragment.java` | `android::ui::gamepad_mapper` | Gamepad mapping UI |
| `CustomControlsActivity.java` / fragments | `android::ui::custom_controls` | Touch control editor |
| `LauncherPreference*.java` | `android::ui::settings` | Settings screens (egui) |
| `BTAInstallFragment.java` | `android::ui::bta_install` | |
| `ModVersionListFragment.java` / `SearchModFragment.java` | `android::ui::mod_browser` | |
| `ProfileTypeSelectFragment.java` | `android::ui::profile_type` | |
| `FileSelectorFragment.java` | `android::ui::file_picker` | |
| `WebViewCompletionFragment.java` / `OAuthFragment.java` | `android::ui::webview` | OAuth webview |

#### Game Rendering (`android::render`)
| Java File | Rust Module | Notes |
|-----------|-------------|-------|
| `GameActivity.java` / `GameView.java` | `android::render::game_view` | egui + wgpu surface |
| `GameRenderer.java` / `GLESRenderSpec.java` / `MesaRenderSpec.java` | `android::render::renderer` | wgpu backend |
| `SurfaceProvider.java` / `SurfaceViewSurfaceProvider.java` / `TextureViewSurfaceProvider.java` | `android::render::surface` | ANativeWindow integration |
| `Platform.java` / backends (AWT, GLFW, SDL, Dummy) | `android::render::platform` | Platform abstraction |
| `InGameEventProcessor.java` / `InGUIEventProcessor.java` / `TouchEventProcessor.java` | `android::render::input` | Input processing |
| `GameCursorView.java` | `android::render::cursor` | Custom cursor rendering |

#### Input System (`android::input`)
| Java File | Rust Module | Notes |
|-----------|-------------|-------|
| `ControlLayout.java` / `CustomControls.java` | `android::input::layout` | Touch layout system |
| `ControlButton.java` / `ControlJoystick.java` / `ControlDrawer.java` | `android::input::widgets` | Touch widgets |
| `Gamepad*.java` | `android::input::gamepad` | Gamepad mapping |
| `Mouse/*.java` (Touchpad, Gestures, Click, Scroll) | `android::input::mouse` | Virtual mouse |
| `Keyboard/TouchCharInput.java` | `android::input::keyboard` | Virtual keyboard |
| `HandleView/*.java` | `android::input::editor` | Control editor UI |

#### Platform Integration (`android::platform`)
| Java File | Rust Module | Notes |
|-----------|-------------|-------|
| `BaseActivity.java` / `MojoApplication.java` | `android::platform::app` | App lifecycle |
| `CallbackBridge.java` | `android::platform::callback` | Kotlin/Rust bridge |
| `Tools.java` | `android::platform::tools` | Android utilities |
| `Vulkan/Driver*.java` / `DriverManager.java` | `android::platform::vulkan` | Vulkan driver install |
| `MultiRT*.java` | `android::platform::multirt` | Multi-runtime management |

---

### 3. Desktop Frontend (`larp-jv-desktop`)
- Same UI modules as Android but using native egui/wgpu
- Platform modules: file dialogs (rfd), system tray (tray-icon), notifications
- Render: wgpu with Vulkan/Metal/DX12/OpenGL backends

---

### 4. Native Libraries (`larp-jv-native`)

| Native Component | Source | Rust Approach |
|------------------|--------|---------------|
| `gl4es` (OpenGL→GLES) | `jni/awt/`, `jni/glfw/` | Port to Rust OR bind to system gl4es via dynamic loading |
| `freetype` | `jniLibs/*/libfreetype.so` | `freetype-rs` crate (system lib) |
| `jnidispatch` (JNA) | `jniLibs/*/libjnidispatch.so` | `jni` crate + custom FFI — replace JNA entirely |
| `unpack200` | `jniLibs/*/libunpack200.so` | `pack200` crate (pure Rust) or bind system |
| `mojoexec` (JVM launcher) | `jni/jre_launcher/`, `jni/mojoexec/` | **Pure Rust reimplementation** — spawn JVM process directly |
| `jvm_hooks` (LD_PRELOAD) | `jni/jvm_hooks/` | Port hooks to Rust (use `libc`/`nix` for LD_PRELOAD) |
| `awt` / `awt_xawt` | `jni/awt/`, `jni/awt_xawt/` | Replace with pure Rust AWT implementation (headless) |
| `minibridge` | `jni/minibridge.c` | Remove — not needed in pure Rust |
| `affinity.c` / `stdio_is.c` | `jni/` | Rust std::thread + std::os::fd |

---

### 5. Submodules (External Dependencies)

| Submodule | Purpose | Rust Replacement |
|-----------|---------|------------------|
| `glfw` | Window/input (desktop) | `glfw-rs` crate (or `winit` + `raw-window-handle`) |
| `sdl` | Gamepad/audio (Android) | `sdl2-rs` / `sdl3-rs` crate |
| `mojoexec` | Custom JVM launcher | **Pure Rust** (see above) |
| `NoBlocklist` | Mojang blocklist bypass | Built into `mojson::rule` evaluation |

---

## Build System

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "larp-jv-core",
    "larp-jv-android",
    "larp-jv-desktop",
    "larp-jv-native",
    "larp-jv-build",
]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1.40", features = ["full", "rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.12", features = ["json", "rustls-tls", "gzip", "brotli"] }
ring = "0.17"
zip = "0.6"
egui = { version = "0.30", features = ["wgpu", "accesskit"] }
wgpu = { version = "0.19", features = ["wgpu-hal"] }
winit = "0.30"
jni = "0.21"
ndk = "0.9"  # Android NDK bindings
ndk-sys = "0.9"
anyhow = "1.0"
thiserror = "1.0"
tracing = { version = "0.1", features = ["log"] }
parking_lot = "0.12"
dashmap = "6.1"
```

**Android Build**: `cargo apk` (via `cargo-apk` or `ndk-build` integration)
**Desktop Build**: Standard `cargo build --release`

---

## Porting Strategy

### Phase 1: Core Library (Pure Rust, No Platform Deps)
1. **Data Models** (`value/`, `mojson::library`, `auth::account`) — serde + validation
2. **Utils** (`hash`, `zip`, `json`, `file`, `download`, `math`) — direct crate mappings
3. **Auth** (Microsoft OAuth2, ElyBy, Offline) — `reqwest` + `async-trait`
4. **Downloader** — `tokio` + `bytes` + resumable downloads
5. **JRE Management** — Detect, download, extract JREs (Zulu, Eclipse Temurin)
6. **Modding** — Forge/Fabric/Quilt/NeoForge/OptiFine installers
6. **MoJson** — Version manifest parsing, rule evaluation, library resolution
7. **Instances** — Serialization (TOML/JSON), version management

### Phase 2: Native Layer
1. **mojoexec** — Pure Rust JVM launcher (replaces C `jre_launcher`)
2. **JVM Hooks** — LD_PRELOAD hooks in Rust
3. **JNI Dispatch** — Replace JNA with `jni` crate
4. **gl4es/freetype/unpack200** — Bind to system libs or pure Rust alternatives

### Phase 3: Android Frontend
1. **egui/wgpu Setup** — `ndk-glue` + `wgpu` Android backend
2. **UI Components** — Port all fragments/activities to egui
3. **Input System** — Touch, gamepad, virtual mouse/keyboard
4. **Game Rendering** — Surface management, renderer backends
5. **Platform Integration** — Permissions, Vulkan drivers, multi-runtime

### Phase 4: Desktop Frontend
1. **egui/wgpu/winit** — Standard desktop stack
2. **UI Components** — Reuse core UI logic from Android
3. **Platform Integration** — File dialogs, tray, notifications

### Phase 5: Polish & Optimization
1. **Binary Size** — LTO, strip, panic=abort
2. **Startup Time** — Lazy loading, async init
3. **Memory** — Arena allocators, pooling
4. **Testing** — Unit tests, integration tests, fuzzing

---

## Critical Implementation Notes

### 1. **Zero-JVM Architecture**
- No `jni` crate in core — only in Android frontend for platform APIs
- JVM launched as **child process** via `mojoexec` (Rust)
- Classfile parsing done in Rust (`jre::classfile`)
- No Java reflection, no JNI calls from core

### 2. **Async-First Design**
- All I/O: `tokio` + `async-trait`
- Downloads: resumable, parallel, verified
- UI: egui runs on main thread; core runs on tokio runtime

### 3. **Cross-Platform Abstractions**
```rust
// larp-jv-core/src/platform.rs
pub trait Platform: Send + Sync {
    fn clipboard(&self) -> Box<dyn Clipboard>;
    fn notifications(&self) -> Box<dyn Notifications>;
    fn file_dialog(&self) -> Box<dyn FileDialog>;
    fn gpu_info(&self) -> GpuInfo;
    fn vulkan_driver_install(&self) -> Result<()>;
}
```

### 4. **Android-Specific**
- `ndk-glue` for `android_main` entry point
- `jni` crate only for: clipboard, vibration, permissions, system APIs
- Assets bundled via `android-assets` crate
- Shared library: `cdylib` output

### 5. **Error Handling**
```rust
// larp-jv-core/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum LarpError {
    #[error("Auth failed: {0}")] Auth(#[from] AuthError),
    #[error("Download failed: {0}")] Download(#[from] DownloadError),
    #[error("JRE error: {0}")] Jre(#[from] JreError),
    #[error("Mod install failed: {0}")] Modding(#[from] ModdingError),
    #[error("IO error: {0}")] Io(#[from] std::io::Error),
    // ...
}
```

### 6. **Configuration**
- `config.toml` for user settings (replaces `SharedPreferences`)
- `instances/*.toml` for instance configs
- `launcher_profiles.json` compatibility layer

---

## Testing Strategy

| Layer | Approach |
|-------|----------|
| Core | `cargo test` — unit + property tests (proptest) |
| Downloader | Mock HTTP server (`mockito`/`wiremock`) |
| Auth | Integration tests with test accounts |
| Modding | Test against real Forge/Fabric installers |
| JRE | Test with multiple JRE vendors/versions |
| Android | `cargo test --target aarch64-linux-android` + device tests |
| Desktop | `cargo test` + CI on Linux/macOS/Windows |

---

## Migration Checklist

- [ ] Initialize workspace with `cargo new --workspace`
- [ ] Port `value/` models (serde + validation)
- [ ] Port `utils/` (hash, zip, json, file, download, math)
- [ ] Port `auth/` (Microsoft, ElyBy, Offline)
- [ ] Port `downloader/` (async, resumable, verified)
- [ ] Port `jre/` (detection, download, classfile parsing, runner)
- [ ] Port `mojson/` (manifest, rules, libraries, mirrors)
- [ ] Port `instances/` (serialization, version management)
- [ ] Port `modding/` (Forge, Fabric, Quilt, NeoForge, OptiFine, BTA, Modpacks)
- [ ] Implement `mojoexec` (pure Rust JVM launcher)
- [ ] Implement JVM hooks (LD_PRELOAD)
- [ ] Android frontend: egui/wgpu + JNI glue
- [ ] Desktop frontend: egui/wgpu/winit
- [ ] Assets: fonts, icons, shaders, default configs
- [ ] CI/CD: GitHub Actions (Linux/macOS/Windows/Android)
- [ ] Release: `cargo dist` / `cargo apk`

---

## Resources to Bundle

| Asset | Source | Target |
|-------|--------|--------|
| `minecraft_ten.ttf` | `app_mojo/src/main/res/font/` | `assets/fonts/` |
| `noto_sans_bold.ttf` | `app_mojo/src/main/res/font/` | `assets/fonts/` |
| `mojo.png` / icons | `app_mojo/src/main/assets/` / `res/drawable*/` | `assets/icons/` |
| `default.json` / `options.txt` / `substitutions.json` | `app_mojo/src/main/assets/` | `assets/config/` |
| `cert.pem` / `resolv.conf` | `app_mojo/src/main/assets/` | `assets/config/` |
| Shaders | Extract from `shaderc.aar` / `spvc.aar` | `assets/shaders/` |
| Authlib injector | `assets/components/authlib-injector/` | `assets/components/` |
| Security patches | `assets/components/security/` | `assets/components/` |
| Caciocavallo | `assets/components/caciocavallo/` | `assets/components/` |

---

## License

Original: GPL-3.0 (MojoLauncher/PojavLauncher)
Port: **GPL-3.0** — must maintain license compatibility

---

## References

- MojoLauncher: https://github.com/MojoLauncher/MojoLauncher (branch `v3_openjdk`)
- PojavLauncher: https://github.com/PojavLauncherTeam/PojavLauncher
- Mojang Version Manifest: https://launchermeta.mojang.com/mc/game/version_manifest.json
- Microsoft Auth: https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code
- ElyBy API: https://ely.by/api/doc
- Forge Installer: https://github.com/MinecraftForge/ForgeInstaller
- Fabric Loader: https://github.com/FabricMC/fabric-loader
- Quilt Loader: https://github.com/QuiltMC/quilt-loader
- NeoForge: https://github.com/neoforged/neoforge
- OptiFine: https://optifine.net/
- gl4es: https://github.com/ptitSeb/gl4es
- mojoexec: (submodule in MojoLauncher)