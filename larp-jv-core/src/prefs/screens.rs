//! Preference Screens — Port of LauncherPreferenceFragment, ControlFragment, VideoFragment, JavaFragment, ExperimentalFragment, MiscellaneousFragment

use eframe::egui;
use crate::value::Instance;

/// Base Preference Fragment Trait
pub trait PreferenceFragment {
    fn title(&self) -> &str;
    fn update(&mut self, ui: &mut egui::Ui, instance: &mut Instance);
    fn on_save(&mut self, instance: &mut Instance) -> crate::error::LarpResult<()>;
    fn on_reset(&mut self, instance: &mut Instance);
}

/// Control Preferences — Port of LauncherPreferenceControlFragment.java
pub struct ControlPreferenceFragment {
    key_bindings: std::collections::HashMap<String, KeyBinding>,
    selected_category: Option<String>,
    editing: Option<String>,
}

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub name: String,
    pub category: String,
    pub primary: Option<egui::Key>,
    pub secondary: Option<egui::Key>,
    pub description: String,
}

impl ControlPreferenceFragment {
    pub fn new() -> Self {
        let mut key_bindings = std::collections::HashMap::new();
        
        // Default Minecraft keybindings
        let defaults = vec![
            ("forward", "Forward", "Movement", egui::Key::W, "Move forward"),
            ("back", "Back", "Movement", egui::Key::S, "Move backward"),
            ("left", "Left", "Movement", egui::Key::A, "Strafe left"),
            ("right", "Right", "Movement", egui::Key::D, "Strafe right"),
            ("jump", "Jump", "Movement", egui::Key::Space, "Jump"),
            ("sprint", "Sprint", "Movement", egui::Key::LeftCtrl, "Sprint"),
            ("sneak", "Sneak", "Movement", egui::Key::LeftShift, "Sneak"),
            ("inventory", "Inventory", "Inventory", egui::Key::E, "Open inventory"),
            ("drop", "Drop", "Inventory", egui::Key::Q, "Drop item"),
            ("swap_hands", "Swap Hands", "Inventory", egui::Key::F, "Swap main/off hand"),
            ("chat", "Chat", "Multiplayer", egui::Key::T, "Open chat"),
            ("list_players", "Player List", "Multiplayer", egui::Key::Tab, "Show player list"),
            ("screenshot", "Screenshot", "Misc", egui::Key::F2, "Take screenshot"),
            ("toggle_perspective", "Toggle Perspective", "Misc", egui::Key::F5, "Toggle perspective"),
            ("command", "Command", "Multiplayer", egui::Key::Slash, "Open command"),
        ];
        
        for (id, name, category, key, desc) in defaults {
            key_bindings.insert(id.to_string(), KeyBinding {
                name: name.to_string(),
                category: category.to_string(),
                primary: Some(key),
                secondary: None,
                description: desc.to_string(),
            });
        }
        
        // Hotbar 1-9
        for i in 1..=9 {
            key_bindings.insert(format!("hotbar.{}", i), KeyBinding {
                name: format!("Hotbar Slot {}", i),
                category: "Hotbar".to_string(),
                primary: Some(match i {
                    1 => egui::Key::Num1,
                    2 => egui::Key::Num2,
                    3 => egui::Key::Num3,
                    4 => egui::Key::Num4,
                    5 => egui::Key::Num5,
                    6 => egui::Key::Num6,
                    7 => egui::Key::Num7,
                    8 => egui::Key::Num8,
                    9 => egui::Key::Num9,
                    _ => egui::Key::Num1,
                }),
                secondary: None,
                description: format!("Select hotbar slot {}", i),
            });
        }
        
        Self {
            key_bindings,
            selected_category: Some("Movement".to_string()),
            editing: None,
        }
    }
}

impl Default for ControlPreferenceFragment {
    fn default() -> Self {
        Self::new()
    }
}

impl PreferenceFragment for ControlPreferenceFragment {
    fn title(&self) -> &str {
        "Controls"
    }

