//! Microsoft Login — port of MicrosoftLoginFragment.java

use eframe::egui;
use larp_jv_core::auth::{MicrosoftAuthClient, BackgroundLogin};
use std::sync::Arc;
use std::time::Duration;

pub struct MicrosoftLoginUi {
    client: Arc<MicrosoftAuthClient>,
    login: Option<Box<dyn BackgroundLogin>>,
    user_code: String,
    verification_uri: String,
    message: String,
    polling: bool,
    last_poll: std::time::Instant,
    error: Option<String>,
}

impl MicrosoftLoginUi {
    pub fn new(client: Arc<crate::auth::MicrosoftAuthClient>) -> Self {
        Self {
            client,
            login: None,
            user_code: String::new(),
            verification_uri: String::new(),
            message: String::new(),
            polling: false,
            last_poll: std::time::Instant::now(),
            error: None,
        }
    }

    pub fn start_login(&mut self) {
        // Start device code flow
        // This would be async in real implementation
        self.polling = true;
        self.last_poll = std::time::Instant::now();
        self.error = None;
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Microsoft Login")
            .resizable(false)
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Sign in with Microsoft");
                ui.separator();
                ui.add_space(8.0);

                if !self.polling {
                    if ui.button("Sign in with Microsoft").clicked() {
                        self.start_login();
                    }
                } else {
                    // Show device code
                    ui.label("Please sign in at:");
                    ui.add_space(8.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("URL: ");
                        if ui.button(&self.verification_uri).clicked() {
                            // Copy to clipboard
                        }
                    });
                    
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("Code: ");
                        ui.strong(&self.user_code);
                        if ui.button("Copy").clicked() {
                            // Copy to clipboard
                        }
                    });
                    
                    ui.add_space(16.0);
                    ui.label(&self.message);
                    
                    // Progress
                    ui.add_space(16.0);
                    ui.add(egui::ProgressBar::new(0.5).show_percentage());
                    ui.label("Waiting for sign-in...");
                    
                    if ui.button("Cancel").clicked() {
                        self.polling = false;
                    }
                }
            });
    }
}