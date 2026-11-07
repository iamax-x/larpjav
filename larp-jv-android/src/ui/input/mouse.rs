//! Virtual Mouse Input — port of mouse input handling

use eframe::egui;

pub struct VirtualMouseUi {
    sensitivity: f32,
    inverted: bool,
    gesture_enabled: bool,
}

impl VirtualMouseUi {
    pub fn new() -> Self {
        Self {
            sensitivity: 1.0,
            inverted: false,
            gesture_enabled: true,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Virtual Mouse Settings")
            .resizable(true)
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Virtual Mouse");
                ui.separator();
                ui.add_space(8.0);

                ui.add(egui::Slider::new(&mut self.sensitivity, 0.1..=3.0).text("Sensitivity"));
                ui.add_space(8.0);

                ui.checkbox(&mut self.inverted, "Invert Y Axis");
                ui.add_space(8.0);

                ui.checkbox(&mut self.gesture_enabled, "Enable Gestures");
                ui.add_space(16.0);

                ui.separator();
                ui.label("Gestures:");
                ui.add_space(8.0);
                ui.label("• Tap = Left Click");
                ui.label("• Two-finger tap = Right Click");
                ui.label("• Two-finger drag = Scroll");
                ui.label("• Pinch = Zoom");
            });
    }
}

impl Default for VirtualMouseUi {
    fn default() -> Self {
        Self::new()
    }
}