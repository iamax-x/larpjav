//! OptiFine Installer UI — port of OptiFineInstallFragment.java

use eframe::egui;
use std::sync::Arc;

pub struct OptiFineInstallUi {
    mc_version: String,
    optifine_version: String,
    installing: bool,
    progress: f32,
    error: Option<String>,
    versions: Vec<OptiFineVersion>,
}

#[derive(Debug, Clone)]
pub struct OptiFineVersion {
    pub version: String,
    pub mc_version: String,
    pub url: String,
    pub patch: String,
    pub preview: bool,
}

impl OptiFineInstallUi {
    pub fn new(_mod_manager: Arc<crate::modding::ModManager>) -> Self {
        Self {
            mc_version: String::new(),
            optifine_version: String::new(),
            installing: false,
            progress: 0.0,
            error: None,
            versions: Vec::new(),
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Install OptiFine")
            .resizable(false)
            .min_width(500.0)
            .show(ctx, |ui| {
                ui.heading("Install OptiFine");
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("MC Version: ");
                    ui.text_edit_singleline(&mut self.mc_version);
                    if ui.button("Fetch Versions").clicked() {
                        // Scrape OptiFine website for versions
                    }
                });
                ui.add_space(8.0);

                if !self.versions.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("OptiFine Version: ");
                        egui::ComboBox::from_label("")
                            .selected_text(&self.optifine_version)
                            .show_ui(ui, |ui| {
                                for v in &self.versions {
                                    let label = if v.preview {
                                        format!("{} (Preview)", v.version)
                                    } else {
                                        v.version.clone()
                                    };
                                    ui.selectable_value(&mut self.optifine_version, v.version.clone(), label);
                                }
                            });
                    });
                    ui.add_space(16.0);
                }

                if let Some(err) = &self.error {
                    ui.colored_label(egui::Color32::RED, err);
                    ui.add_space(8.0);
                }

                if self.installing {
                    ui.add(egui::ProgressBar::new(self.progress).show_percentage());
                    ui.label("Installing OptiFine...");
                    ui.add_space(8.0);
                    if ui.button("Cancel").clicked() {
                        // Cancel
                    }
                } else {
                    ui.horizontal(|ui| {
                        let install_btn = ui.add_enabled(
                            !self.mc_version.is_empty() && !self.optifine_version.is_empty(),
                            egui::Button::new("Install OptiFine").fill(egui::Color32::from_rgb(0, 150, 0))
                        );
                        if install_btn.clicked() {
                            // Download and install OptiFine
                        }
                        if ui.button("Cancel").clicked() {
                            // Close
                        }
                    });
                }
                
                ui.add_space(8.0);
                ui.separator();
                ui.label("Note: OptiFine downloads from optifine.net");
            });
    }
}

#[derive(Debug, Clone)]
pub struct OptiFineVersion {
    pub version: String,
    pub mc_version: String,
    pub url: String,
    pub patch: String,
    pub preview: bool,
}