    fn update(&mut self, ui: &mut egui::Ui, _instance: &mut Instance) {
        // Category selector
        let categories: Vec<String> = self.key_bindings.values()
            .map(|b| b.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        ui.horizontal(|ui| {
            ui.label("Category: ");
            egui::ComboBox::from_id_source("control_category")
                .selected_text(self.selected_category.as_deref().unwrap_or("All"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_category, None, "All");
                    for cat in &categories {
                        ui.selectable_value(&mut self.selected_category, Some(cat.clone()), cat);
                    }
                });
        });
        ui.separator();
        
        // Keybindings list
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (id, binding) in &mut self.key_bindings {
                if let Some(cat) = &self.selected_category {
                    if &binding.category != cat { continue; }
                }
                
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.strong(&binding.name);
                            ui.small(&binding.description);
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let key_text = binding.primary.map(|k| format!("{:?}", k)).unwrap_or_else(|| "Unbound".to_string());
                            if ui.button(key_text).clicked() {
                                self.editing = Some(id.clone());
                            }
                            if ui.button("Reset").clicked() {
                                // Reset to default
                            }
                        });
                    });
                });
            }
        });
        
        // Editing overlay
        if let Some(editing_id) = &self.editing {
            if let Some(binding) = self.key_bindings.get_mut(editing_id) {
                egui::Area::new(egui::Id::new("keybinding_edit"))
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(ui.ctx(), |ui| {
                        egui::Frame::window(ui.style()).show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.heading("Press a key...");
                                ui.add_space(8.0);
                                ui.label(format!("Editing: {}", binding.name));
                                ui.add_space(8.0);
                                if ui.button("Cancel").clicked() {
                                    self.editing = None;
                                }
                            });
                        });
                    });
                
                ui.ctx().input(|i| {
                    for key in i.keys_down.iter() {
                        binding.primary = Some(*key);
                        self.editing = None;
                    }
                });
            }
        }
    }

    fn on_save(&mut self, _instance: &mut Instance) -> crate::error::LarpResult<()> {
        // Save keybindings to instance config
        Ok(())
    }

    fn on_reset(&mut self, _instance: &mut Instance) {
        *self = Self::new();
    }
}

/// Video Preferences — Port of LauncherPreferenceVideoFragment.java
pub struct VideoPreferenceFragment {
    render_distance: u8,
    fov: f32,
    vsync: bool,
    max_fps: u32,
    particles: String,
    clouds: String,
    graphics: String,
    smooth_lighting: String,
    entity_shadows: bool,
    fullscreen: bool,
    resolution: (u32, u32),
    use_custom_resolution: bool,
}

impl VideoPreferenceFragment {
    pub fn new() -> Self {
        Self {
            render_distance: 12,
            fov: 70.0,
            vsync: true,
            max_fps: 60,
            particles: "all".to_string(),
            clouds: "fancy".to_string(),
            graphics: "fancy".to_string(),
            smooth_lighting: "maximum".to_string(),
            entity_shadows: true,
            fullscreen: false,
            resolution: (1280, 720),
            use_custom_resolution: false,
        }
    }
}

impl Default for VideoPreferenceFragment {
    fn default() -> Self {
        Self::new()
    }
}

impl PreferenceFragment for VideoPreferenceFragment {
    fn title(&self) -> &str {
        "Video Settings"
    }

