//! Settings Main — port of LauncherPreferenceFragment.java

use eframe::egui;
use larp_jv_core::{AuthManager, InstanceManager, ModManager, JreManager, MoJsonManager};
use std::sync::Arc;

pub struct SettingsMainUi {
    auth_manager: Arc<crate::auth::AuthManager>,
    instance_manager: Arc<crate::instances::InstanceManager>,
    mod_manager: Arc<crate::modding::ModManager>,
    jre_manager: Arc<crate::jre::JreManager>,
    mojson_manager: Arc<crate::mojson::MoJsonManager>,
}

impl SettingsMainUi {
    pub fn new(
        auth_manager: Arc<crate::auth::AuthManager>,
        instance_manager: Arc<crate::instances::InstanceManager>,
        mod_manager: Arc<crate::modding::ModManager>,
        jre_manager: Arc<crate::jre::JreManager>,
        mojson_manager: Arc<crate::mojson::MoJsonManager>,
    ) -> Self {
        Self {
            auth_manager,
            instance_manager,
            mod_manager,
            jre_manager,
            mojson_manager,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Settings")
            .resizable(true)
            .min_width(600.0)
            .min_height(500.0)
            .show(ctx, |ui| {
                ui.heading("Settings");
                ui.separator();
                ui.add_space(8.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Java settings
                    ui.collapsing("Java", |ui| {
                        ui.label("Java Path: Auto-detect");
                        ui.label("JVM Arguments: -Xmx2G -Xms1G");
                        if ui.button("Detect Java").clicked() {
                            // Trigger JRE detection
                        }
                        if ui.button("Download JRE").clicked() {
                            // Open JRE downloader
                        }
                    });
                    
                    // Video settings
                    ui.collapsing("Video", |ui| {
                        ui.label("Resolution: Auto");
                        ui.checkbox(&mut false, "Fullscreen");
                        ui.checkbox(&mut true, "VSync");
                        ui.add(egui::Slider::new(&mut 12, 2..=32).text("Render Distance"));
                    });
                    
                    // Control settings
                    ui.collapsing("Controls", |ui| {
                        ui.add(egui::Slider::new(&mut 100.0, 0.0..=200.0).text("Mouse Sensitivity"));
                        ui.checkbox(&mut true, "Invert Mouse");
                        ui.checkbox(&mut false, "Touchscreen Mode");
                        ui.checkbox(&mut true, "Auto Jump");
                    });
                    
                    // Experimental settings
                    ui.collapsing("Experimental", |ui| {
                        ui.checkbox(&mut false, "Enable Debug Logging");
                        ui.checkbox(&mut false, "Force OpenGL 3.0+");
                        ui.checkbox(&mut false, "Custom Renderer");
                    });
                    
                    // Miscellaneous
                    ui.collapsing("Miscellaneous", |ui| {
                        ui.label("Language: System Default");
                        ui.checkbox(&mut true, "Show Console");
                        ui.checkbox(&mut true, "Show FPS");
                        if ui.button("Clear Cache").clicked() {
                            // Clear cache
                        }
                    });
                });
            });
    }
}