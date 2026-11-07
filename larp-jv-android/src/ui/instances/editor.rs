//! Instance Editor Fragment — port of InstanceEditorFragment.java

use eframe::egui;
use larp_jv_core::{Instance, InstanceManager, VersionManifest};
use std::sync::Arc;
use std::path::PathBuf;

pub struct InstanceEditorUi {
    instance_manager: Arc<crate::instances::InstanceManager>,
    editing_instance: Option<Instance>,
    name: String,
    version_id: String,
    game_dir: PathBuf,
    java_path: Option<PathBuf>,
    jvm_args: String,
    game_args: String,
    account_id: Option<String>,
    custom_resolution: Option<(u32, u32)>,
    show_version_selector: bool,
    show_dir_picker: bool,
    show_java_picker: bool,
    available_versions: Vec<String>,
    error_message: Option<String>,
}

impl InstanceEditorUi {
    pub fn new(instance_manager: Arc<crate::instances::InstanceManager>) -> Self {
        Self {
            instance_manager,
            editing_instance: None,
            name: String::new(),
            version_id: String::new(),
            game_dir: PathBuf::from("/sdcard/games/com.mojang/minecraftInstances"),
            java_path: None,
            jvm_args: "-Xmx2G -Xms1G".to_string(),
            game_args: String::new(),
            account_id: None,
            custom_resolution: None,
            show_version_selector: false,
            show_dir_picker: false,
            show_java_picker: false,
            available_versions: Vec::new(),
            error_message: None,
        }
    }

    pub fn edit_instance(&mut self, instance: Instance) {
        self.name = instance.name.clone();
        self.version_id = instance.version_id.clone();
        self.game_dir = PathBuf::from(&instance.game_dir);
        self.java_path = instance.java_path.clone().map(PathBuf::from);
        self.jvm_args = instance.jvm_args.join(" ");
        self.game_args = instance.game_args.join(" ");
        self.account_id = instance.account_id.clone();
        self.custom_resolution = instance.custom_resolution.map(|r| (r.width, r.height));
        self.editing_instance = Some(instance);
    }

    pub fn new_instance(&mut self) {
        self.name.clear();
        self.version_id.clear();
        self.game_dir = PathBuf::from("/sdcard/games/com.mojang/minecraftInstances");
        self.java_path = None;
        self.jvm_args = "-Xmx2G -Xms1G".to_string();
        self.game_args.clear();
        self.account_id = None;
        self.custom_resolution = None;
        self.editing_instance = None;
        self.error_message = None;
    }

