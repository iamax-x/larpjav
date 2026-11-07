//! Ely.by Login — port of ElyByLoginFragment.java

use eframe::egui;
use larp_jv_core::auth::{ElyByAuthClient, BackgroundLogin};
use std::sync::Arc;

pub struct ElyByLoginUi {
    client: Arc<crate::auth::ElyByAuthClient>,
    login: Option<Box<dyn BackgroundLogin>>,
    email: String,
    password: String,
    logging_in: bool,
    error: Option<String>,
}

impl ElyByLoginUi {
    pub fn new(client: Arc<crate::auth::ElyByAuthClient>) -> Self {
        Self {
            client,
            login: None,
            email: String::new(),
            password: String::new(),
            logging_in: false,
            error: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, on_success: impl FnOnce()) {
        egui::Window::new("Ely.by Login")
            .resizable(false)
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Sign in with Ely.by");
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Email: ");
                    ui.text_edit_singleline(&mut self.email);
                });
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Password: ");
                    ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
                });
                ui.add_space(16.0);

                if let Some(err) = &self.error {
                    ui.colored_label(egui::Color32::RED, err);
                    ui.add_space(8.0);
                }

                ui.horizontal(|ui| {
                    let login_btn = ui.add_enabled(!self.email.is_empty() && !self.password.is_empty(),
                        egui::Button::new("Login").fill(egui::Color32::from_rgb(0, 150, 0)));
                    if login_btn.clicked() {
                        // Start login
                    }
                    if ui.button("Cancel").clicked() {
                        // Close
                    }
                });
            });
    }
}