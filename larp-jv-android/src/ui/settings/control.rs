//! Settings Control — port of LauncherPreferenceControlFragment.java

use eframe::egui;

pub struct SettingsControlUi {
    sensitivity: f32,
    invert_mouse: bool,
    auto_jump: bool,
    touchscreen: bool,
    touchscreen_mode: bool,
}

impl SettingsControlUi {
    pub fn new() -> Self {
        Self {
            sensitivity: 100.0,
            invert_mouse: false,
            auto_jump: true,
            touchscreen: false,
            touchscreen_mode: false,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Control Settings")
            .resizable(true)
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Control Settings");
                ui.separator();
                ui.add_space(8.0);

                ui.add(egui::Slider::new(&mut self.sensitivity, 0.0..=200.0).text("Mouse Sensitivity"));
                ui.add_space(8.0);

                ui.checkbox(&mut self.invert_mouse, "Invert Mouse");
                ui.add_space(8.0);

                ui.checkbox(&mut self.auto_jump, "Auto Jump");
                ui.add_space(8.0);

                ui.checkbox(&mut self.touchscreen, "Touchscreen Mode");
                ui.add_space(8.0);

                ui.checkbox(&mut self.touchscreen_mode, "Touchscreen Movement");
                ui.add_space(16.0);

                ui.separator();
                ui.label("Gamepad Settings");
                ui.add_space(8.0);
                ui.label("Gamepad mapping available in Controls screen");
            });
    }
}

impl Default for SettingsControlUi {
    fn default() -> Self {
        Self::new()
    }
}