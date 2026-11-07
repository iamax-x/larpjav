# larp.jv — Master Prompt: All Remaining Work (Prompts 3-8)

> **Objetivo**: Completar 100% do port do MojoLauncher → larp.jv
> **Base**: Prompt 1 (Plataforma) + Prompt 2 (UI Core) já commitados
> **Alvo**: 100% funcional, pronto para `cargo apk build --release`

---

## Prompt 3 — Modding Completo (Todos Loaders)

### Arquivos a Criar: `src/ui/modding/*.rs`

| Loader | Arquivo | Funcionalidades |
|--------|---------|-----------------|
| Quilt | `quilt.rs` | Loader version, MC version, install |
| NeoForge | `neoforge.rs` | MC version + NeoForge version, install |
| OptiFine | `optifine.rs` | Scraper de versões, patcher, install |
| Legacy Fabric | `legacy_fabric.rs` | Versões 0.x para MC antigo |
| BTA | `bta.rs` | GitHub releases, install |
| Modpack | `modpack.rs` | MRPACK parser, install, dependency resolution |
| Search | `search.rs` | Modrinth + CurseForge API, filtros |
| Browser | `browser.rs` | Lista paginada, detalhes, install |
| Installer | `installer.rs` | Unified installer trait, progress callbacks |

### Backend Modding (já existe no core, completar UI)

```rust
// Trait unificado para todos loaders
trait ModLoaderInstaller {
    async fn fetch_versions(&self, mc_version: &str) -> LarpResult<Vec<String>>;
    async fn install(&self, mc_version: &str, loader_version: &str, dest: &Path) -> LarpResult<InstalledModLoader>;
    fn loader_type(&self) -> ModLoaderType;
}
```

---

## Prompt 4 — Settings Completos

### Arquivos: `src/ui/settings/*.rs`

| Arquivo | Fragment Original | Conteúdo |
|---------|-------------------|----------|
| `control.rs` | `LauncherPreferenceControlFragment` | Gamepad mapping, touch controls, sensitivity |
| `experimental.rs` | `LauncherPreferenceExperimentalFragment` | Debug logging, custom renderer, force GL |
| `java.rs` | `LauncherPreferenceJavaFragment` | Java path, JVM args, JRE downloader, version selector |
| `misc.rs` | `LauncherPreferenceMiscellaneousFragment` | Language, cache, logs, FPS, console |
| `video.rs` | `LauncherPreferenceVideoFragment` | Resolution, fullscreen, VSync, render distance, shaders |

---

## Prompt 5 — Input Avançado

### Arquivos: `src/ui/input/*.rs`

| Arquivo | Fragment Original | Conteúdo |
|---------|-------------------|----------|
| `gamepad.rs` | `GamepadMapperFragment` | Mapeamento visual, deadzone, vibração |
| `mouse.rs` | `Mouse/*.java` | Virtual mouse, gestos, scroll, click |
| `keyboard.rs` | `Keyboard/*.java` | Teclado virtual, char input |
| `custom_controls.rs` | `CustomControlsActivity` | Editor visual de touch controls |

---

## Prompt 6 — Game Rendering (Core do Jogo)

### Arquivos: `src/render/*.rs`

| Arquivo | Java Original | Conteúdo |
|---------|---------------|----------|
| `renderer.rs` | `GameRenderer.java` | Pipeline principal, frame loop |
| `gles.rs` | `GLESRenderSpec.java` | OpenGL ES 3.0 backend |
| `mesa.rs` | `MesaRenderSpec.java` | Mesa fallback |
| `platform.rs` | `Platform.java` | Abstração AWT/GLFW/SDL/Dummy |
| `surface.rs` | `SurfaceProvider.java` | SurfaceView/TextureView handling |
| `backend/awt.rs` | `AWTBackend.java` | AWT backend |
| `backend/glfw.rs` | `GLFWBackend.java` | GLFW backend |
| `backend/sdl.rs` | `SDLBackend.java` | SDL backend |
| `input.rs` | `InGameEventProcessor.java` | Processamento input in-game |
| `cursor.rs` | `GameCursorView.java` | Custom cursor rendering |

---

## Prompt 7 — Native Libraries Completos

### `larp-jv-native/` (crate separado)

| Componente | Fonte C Original | Rust Target |
|------------|------------------|-------------|
| `mojoexec` | `jre_launcher/*.c` | Pure Rust JVM launcher |
| `jvm_hooks` | `jvm_hooks/*.c` | LD_PRELOAD hooks |
| `unpack200` | `libunpack200.so` | Pack200 unpacker |
| `jnidispatch` | `libjnidispatch.so` | JNA replacement |
| `gl4es` | `libgl4es_114.so` | Bind system lib |
| `freetype` | `libfreetype.so` | Bind system lib |

---

## Prompt 8 — Desktop Frontend + CI/CD

### `larp-jv-desktop/` (crate separado)

| Componente | Descrição |
|------------|-----------|
| `src/main.rs` | Entry point winit + egui/wgpu |
| `src/platform/` | rfd, arboard, notify-rust, tray-icon |
| `src/ui/` | Mesmo UI do Android (reutilizar lógica) |
| `src/render/` | wgpu backends Vulkan/Metal/DX12/GL |

### CI/CD: `.github/workflows/`

| Workflow | Gatilho | Ações |
|----------|---------|-------|
| `ci.yml` | push/PR | test, clippy, fmt, build desktop (Linux/Win/Mac) |
| `android.yml` | push/PR | cargo-apk build, device test |
| `release.yml` | tag `v*` | cargo-dist, cargo-apk, artifacts upload |

---

## Ordem de Execução Recomendada

```bash
# 1. Modding UI completo (Prompt 3)
# 2. Settings UI completo (Prompt 4)
# 3. Input avançado (Prompt 5)
# 4. Game Rendering (Prompt 6)
# 5. Native libs (Prompt 7)
# 6. Desktop + CI/CD (Prompt 8)

# Cada prompt = commit separado + push
```

---

## Critérios de Aceite Finais

```bash
# 1. Build Android
cargo apk build --release -p larp-jv-android
# → APK instalável, roda no device, loga "wgpu OK"

# 2. Build Desktop
cargo build --release -p larp-jv-desktop
# → Binário nativo Linux/Win/Mac

# 3. Testes
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check

# 4. Release
git tag v0.1.0 && git push --tags
# → GitHub Actions publica APK + binários desktop
```

---

**Total estimado**: ~150 arquivos Rust + Kotlin + XML restantes
**Estratégia**: 1 prompt = 1 commit = push → testar no device → próximo prompt