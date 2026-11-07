//! Settings Miscellaneous — port of LauncherPreferenceMiscellaneousFragment.java

use eframe::egui;

pub struct SettingsMiscellaneousUi {
    language: String,
    show_console: bool,
    show_fps: bool,
    clear_cache_on_exit: bool,
    notifications: bool,
}

impl SettingsMiscellaneousUi {
    pub fn new() -> Self {
        Self {
            language: "System Default".to_string(),
            show_console: true,
            show_fps: true,
            clear_cache_on_exit: false,
            notifications: true,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Miscellaneous Settings")
            .resizable(true)
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Miscellaneous");
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Language: ");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.language)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.language, "System Default".to_string(), "System Default");
                            ui.selectable_value(&mut self.language, "English".to_string(), "English");
                            ui.selectable_value(&mut self.language, "Português".to_string(), "Português");
                            ui.selectable_value(&mut self.language, "Español".to_string(), "Español");
                            ui.selectable_value(&mut self.language, "Français".to_string(), "Français");
                            ui.selectable_value(&mut self.language, "Deutsch".to_string(), "Deutsch");
                            ui.selectable_value(&mut self.language, "中文".to_string(), "中文");
                            ui.selectable_value(&mut self.language, "日本語".to_string(), "日本語");
                        });
                });
                ui.add_space(16.0);

                ui.checkbox(&mut self.show_console, "Show Console");
                ui.add_space(8.0);

                ui.checkbox(&mut self.show_fps, "Show FPS Counter");
                ui.add_space(8.0);

                ui.checkbox(&mut self.notifications, "Enable Notifications");
                ui.add_space(8.0);

                ui.checkbox(&mut self.clear_cache_on_exit, "Clear Cache on Exit");
                ui.add_space(16.0);

                ui.separator();
                ui.add_space(8.0);

                if ui.button("Clear Cache Now").clicked() {
                    // Clear cache
                }
                ui.add_space(8.0);

                if ui.button("Open Logs Folder").clicked() {
                    // Open logs
                }
            });
    }
}

impl Default for SettingsMiscellaneousUi {
    fn default() -> Self {
        Self::new()
    }
}