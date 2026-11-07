//! Select Auth Method — port of SelectAuthFragment.java

use eframe::egui;
use larp_jv_core::value::AuthType;

pub struct SelectAuthUi {
    selected: Option<AuthType>,
}

impl SelectAuthUi {
    pub fn new() -> Self {
        Self { selected: None }
    }

    pub fn update(&mut self, ctx: &egui::Context, on_select: impl FnOnce(AuthType)) {
        egui::Window::new("Sign In")
            .resizable(false)
            .min_width(350.0)
            .show(ctx, |ui| {
                ui.heading("Choose Sign-In Method");
                ui.separator();
                ui.add_space(8.0);

                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    
                    // Microsoft
                    let ms_response = ui.add(egui::Button::new(
                        egui::RichText::new("Microsoft (Xbox Live)").size(16.0)
                    ).min_size(egui::vec2(280.0, 50.0)));
                    if ms_response.clicked() {
                        // Select Microsoft
                    }
                    ui.add_space(8.0);
                    
                    // Ely.by
                    let ely_response = ui.add(egui::Button::new(
                        egui::RichText::new("Ely.by").size(16.0)
                    ).min_size(egui::vec2(280.0, 50.0)));
                    if ely_response.clicked() {
                        // Select Ely.by
                    }
                    ui.add_space(8.0);
                    
                    // Offline
                    let offline_response = ui.add(egui::Button::new(
                        egui::RichText::new("Offline").size(16.0)
                    ).min_size(egui::vec2(280.0, 50.0)));
                    if offline_response.clicked() {
                        // Select Offline
                    }
                });
            });
    }
}

impl Default for SelectAuthUi {
    fn default() -> Self {
        Self::new()
    }
}