    fn update(&mut self, ui: &mut egui::Ui, _instance: &mut Instance) {
        ui.horizontal(|ui| {
            ui.label("Render Distance: ");
            ui.add(egui::DragValue::new(&mut self.render_distance).speed(0.5).range(2..=32).suffix(" chunks"));
        });
        ui.horizontal(|ui| {
            ui.label("FOV: ");
            ui.add(egui::Slider::new(&mut self.fov, 30.0..=110.0).suffix("°"));
        });
        ui.checkbox(&mut self.vsync, "VSync");
        ui.horizontal(|ui| {
            ui.label("Max FPS: ");
            ui.add(egui::DragValue::new(&mut self.max_fps).speed(10).range(10..=1000).suffix(" FPS"));
        });
        ui.add_space(8.0);
        
        ui.separator();
        ui.label("Quality");
        ui.add_space(4.0);
        
        ui.horizontal(|ui| {
            ui.label("Graphics: ");
            egui::ComboBox::from_id_source("graphics")
                .selected_text(&self.graphics)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.graphics, "fast".to_string(), "Fast");
                    ui.selectable_value(&mut self.graphics, "fancy".to_string(), "Fancy");
                });
        });
        ui.horizontal(|ui| {
            ui.label("Smooth Lighting: ");
            egui::ComboBox::from_id_source("smooth_lighting")
                .selected_text(&self.smooth_lighting)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.smooth_lighting, "off".to_string(), "Off");
                    ui.selectable_value(&mut self.smooth_lighting, "minimum".to_string(), "Minimum");
                    ui.selectable_value(&mut self.smooth_lighting, "maximum".to_string(), "Maximum");
                });
        });
        ui.horizontal(|ui| {
            ui.label("Particles: ");
            egui::ComboBox::from_id_source("particles")
                .selected_text(&self.particles)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.particles, "minimal".to_string(), "Minimal");
                    ui.selectable_value(&mut self.particles, "decreased".to_string(), "Decreased");
                    ui.selectable_value(&mut self.particles, "all".to_string(), "All");
                });
        });
        ui.horizontal(|ui| {
            ui.label("Clouds: ");
            egui::ComboBox::from_id_source("clouds")
                .selected_text(&self.clouds)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.clouds, "off".to_string(), "Off");
                    ui.selectable_value(&mut self.clouds, "fast".to_string(), "Fast");
                    ui.selectable_value(&mut self.clouds, "fancy".to_string(), "Fancy");
                });
        });
        ui.checkbox(&mut self.entity_shadows, "Entity Shadows");
        ui.add_space(8.0);
        
        ui.separator();
        ui.label("Display");
        ui.add_space(4.0);
        
        ui.checkbox(&mut self.fullscreen, "Fullscreen");
        if !self.fullscreen {
            ui.horizontal(|ui| {
                ui.label("Resolution: ");
                ui.add(egui::DragValue::new(&mut self.resolution.0).speed(1).range(320..=7680).suffix("w"));
                ui.label("x");
                ui.add(egui::DragValue::new(&mut self.resolution.1).speed(1).range(240..=4320).suffix("h"));
            });
        }
    }

    fn on_save(&mut self, instance: &mut Instance) -> crate::error::LarpResult<()> {
        instance.video_settings = Some(crate::value::VideoSettings {
            render_distance: self.render_distance,
            fov: self.fov,
            vsync: self.vsync,
            max_fps: self.max_fps,
            particles: self.particles.clone(),
            clouds: self.clouds.clone(),
            graphics: self.graphics.clone(),
            smooth_lighting: self.smooth_lighting.clone(),
            entity_shadows: self.entity_shadows,
            fullscreen: self.fullscreen,
            resolution: self.resolution,
            use_custom_resolution: self.use_custom_resolution,
        });
        Ok(())
    }

    fn on_reset(&mut self, _instance: &mut Instance) {
        *self = Self::new();
    }
}

/// Java Preferences — Port of LauncherPreferenceJavaFragment.java
pub struct JavaPreferenceFragment {
    java_path: String,
    jvm_args: String,
    max_memory: u32,
    min_memory: u32,
    auto_detect: bool,
    java_versions: Vec<JavaVersionInfo>,
    selected_java: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct JavaVersionInfo {
    pub version: String,
    pub path: String,
    pub vendor: String,
    pub architecture: String,
}

impl JavaPreferenceFragment {
    pub fn new() -> Self {
        Self {
            java_path: String::new(),
            jvm_args: "-XX:+UseG1GC -XX:+UnlockExperimentalVMOptions -XX:G1NewSizePercent=20 -XX:G1ReservePercent=20 -XX:MaxGCPauseMillis=50 -XX:G1HeapRegionSize=32M".to_string(),
            max_memory: 2048,
            min_memory: 512,
            auto_detect: true,
            java_versions: Vec::new(),
            selected_java: None,
        }
    }
}

impl Default for JavaPreferenceFragment {
    fn default() -> Self {
        Self::new()
    }
}

impl PreferenceFragment for JavaPreferenceFragment {
    fn title(&self) -> &str {
        "Java Settings"
    }