    pub fn update(&mut self, ctx: &egui::Context, on_save: impl FnOnce(Instance)) {
        egui::Window::new(if self.editing_instance.is_some() { "Edit Instance" } else { "Create Instance" })
            .resizable(true)
            .min_width(500.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading(if self.editing_instance.is_some() { "Edit Instance" } else { "Create Instance" });
                    ui.separator();
                    ui.add_space(8.0);

                    // Name
                    ui.horizontal(|ui| {
                        ui.label("Name: ");
                        ui.text_edit_singleline(&mut self.name);
                    });
                    ui.add_space(8.0);

                    // Version
                    ui.horizontal(|ui| {
                        ui.label("Version: ");
                        ui.text_edit_singleline(&mut self.version_id);
                        if ui.button("Select").clicked() {
                            self.show_version_selector = true;
                        }
                    });
                    ui.add_space(8.0);

                    // Game Directory
                    ui.horizontal(|ui| {
                        ui.label("Game Dir: ");
                        ui.text_edit_singleline(&mut self.game_dir.to_string_lossy().to_string());
                        if ui.button("Browse").clicked() {
                            self.show_dir_picker = true;
                        }
                    });
                    ui.add_space(8.0);

                    // Java Path
                    ui.horizontal(|ui| {
                        ui.label("Java Path: ");
                        if let Some(java) = &self.java_path {
                            ui.text_edit_singleline(&mut java.to_string_lossy().to_string());
                        } else {
                            ui.label("(auto-detect)");
                        }
                        if ui.button("Browse").clicked() {
                            self.show_java_picker = true;
                        }
                        if ui.button("Auto").clicked() {
                            self.java_path = None;
                        }
                    });
                    ui.add_space(8.0);

                    // JVM Arguments
                    ui.horizontal(|ui| {
                        ui.label("JVM Args: ");
                        ui.text_edit_singleline(&mut self.jvm_args);
                    });
                    ui.add_space(8.0);

                    // Game Arguments
                    ui.horizontal(|ui| {
                        ui.label("Game Args: ");
                        ui.text_edit_singleline(&mut self.game_args);
                    });
                    ui.add_space(8.0);

                    // Account
                    ui.horizontal(|ui| {
                        ui.label("Account: ");
                        if let Some(account_id) = &self.account_id {
                            ui.label(account_id);
                            if ui.button("Change").clicked() {
                                // Navigate to account selector
                            }
                        } else {
                            ui.label("(default)");
                        }
                    });
                    ui.add_space(8.0);

                    // Custom Resolution
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.custom_resolution.is_some(), "Custom Resolution");
                        if let Some((w, h)) = self.custom_resolution {
                            ui.label(format!("{}x{}", w, h));
                            if ui.button("Set").clicked() {
                                // Show resolution picker
                            }
                        }
                    });
                    ui.add_space(8.0);

                    // Error message
                    if let Some(err) = &self.error_message {
                        ui.colored_label(egui::Color32::RED, err);
                        ui.add_space(8.0);
                    }

                    // Buttons
                    ui.separator();
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            // Close window
                        }
                        if ui.add(egui::Button::new(if self.editing_instance.is_some() { "Save Changes" } else { "Create Instance" }).fill(egui::Color32::from_rgb(0, 150, 0))).clicked() {
                            if let Err(e) = self.validate_and_save() {
                                self.error_message = Some(e);
                            } else {
                                on_save(self.build_instance());
                            }
                        }
                    });
                });
            });

        // Version selector modal
        if self.show_version_selector {
            self.show_version_selector_modal(ctx);
        }
    }

    fn validate_and_save(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.version_id.trim().is_empty() {
            return Err("Version cannot be empty".to_string());
        }
        if !self.game_dir.exists() {
            return Err("Game directory does not exist".to_string());
        }
        Ok(())
    }

    fn build_instance(&self) -> Instance {
        Instance {
            id: self.editing_instance.as_ref().map(|i| i.id.clone()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: self.name.clone(),
            version_id: self.version_id.clone(),
            game_dir: self.game_dir.to_string_lossy().to_string(),
            java_path: self.java_path.as_ref().map(|p| p.to_string_lossy().to_string()),
            jvm_args: self.jvm_args.split_whitespace().map(|s| s.to_string()).collect(),
            game_args: self.game_args.split_whitespace().map(|s| s.to_string()).collect(),
            account_id: self.account_id.clone(),
            icon: None,
            created_at: chrono::Utc::now(),
            last_played: None,
            play_time: 0,
            custom_resolution: self.custom_resolution.map(|(w, h)| crate::value::Resolution { width: w, height: h }),
            mods: Vec::new(),
            resource_packs: Vec::new(),
            shader_pack: None,
            environment: std::collections::HashMap::new(),
        }
    }

    fn show_version_selector_modal(&mut self, ctx: &egui::Context) {
        egui::Window::new("Select Version")
            .resizable(true)
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Select Minecraft Version");
                ui.separator();
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for version in &self.available_versions {
                        if ui.selectable_label(self.version_id == *version, version).clicked() {
                            self.version_id = version.clone();
                            self.show_version_selector = false;
                        }
                    }
                });
                
                ui.separator();
                if ui.button("Cancel").clicked() {
                    self.show_version_selector = false;
                }
            });
    }
}