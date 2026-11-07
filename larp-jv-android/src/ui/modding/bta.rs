//! BTA (Better Than Adventure) Installer UI — port of BTAInstallFragment.java

use eframe::egui;
use std::sync::Arc;

pub struct BTAInstallUi {
    version: String,
    versions: Vec<BTAVersion>,
    installing: bool,
    progress: f32,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BTAVersion {
    pub version: String,
    pub url: String,
    pub mc_version: String,
}

impl BTAInstallUi {
    pub fn new(_mod_manager: Arc<crate::modding::ModManager>) -> Self {
        Self {
            version: String::new(),
            versions: Vec::new(),
            installing: false,
            progress: 0.0,
            error: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Install Better Than Adventure")
            .resizable(false)
            .min_width(450.0)
            .show(ctx, |ui| {
                ui.heading("Install Better Than Adventure");
                ui.separator();
                ui.add_space(8.0);

                ui.label("BTA versions from GitHub releases");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Version: ");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.version)
                        .show_ui(ui, |ui| {
                            for v in &self.versions {
                                ui.selectable_value(&mut self.version, v.version.clone(), &v.version);
                            }
                        });
                    if ui.button("Fetch Releases").clicked() {
                        // Fetch from GitHub API
                    }
                });
                ui.add_space(16.0);

                if let Some(err) = &self.error {
                    ui.colored_label(egui::Color32::RED, err);
                    ui.add_space(8.0);
                }

                if self.installing {
                    ui.add(egui::ProgressBar::new(self.progress).show_percentage());
                    ui.label("Installing BTA...");
                    ui.add_space(8.0);
                    if ui.button("Cancel").clicked() {
                        // Cancel
                    }
                } else {
                    ui.horizontal(|ui| {
                        let install_btn = ui.add_enabled(
                            !self.version.is_empty(),
                            egui::Button::new("Install BTA").fill(egui::Color32::from_rgb(0, 150, 0))
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
                ui.label("Source: https://github.com/Better-Than-Adventure/Better-Than-Adventure");
            });
    }
}