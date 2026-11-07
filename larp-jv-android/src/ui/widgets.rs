//! Widgets module - Custom widgets ported from MojoLauncher

use eframe::egui;

/// MineButton - Minecraft-style button
pub struct MineButton {
    text: String,
    enabled: bool,
    size: egui::Vec2,
}

impl MineButton {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            enabled: true,
            size: egui::vec2(200.0, 40.0),
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn min_size(mut self, size: egui::Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        let mut btn = egui::Button::new(egui::RichText::new(&self.text).size(16.0))
            .min_size(self.size)
            .fill(if self.enabled { 
                egui::Color32::from_rgb(0, 150, 0) 
            } else { 
                egui::Color32::from_rgb(80, 80, 80) 
            });
        
        if !self.enabled {
            btn = btn.fill(egui::Color32::from_rgb(80, 80, 80));
        }
        
        ui.add_enabled(self.enabled, btn)
    }
}

/// MineKnob - Minecraft-style knob/slider
pub struct MineKnob {
    value: f32,
    range: std::ops::RangeInclusive<f32>,
    label: String,
}

impl MineKnob {
    pub fn new(value: f32, range: std::ops::RangeInclusive<f32>) -> Self {
        Self {
            value,
            range,
            label: String::new(),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            if !self.label.is_empty() {
                ui.label(&self.label);
            }
            let response = ui.add(egui::Slider::new(&mut self.value, self.range));
            ui.label(format!("{:.1}", self.value));
            response
        }).inner
    }
}

/// ColorSelector - port of ColorSelector.java
pub struct ColorSelector {
    color: egui::Color32,
    show_alpha: bool,
}

impl ColorSelector {
    pub fn new(color: egui::Color32) -> Self {
        Self {
            color,
            show_alpha: false,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.color_edit_button_srgba(&mut self.color)
    }
}

/// ControlButton - port of ControlButton.java
pub struct ControlButton {
    label: String,
    icon: Option<egui::TextureHandle>,
    size: egui::Vec2,
}

impl ControlButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            size: egui::vec2(60.0, 60.0),
        }
    }

    pub fn icon(mut self, icon: egui::TextureHandle) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn size(mut self, size: egui::Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        let mut btn = egui::Button::new("").min_size(self.size);
        
        if let Some(icon) = &self.icon {
            btn = btn.icon(icon.clone());
        } else {
            btn = btn.text(self.label.clone());
        }
        
        ui.add(btn)
    }
}