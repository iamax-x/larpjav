//! Settings Java — port of LauncherPreferenceJavaFragment.java

use eframe::egui;
use larp_jv_core::jre::JreManager;
use std::sync::Arc;

pub struct SettingsJavaUi {
    jre_manager: Arc<crate::jre::JreManager>,
    java_path: String,
    jvm_args: String,
    auto_detect: bool,
    jre_list: Vec<String>,
    selected_jre: Option<String>,
}

impl SettingsJavaUi {
    pub fn new(jre_manager: Arc<crate::jre::JreManager>) -> Self {
        Self {
            jre_manager,
            java_path: String::new(),
            jvm_args: "-Xmx2G -Xms1G".to_string(),
            auto_detect: true,
            jre_list: Vec::new(),
            selected_jre: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Java Settings")
            .resizable(true)
            .min_width(500.0)
            .show(ctx, |ui| {
                ui.heading("Java Settings");
                ui.separator();
                ui.add_space(8.0);

                ui.checkbox(&mut self.auto_detect, "Auto-detect Java");
                ui.add_space(8.0);

                if self.auto_detect {
                    ui.label("Java will be auto-detected from system paths");
                    ui.add_space(8.0);
                    if ui.button("Detect Now").clicked() {
                        // Trigger detection
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Java Path: ");
                        ui.text_edit_singleline(&mut self.java_path);
                        if ui.button("Browse").clicked() {
                            // Open file picker
                        }
                    });
                    ui.add_space(8.0);
                }

                ui.separator();
                ui.add_space(8.0);

                ui.label("JVM Arguments:");
                ui.text_edit_multiline(&mut self.jvm_args);
                ui.add_space(8.0);

                ui.label("Available JREs:");
                if self.jre_list.is_empty() {
                    ui.label("No JREs detected. Click 'Detect Now' or 'Download JRE'.");
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for jre in &self.jre_list {
                            let selected = self.selected_jre.as_ref() == Some(jre);
                            if ui.selectable_label(selected, jre).clicked() {
                                self.selected_jre = Some(jre.clone());
                            }
                        }
                    });
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Detect Now").clicked() {
                        // Detect
                    }
                    if ui.button("Download JRE").clicked() {
                        // Open JRE downloader
                    }
                });
            });
    }
}