    fn update(&mut self, ui: &mut egui::Ui, _instance: &mut Instance) {
        ui.collapsing("Java Detection", |ui| {
            ui.checkbox(&mut self.auto_detect, "Auto-detect Java");
            if !self.auto_detect {
                ui.horizontal(|ui| {
                    ui.label("Java Path: ");
                    ui.text_edit_singleline(&mut self.java_path);
                    if ui.button("Browse").clicked() {
                        // File picker
                    }
                });
            }
            ui.add_space(4.0);
            if ui.button("Detect Java").clicked() {
                self.detect_java();
            }
        });
        
        if !self.java_versions.is_empty() {
            ui.collapsing("Detected Java Versions", |ui| {
                for (i, v) in self.java_versions.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let selected = self.selected_java == Some(i);
                        if ui.selectable_label(selected, format!("{} ({}) - {}", v.version, v.architecture, v.path)).clicked() {
                            self.selected_java = Some(i);
                            self.java_path = v.path.clone();
                        }
                    });
                }
            });
        }
        
        ui.collapsing("Memory", |ui| {
            ui.horizontal(|ui| {
                ui.label("Min: ");
                ui.add(egui::DragValue::new(&mut self.min_memory).suffix(" MB").speed(64).range(256..=self.max_memory));
            });
            ui.horizontal(|ui| {
                ui.label("Max: ");
                ui.add(egui::DragValue::new(&mut self.max_memory).suffix(" MB").speed(64).range(self.min_memory..=16384));
            });
        });
        
        ui.collapsing("JVM Arguments", |ui| {
            ui.text_edit_multiline(&mut self.jvm_args);
            ui.small("Default: -XX:+UseG1GC -XX:+UnlockExperimentalVMOptions -XX:G1NewSizePercent=20 -XX:G1ReservePercent=20 -XX:MaxGCPauseMillis=50 -XX:G1HeapRegionSize=32M");
        });
    }

    fn on_save(&mut self, instance: &mut Instance) -> crate::error::LarpResult<()> {
        instance.java_settings = Some(crate::value::JavaSettings {
            java_path: if self.auto_detect { None } else { Some(self.java_path.clone()) },
            jvm_args: self.jvm_args.clone(),
            max_memory: self.max_memory,
            min_memory: self.min_memory,
            auto_detect: self.auto_detect,
        });
        Ok(())
    }

    fn on_reset(&mut self, _instance: &mut Instance) {
        *self = Self::new();
    }
}

impl JavaPreferenceFragment {
    fn detect_java(&mut self) {
        // Call JreManager.detect()
        self.java_versions = vec![
            JavaVersionInfo { version: "21.0.1".to_string(), path: "/usr/lib/jvm/java-21-openjdk/bin/java".to_string(), vendor: "OpenJDK".to_string(), architecture: "aarch64".to_string() },
            JavaVersionInfo { version: "17.0.9".to_string(), path: "/usr/lib/jvm/java-17-openjdk/bin/java".to_string(), vendor: "OpenJDK".to_string(), architecture: "aarch64".to_string() },
            JavaVersionInfo { version: "8.0.382".to_string(), path: "/usr/lib/jvm/java-8-openjdk/bin/java".to_string(), vendor: "OpenJDK".to_string(), architecture: "aarch64".to_string() },
        ];
    }
}

/// Experimental Preferences — Port of LauncherPreferenceExperimentalFragment.java
pub struct ExperimentalPreferenceFragment {
    enable_gl_thread: bool,
    enable_multi_rt: bool,
    enable_vulkan: bool,
    enable_mesa: bool,
    enable_gles32: bool,
    force_gpu: Option<String>,
    debug_render: bool,
    show_fps: bool,
    dump_shaders: bool,
    enable_profiling: bool,
}

