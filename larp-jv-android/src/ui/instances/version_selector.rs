//! Version Selector — port of VersionSelectorDialog.java

use eframe::egui;
use larp_jv_core::{VersionManifest, VersionInfo};

pub struct VersionSelectorUi {
    manifest: Option<VersionManifest>,
    selected_version: Option<String>,
    filter_text: String,
    show_snapshots: bool,
    show_old_beta: bool,
    show_old_alpha: bool,
}

impl VersionSelectorUi {
    pub fn new() -> Self {
        Self {
            manifest: None,
            selected_version: None,
            filter_text: String::new(),
            show_snapshots: true,
            show_old_beta: false,
            show_old_alpha: false,
        }
    }

    pub fn set_manifest(&mut self, manifest: VersionManifest) {
        self.manifest = Some(manifest);
    }

    pub fn show(&mut self, ctx: &egui::Context, on_select: impl FnOnce(String)) {
        egui::Window::new("Select Minecraft Version")
            .resizable(true)
            .min_width(500.0)
            .min_height(600.0)
            .show(ctx, |ui| {
                ui.heading("Select Minecraft Version");
                ui.separator();

                // Filter controls
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.filter_text).hint_text("Search versions...");
                    ui.checkbox(&mut self.show_snapshots, "Snapshots");
                    ui.checkbox(&mut self.show_old_beta, "Old Beta");
                    ui.checkbox(&mut self.show_old_alpha, "Old Alpha");
                });
                ui.separator();

                // Version list
                if let Some(manifest) = &self.manifest {
                    let versions = self.filter_versions(manifest);
                    
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for version in versions {
                            let is_selected = self.selected_version.as_ref() == Some(&version.id);
                            let label = format!("{} ({})", version.id, version.version_type);
                            
                            if ui.selectable_label(is_selected, &label).clicked() {
                                self.selected_version = Some(version.id.clone());
                            }
                        }
                    }
                } else {
                    ui.label("Loading versions...");
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        // Close
                    }
                    if ui.add_enabled(self.selected_version.is_some(), egui::Button::new("Select")).clicked() {
                        if let Some(version) = self.selected_version.clone() {
                            // Callback would be here
                        }
                    }
                });
            });
    }

    fn filter_versions(&self, manifest: &VersionManifest) -> Vec<VersionInfo> {
        manifest.versions.iter()
            .filter(|v| {
                // Filter by type
                let type_match = match v.version_type {
                    crate::value::VersionType::Release => true,
                    crate::value::VersionType::Snapshot => self.show_snapshots,
                    crate::value::VersionType::OldBeta => self.show_old_beta,
                    crate::value::VersionType::OldAlpha => self.show_old_alpha,
                };
                
                // Filter by text
                let text_match = self.filter_text.is_empty() 
                    || v.id.to_lowercase().contains(&self.filter_text.to_lowercase());
                
                type_match && text_match
            })
            .cloned()
            .collect()
    }
}

impl Default for VersionSelectorUi {
    fn default() -> Self {
        Self::new()
    }
}