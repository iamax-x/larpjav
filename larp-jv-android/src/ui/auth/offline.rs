//! Offline Login — port of LocalLoginFragment.java

use eframe::egui;
use larp_jv_core::auth::{OfflineLogin, BackgroundLogin};
use std::sync::Arc;

pub struct OfflineLoginUi {
    username: String,
    logging_in: bool,
    error: Option<String>,
}

impl OfflineLoginUi {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            logging_in: false,
            error: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, on_success: impl FnOnce()) {
        egui::Window::new("Offline Login")
            .resizable(false)
            .min_width(350.0)
            .show(ctx, |ui| {
                ui.heading("Offline Account");
                ui.separator();
                ui.add_space(8.0);

                ui.label("Create an offline account (no internet required)");
                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    ui.label("Username: ");
                    ui.text_edit_singleline(&mut self.username);
                });
                ui.add_space(16.0);

                if let Some(err) = &self.error {
                    ui.colored_label(egui::Color32::RED, err);
                    ui.add_space(8.0);
                }

                ui.horizontal(|ui| {
                    let login_btn = ui.add_enabled(!self.username.trim().is_empty(),
                        egui::Button::new("Create Account").fill(egui::Color32::from_rgb(0, 150, 0)));
                    if login_btn.clicked() {
                        // Create offline account
                    }
                    if ui.button("Cancel").clicked() {
                        // Close
                    }
                });
            });
    }
}

impl Default for OfflineLoginUi {
    fn default() -> Self {
        Self::new()
    }
}