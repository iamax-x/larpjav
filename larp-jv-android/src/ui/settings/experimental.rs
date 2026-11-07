//! Settings Experimental — port of LauncherPreferenceExperimentalFragment.java

use eframe::egui;

pub struct SettingsExperimentalUi {
    debug_logging: bool,
    force_gl3: bool,
    custom_renderer: bool,
    show_fps: bool,
    show_coords: bool,
}

impl SettingsExperimentalUi {
    pub fn new() -> Self {
        Self {
            debug_logging: false,
            force_gl3: false,
            custom_renderer: false,
            show_fps: false,
            show_coords: false,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Experimental Settings")
            .resizable(true)
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Experimental Settings");
                ui.separator();
                ui.add_space(8.0);

                ui.label("⚠️ These settings may cause instability");
                ui.separator();
                ui.add_space(8.0);

                ui.checkbox(&mut self.debug_logging, "Enable Debug Logging");
                ui.add_space(8.0);

                ui.checkbox(&mut self.force_gl3, "Force OpenGL 3.0+");
                ui.add_space(8.0);

                ui.checkbox(&mut self.custom_renderer, "Custom Renderer");
                ui.add_space(8.0);

                ui.checkbox(&mut self.show_fps, "Show FPS Counter");
                ui.add_space(8.0);

                ui.checkbox(&mut self.show_coords, "Show Coordinates");
                ui.add_space(16.0);

                ui.separator();
                ui.colored_label(egui::Color32::YELLOW, "Changes require restart");
            });
    }
}

impl Default for SettingsExperimentalUi {
    fn default() -> Self {
        Self::new()
    }
}