impl ExperimentalPreferenceFragment {
    pub fn new() -> Self {
        Self {
            enable_gl_thread: false,
            enable_multi_rt: false,
            enable_vulkan: true,
            enable_mesa: false,
            enable_gles32: false,
            force_gpu: None,
            debug_render: false,
            show_fps: false,
            dump_shaders: false,
            enable_profiling: false,
        }
    }
}

impl Default for ExperimentalPreferenceFragment {
    fn default() -> Self {
        Self::new()
    }
}

impl PreferenceFragment for ExperimentalPreferenceFragment {
    fn title(&self) -> &str {
        "Experimental"
    }

    fn update(&mut self, ui: &mut egui::Ui, _instance: &mut Instance) {
        ui.label(egui::RichText::new("⚠ Experimental features - may cause instability").color(egui::Color32::YELLOW));
        ui.separator();
        ui.add_space(8.0);
        
        ui.collapsing("Rendering", |ui| {
            ui.checkbox(&mut self.enable_vulkan, "Enable Vulkan");
            ui.checkbox(&mut self.enable_mesa, "Enable Mesa/Turnip");
            ui.checkbox(&mut self.enable_gles32, "Force GLES 3.2");
            ui.checkbox(&mut self.enable_gl_thread, "Enable GL Thread");
            ui.checkbox(&mut self.enable_multi_rt, "Enable MultiRT");
            
            ui.horizontal(|ui| {
                ui.label("Force GPU: ");
                egui::ComboBox::from_id_source("force_gpu")
                    .selected_text(self.force_gpu.as_deref().unwrap_or("Auto"))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.force_gpu, None, "Auto");
                        ui.selectable_value(&mut self.force_gpu, Some("adreno".to_string()), "Adreno");
                        ui.selectable_value(&mut self.force_gpu, Some("builtin".to_string()), "Builtin");
                        ui.selectable_value(&mut self.force_gpu, Some("display".to_string()), "Display");
                    });
            });
        });
        
        ui.collapsing("Debug", |ui| {
            ui.checkbox(&mut self.debug_render, "Debug Render");
            ui.checkbox(&mut self.show_fps, "Show FPS");
            ui.checkbox(&mut self.dump_shaders, "Dump Shaders");
            ui.checkbox(&mut self.enable_profiling, "Enable Profiling");
        });
        
        ui.add_space(16.0);
        ui.label("Changes require restart");
    }

    fn on_save(&mut self, instance: &mut Instance) -> crate::error::LarpResult<()> {
        instance.experimental_settings = Some(crate::value::ExperimentalSettings {
            enable_gl_thread: self.enable_gl_thread,
            enable_multi_rt: self.enable_multi_rt,
            enable_vulkan: self.enable_vulkan,
            enable_mesa: self.enable_mesa,
            enable_gles32: self.enable_gles32,
            force_gpu: self.force_gpu.clone(),
            debug_render: self.debug_render,
            show_fps: self.show_fps,
            dump_shaders: self.dump_shaders,
            enable_profiling: self.enable_profiling,
        });
        Ok(())
    }

    fn on_reset(&mut self, _instance: &mut Instance) {
        *self = Self::new();
    }
}

/// Miscellaneous Preferences — Port of LauncherPreferenceMiscellaneousFragment.java
pub struct MiscellaneousPreferenceFragment {
    language: String,
    theme: String,
    auto_update: bool,
    show_snapshots: bool,
    enable_analytics: bool,
    custom_skin_url: String,
    custom_cape_url: String,
    backup_instances: bool,
    max_log_size: u32,
}

impl MiscellaneousPreferenceFragment {
    pub fn new() -> Self {
        Self {
            language: "en_US".to_string(),
            theme: "dark".to_string(),
            auto_update: true,
            show_snapshots: false,
            enable_analytics: false,
            custom_skin_url: String::new(),
            custom_cape_url: String::new(),
            backup_instances: true,
            max_log_size: 10,
        }
    }
}

