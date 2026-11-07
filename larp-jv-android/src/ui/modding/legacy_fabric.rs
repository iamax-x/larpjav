//! Legacy Fabric Installer UI — port of LegacyFabricInstallFragment.java

use eframe::egui;
use std::sync::Arc;

pub struct LegacyFabricInstallUi {
    mc_version: String,
    loader_version: String,
    installing: bool,
    progress: f32,
    error: Option<String>,
}

impl LegacyFabricInstallUi {
    pub fn new(_mod_manager: Arc<crate::modding::ModManager>) -> Self {
        Self {
            mc_version: String::new(),
            loader_version: String::new(),
            installing: false,
            progress: 0.0,
            error: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Install Legacy Fabric")
            .resizable(false)
            .min_width(450.0)
            .show(ctx, |ui| {
                ui.heading("Install Legacy Fabric");
                ui.separator();
                ui.add_space(8.0);

                ui.label("For Minecraft versions before 1.14");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("MC Version: ");
                    ui.text_edit_singleline(&mut self.mc_version);
                    if ui.button("Fetch").clicked() {
                        // Fetch Legacy Fabric versions
                    }
                });
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Loader Version: ");
                    ui.text_edit_singleline(&mut self.loader_version);
                });
                ui.add_space(16.0);

                if let Some(err) = &self.error {
                    ui.colored_label(egui::Color32::RED, err);
                    ui.add_space(8.0);
                }

                if self.installing {
                    ui.add(egui::ProgressBar::new(self.progress).show_percentage());
                    ui.label("Installing Legacy Fabric...");
                    ui.add_space(8.0);
                    if ui.button("Cancel").clicked() {
                        // Cancel
                    }
                } else {
                    ui.horizontal(|ui| {
                        let install_btn = ui.add_enabled(
                            !self.mc_version.is_empty() && !self.loader_version.is_empty(),
                            egui::Button::new("Install Legacy Fabric").fill(egui::Color32::from_rgb(0, 150, 0))
                        );
                        if install_btn.clicked() {
                            // Start install
                        }
                        if ui.button("Cancel").clicked() {
                            // Close
                        }
                    });
                }
            });
    }
}