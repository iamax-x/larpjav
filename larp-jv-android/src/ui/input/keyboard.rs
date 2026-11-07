//! Virtual Keyboard Input — port of TouchCharInput.java

use eframe::egui;

pub struct VirtualKeyboardUi {
    enabled: bool,
    size: f32,
    haptic_feedback: bool,
}

impl VirtualKeyboardUi {
    pub fn new() -> Self {
        Self {
            enabled: true,
            size: 1.0,
            haptic_feedback: true,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Virtual Keyboard")
            .resizable(true)
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Virtual Keyboard");
                ui.separator();
                ui.add_space(8.0);

                ui.checkbox(&mut true, "Enable Virtual Keyboard");
                ui.add_space(8.0);

                ui.add(egui::Slider::new(&mut 1.0, 0.5..=2.0).text("Key Size"));
                ui.add_space(8.0);

                ui.checkbox(&mut true, "Haptic Feedback");
                ui.add_space(16.0);

                ui.separator();
                ui.label("Layout: QWERTY (default)");
                ui.label("Supports: letters, numbers, symbols");
            });
    }
}

impl Default for VirtualKeyboardUi {
    fn default() -> Self {
        Self::new()
    }
}