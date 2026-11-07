//! Main Menu Fragment — port of MainMenuFragment.java

use eframe::egui;
use larp_jv_core::{AuthManager, InstanceManager};
use std::sync::Arc;

pub struct MainMenuUi {
    auth_manager: Arc<crate::auth::AuthManager>,
    instance_manager: Arc<crate::instances::InstanceManager>,
    selected_instance: Option<String>,
    show_play_dialog: bool,
}

impl MainMenuUi {
    pub fn new(
        auth_manager: Arc<crate::auth::AuthManager>,
        instance_manager: Arc<crate::instances::InstanceManager>,
    ) -> Self {
        Self {
            auth_manager,
            instance_manager,
            selected_instance: None,
            show_play_dialog: false,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        // Background panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                
                // Logo
                ui.heading("larp.jv");
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Pure Rust Minecraft Launcher").size(14.0).color(egui::Color32::GRAY));
                ui.add_space(40.0);

                // Selected instance display
                if let Some(selected) = &self.selected_instance {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong("Selected: ");
                            ui.label(selected);
                            if ui.small_button("Change").clicked() {
                                self.selected_instance = None;
                            }
                        });
                    });
                    ui.add_space(20.0);
                }

                // Play button
                let play_enabled = self.selected_instance.is_some();
                if ui.add_enabled(play_enabled, egui::Button::new(egui::RichText::new("PLAY").size(24.0)).min_size(egui::vec2(280.0, 70.0))).clicked() {
                    // Launch instance
                    self.launch_selected();
                }
                ui.add_space(16.0);

                // Quick actions row
                ui.horizontal(|ui| {
                    if ui.button("Instances").clicked() {
                        // Navigate to instances
                    }
                    if ui.button("Mods").clicked() {
                        // Navigate to modding
                    }
                    if ui.button("Settings").clicked() {
                        // Navigate to settings
                    }
                });
                ui.add_space(24.0);

                // Account info
                ui.separator();
                ui.add_space(8.0);
                if let Some(account) = self.auth_manager.accounts().get_selected() {
                    ui.horizontal(|ui| {
                        ui.label("Account: ");
                        ui.strong(&account.username);
                        ui.label(format!("({:?})", account.auth_type));
                    });
                } else {
                    ui.label("No account selected");
                    if ui.button("Sign In").clicked() {
                        // Navigate to auth
                    }
                }
            });
        });
    }

    fn launch_selected(&self) {
        if let Some(instance_id) = &self.selected_instance {
            // Launch logic would go here
            log::info!("Launching instance: {}", instance_id);
        }
    }
}