//! Forge Installer UI — port of ForgeInstallFragment.java

use eframe::egui;
use larp_jv_core::modding::{ModManager, ModLoaderType};
use std::sync::Arc;

pub struct ForgeInstallUi {
    mod_manager: Arc<crate::modding::ModManager>,
    mc_version: String,
    forge_version: String,
    installing: bool,
    progress: f32,
    error: Option<String>,
}

impl ForgeInstallUi {
    pub fn new(mod_manager: Arc<crate::modding::ModManager>) -> Self {
        Self {
            mod_manager,
            mc_version: String::new(),
            forge_version: String::new(),
            installing: false,
            progress: 0.0,
            error: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, on_install: impl FnOnce()) {
        egui::Window::new("Install Forge")
            .resizable(false)
            .min_width(450.0)
            .show(ctx, |ui| {
                ui.heading("Install Forge");
                ui.separator();
                ui.add_space(8.0);

                // Minecraft version
                ui.horizontal(|ui| {
                    ui.label("MC Version: ");
                    ui.text_edit_singleline(&mut self.mc_version);
                    if ui.button("Fetch").clicked() {
                        // Fetch Forge versions for this MC version
                    }
                });
                ui.add_space(8.0);

                // Forge version
                ui.horizontal(|ui| {
                    ui.label("Forge Version: ");
                    ui.text_edit_singleline(&mut self.forge_version);
                });
                ui.add_space(16.0);

                if let Some(err) = &self.error {
                    ui.colored_label(egui::Color32::RED, err);
                    ui.add_space(8.0);
                }

                if self.installing {
                    ui.add(egui::ProgressBar::new(self.progress).show_percentage());
                    ui.label("Installing Forge...");
                    ui.add_space(8.0);
                    if ui.button("Cancel").clicked() {
                        // Cancel
                    }
                } else {
                    ui.horizontal(|ui| {
                        let install_btn = ui.add_enabled(
                            !self.mc_version.is_empty() && !self.forge_version.is_empty(),
                            egui::Button::new("Install Forge").fill(egui::Color32::from_rgb(0, 150, 0))
                        );
                        if install_btn.clicked() {
                            // Start install
                        }
                        if ui.button("Cancel").clicked() {
                            // Close
                        }
                    });
                ui.add_space(8.0);
                
                // Latest versions info
                ui.separator();
                ui.label("Tip: Click 'Fetch' to get available Forge versions for your Minecraft version");
            });
    }
}