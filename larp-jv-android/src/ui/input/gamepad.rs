//! Gamepad Mapper — port of GamepadMapperFragment.java

use eframe::egui;

pub struct GamepadMapperUi {
    mappings: Vec<GamepadMapping>,
    selected: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct GamepadMapping {
    pub button: String,
    pub action: String,
    pub deadzone: f32,
}

impl GamepadMapperUi {
    pub fn new() -> Self {
        Self {
            mappings: vec![
                GamepadMapping { button: "A".to_string(), action: "Jump".to_string(), deadzone: 0.1 },
                GamepadMapping { button: "B".to_string(), action: "Sneak".to_string(), deadzone: 0.1 },
                GamepadMapping { button: "X".to_string(), action: "Inventory".to_string(), deadzone: 0.1 },
                GamepadMapping { button: "Y".to_string(), action: "Drop".to_string(), deadzone: 0.1 },
                GamepadMapping { button: "LB".to_string(), action: "Sprint".to_string(), deadzone: 0.1 },
                GamepadMapping { button: "RB".to_string(), action: "Attack".to_string(), deadzone: 0.1 },
                GamepadMapping { button: "LT".to_string(), action: "Use".to_string(), deadzone: 0.1 },
                GamepadMapping { button: "RT".to_string(), action: "Break".to_string(), deadzone: 0.1 },
            ],
            selected: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Gamepad Mapper")
            .resizable(true)
            .min_width(500.0)
            .show(ctx, |ui| {
                ui.heading("Gamepad Mapping");
                ui.separator();
                ui.add_space(8.0);

                ui.label("Connect your gamepad and map buttons to actions");
                ui.add_space(16.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (idx, mapping) in self.mappings.iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("Button {}:", mapping.button));
                                ui.text_edit_singleline(&mut mapping.action);
                                ui.add(egui::Slider::new(&mut mapping.deadzone, 0.0..=1.0).text("Deadzone"));
                                if ui.button("🗑").clicked() {
                                    // Remove mapping
                                }
                            });
                        });
                    }
                });
                
                ui.add_space(16.0);
                if ui.button("Add Mapping").clicked() {
                    // Add new mapping
                }
                ui.add_space(8.0);
                if ui.button("Reset to Defaults").clicked() {
                    // Reset
                }
            });
    }
}

impl Default for GamepadMapperUi {
    fn default() -> Self {
        Self::new()
    }
}