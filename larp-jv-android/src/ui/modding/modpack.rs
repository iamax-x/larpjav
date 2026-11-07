//! Modpack (MRPACK) Installer UI

use eframe::egui;
use std::sync::Arc;
use std::path::PathBuf;

pub struct ModpackInstallUi {
    mrpack_path: Option<PathBuf>,
    install_dir: PathBuf,
    installing: bool,
    progress: f32,
    status: String,
    error: Option<String>,
}

impl ModpackInstallUi {
    pub fn new(_mod_manager: Arc<crate::modding::ModManager>) -> Self {
        Self {
            mrpack_path: None,
            install_dir: PathBuf::from("/sdcard/games/com.mojang/minecraftInstances"),
            installing: false,
            progress: 0.0,
            status: String::new(),
            error: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Install Modpack (.mrpack)")
            .resizable(false)
            .min_width(500.0)
            .show(ctx, |ui| {
                ui.heading("Install Modpack (.mrpack)");
                ui.separator();
                ui.add_space(8.0);

                // MRPACK file selection
                ui.horizontal(|ui| {
                    ui.label("MRPACK File: ");
                    if let Some(path) = &self.mrpack_path {
                        ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                    } else {
                        ui.label("(none selected)");
                    }
                    if ui.button("Browse").clicked() {
                        // Open file picker
                    }
                });
                ui.add_space(8.0);

                // Install directory
                ui.horizontal(|ui| {
                    ui.label("Install Dir: ");
                    ui.text_edit_singleline(&mut self.install_dir.to_string_lossy().to_string());
                    if ui.button("Browse").clicked() {
                        // Open folder picker
                    }
                });
                ui.add_space(16.0);

                if let Some(err) = &self.error {
                    ui.colored_label(egui::Color32::RED, err);
                    ui.add_space(8.0);
                }

                if self.installing {
                    ui.add(egui::ProgressBar::new(self.progress).show_percentage());
                    ui.label(&self.status);
                    ui.add_space(8.0);
                    if ui.button("Cancel").clicked() {
                        // Cancel
                    }
                } else {
                    ui.horizontal(|ui| {
                        let install_btn = ui.add_enabled(
                            self.mrpack_path.is_some(),
                            egui::Button::new("Install Modpack").fill(egui::Color32::from_rgb(0, 150, 0))
                        );
                        if install_btn.clicked() {
                            // Start install
                        }
                        if ui.button("Cancel").clicked() {
                            // Close
                        }
                    });
                }
                
                ui.add_space(8.0);
                ui.separator();
                ui.label("Supports MRPACK format (Modrinth, CurseForge, ATLauncher)");
            });
    }
}