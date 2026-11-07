//! Settings Video — port of LauncherPreferenceVideoFragment.java

use eframe::egui;

pub struct SettingsVideoUi {
    resolution: String,
    fullscreen: bool,
    vsync: bool,
    render_distance: u32,
    fov: u32,
    gamma: f32,
    view_bobbing: bool,
    clouds: bool,
    particles: String,
    mipmap: u32,
}

impl SettingsVideoUi {
    pub fn new() -> Self {
        Self {
            resolution: "Auto".to_string(),
            fullscreen: false,
            vsync: true,
            render_distance: 12,
            fov: 90,
            gamma: 100.0,
            view_bobbing: true,
            clouds: true,
            particles: "All".to_string(),
            mipmap: 4,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Video Settings")
            .resizable(true)
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Video Settings");
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Resolution: ");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.resolution)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.resolution, "Auto".to_string(), "Auto");
                            ui.selectable_value(&mut self.resolution, "854x480".to_string(), "854x480");
                            ui.selectable_value(&mut self.resolution, "1280x720".to_string(), "1280x720");
                            ui.selectable_value(&mut self.resolution, "1920x1080".to_string(), "1920x1080");
                            ui.selectable_value(&mut self.resolution, "2560x1440".to_string(), "2560x1440");
                        });
                });
                ui.add_space(8.0);

                ui.checkbox(&mut self.fullscreen, "Fullscreen");
                ui.add_space(8.0);

                ui.checkbox(&mut self.vsync, "VSync");
                ui.add_space(8.0);

                ui.add(egui::Slider::new(&mut self.render_distance, 2..=32).text("Render Distance"));
                ui.add_space(8.0);

                ui.add(egui::Slider::new(&mut self.fov, 30..=110).text("FOV"));
                ui.add_space(8.0);

                ui.add(egui::Slider::new(&mut self.gamma, 0.0..=200.0).text("Gamma"));
                ui.add_space(8.0);

                ui.checkbox(&mut self.view_bobbing, "View Bobbing");
                ui.add_space(8.0);

                ui.checkbox(&mut self.clouds, "Clouds");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Particles: ");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.particles)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.particles, "All".to_string(), "All");
                            ui.selectable_value(&mut self.particles, "Decreased".to_string(), "Decreased");
                            ui.selectable_value(&mut self.particles, "Minimal".to_string(), "Minimal");
                        });
                });
                ui.add_space(8.0);

                ui.add(egui::Slider::new(&mut self.mipmap, 0..=4).text("Mipmap Levels"));
            });
    }
}

impl Default for SettingsVideoUi {
    fn default() -> Self {
        Self::new()
    }
}