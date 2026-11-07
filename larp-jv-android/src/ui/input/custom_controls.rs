//! Custom Controls Editor — port of CustomControlsActivity.java

use eframe::egui;

pub struct CustomControlsUi {
    controls: Vec<CustomControl>,
    selected: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct CustomControl {
    pub name: String,
    pub control_type: ControlType,
    pub position: egui::Vec2,
    pub size: f32,
    pub opacity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlType {
    Button,
    Joystick,
    DPad,
}

impl CustomControlsUi {
    pub fn new() -> Self {
        Self {
            controls: vec![],
            selected: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Custom Controls Editor")
            .resizable(true)
            .min_width(600.0)
            .min_height(500.0)
            .show(ctx, |ui| {
                ui.heading("Custom Controls Editor");
                ui.separator();
                ui.add_space(8.0);

                // Toolbar
                ui.horizontal(|ui| {
                    if ui.button("Add Button").clicked() {
                        // Add button
                    }
                    if ui.button("Add Joystick").clicked() {
                        // Add joystick
                    }
                    if ui.button("Add D-Pad").clicked() {
                        // Add D-Pad
                    }
                    ui.separator();
                    if ui.button("Save Layout").clicked() {
                        // Save
                    }
                    if ui.button("Load Layout").clicked() {
                        // Load
                    }
                    if ui.button("Reset").clicked() {
                        // Reset
                    }
                });
                ui.separator();

                // Canvas area
                ui.label("Drag controls to position them");
                ui.add_space(8.0);

                // Controls list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (idx, control) in vec![].iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("Control {}: {:?}", idx + 1, ControlType::Button));
                                ui.add(egui::Slider::new(&mut 1.0, 0.5..=2.0).text("Size"));
                                ui.add(egui::Slider::new(&mut 1.0, 0.0..=1.0).text("Opacity"));
                                if ui.button("🗑").clicked() {
                                    // Delete
                                }
                            });
                        });
                    }
                });
            });
    }
}

impl Default for CustomControlsUi {
    fn default() -> Self {
        Self::new()
    }
}