impl Default for MiscellaneousPreferenceFragment {
    fn default() -> Self {
        Self::new()
    }
}

impl PreferenceFragment for MiscellaneousPreferenceFragment {
    fn title(&self) -> &str {
        "Miscellaneous"
    }

    fn update(&mut self, ui: &mut egui::Ui, _instance: &mut Instance) {
        ui.collapsing("Appearance", |ui| {
            ui.horizontal(|ui| {
                ui.label("Language: ");
                egui::ComboBox::from_id_source("language")
                    .selected_text(&self.language)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.language, "en_US".to_string(), "English");
                        ui.selectable_value(&mut self.language, "pt_BR".to_string(), "Português (Brasil)");
                        ui.selectable_value(&mut self.language, "es_ES".to_string(), "Español");
                        ui.selectable_value(&mut self.language, "fr_FR".to_string(), "Français");
                        ui.selectable_value(&mut self.language, "de_DE".to_string(), "Deutsch");
                        ui.selectable_value(&mut self.language, "zh_CN".to_string(), "中文 (简体)");
                        ui.selectable_value(&mut self.language, "ja_JP".to_string(), "日本語");
                        ui.selectable_value(&mut self.language, "ko_KR".to_string(), "한국어");
                        ui.selectable_value(&mut self.language, "ru_RU".to_string(), "Русский");
                    });
            });
            ui.horizontal(|ui| {
                ui.label("Theme: ");
                egui::ComboBox::from_id_source("theme")
                    .selected_text(&self.theme)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.theme, "dark".to_string(), "Dark");
                        ui.selectable_value(&mut self.theme, "light".to_string(), "Light");
                        ui.selectable_value(&mut self.theme, "system".to_string(), "System");
                    });
            });
        });
        
        ui.collapsing("Behavior", |ui| {
            ui.checkbox(&mut self.auto_update, "Auto-update launcher");
            ui.checkbox(&mut self.show_snapshots, "Show snapshot versions");
            ui.checkbox(&mut self.enable_analytics, "Enable analytics");
            ui.checkbox(&mut self.backup_instances, "Backup instances on modify");
            ui.horizontal(|ui| {
                ui.label("Max log size (MB): ");
                ui.add(egui::DragValue::new(&mut self.max_log_size).speed(1).range(1..=100));
            });
        });
        
        ui.collapsing("Custom Skin/Cape", |ui| {
            ui.horizontal(|ui| {
                ui.label("Skin URL: ");
                ui.text_edit_singleline(&mut self.custom_skin_url);
            });
            ui.horizontal(|ui| {
                ui.label("Cape URL: ");
                ui.text_edit_singleline(&mut self.custom_cape_url);
            });
            ui.small("Leave empty to use default");
        });
        
        ui.collapsing("Danger Zone", |ui| {
            ui.label(egui::RichText::new("⚠ These actions cannot be undone").color(egui::Color32::RED));
            ui.add_space(4.0);
            if ui.button(egui::RichText::new("Clear Cache").color(egui::Color32::RED)).clicked() {
                // Clear cache
            }
            if ui.button(egui::RichText::new("Reset All Settings").color(egui::Color32::RED)).clicked() {
                // Reset settings
            }
            if ui.button(egui::RichText::new("Delete All Instances").color(egui::Color32::RED)).clicked() {
                // Delete instances
            }
        });
    }

    fn on_save(&mut self, instance: &mut Instance) -> crate::error::LarpResult<()> {
        instance.misc_settings = Some(crate::value::MiscSettings {
            language: self.language.clone(),
            theme: self.theme.clone(),
            auto_update: self.auto_update,
            show_snapshots: self.show_snapshots,
            enable_analytics: self.enable_analytics,
            custom_skin_url: self.custom_skin_url.clone(),
            custom_cape_url: self.custom_cape_url.clone(),
            backup_instances: self.backup_instances,
            max_log_size: self.max_log_size,
        });
        Ok(())
    }

    fn on_reset(&mut self, _instance: &mut Instance) {
        *self = Self::new();
    }
}