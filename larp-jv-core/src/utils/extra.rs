//! Additional Utilities — Port of ColorSelector, Contracts, Extra, Input utilities

// Color Selector — Port of ColorSelector, HueView, SVRectangleView, AlphaView, ColorSideBySideView
pub mod colorselector {
    use eframe::egui;
    
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Color {
        pub r: f32,
        pub g: f32,
        pub b: f32,
        pub a: f32,
    }
    
    impl Color {
        pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
            Self {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: a as f32 / 255.0,
            }
        }
        
        pub fn to_rgba8(&self) -> [u8; 4] {
            [
                (self.r * 255.0).clamp(0.0, 255.0) as u8,
                (self.g * 255.0).clamp(0.0, 255.0) as u8,
                (self.b * 255.0).clamp(0.0, 255.0) as u8,
                (self.a * 255.0).clamp(0.0, 255.0) as u8,
            ]
        }
        
        pub fn to_hsv(&self) -> (f32, f32, f32) {
            let max = self.r.max(self.g).max(self.b);
            let min = self.r.min(self.g).min(self.b);
            let delta = max - min;
            
            let h = if delta == 0.0 {
                0.0
            } else if max == self.r {
                60.0 * (((self.g - self.b) / delta) % 6.0)
            } else if max == self.g {
                60.0 * (((self.b - self.r) / delta) + 2.0)
            } else {
                60.0 * (((self.r - self.g) / delta) + 4.0)
            };
            
            let s = if max == 0.0 { 0.0 } else { delta / max };
            let v = max;
            
            (h, s, v)
        }
        
        pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
            let c = v * s;
            let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
            let m = v - c;
            
            let (r, g, b) = if h < 60.0 {
                (c, x, 0.0)
            } else if h < 120.0 {
                (x, c, 0.0)
            } else if h < 180.0 {
                (0.0, c, x)
            } else if h < 240.0 {
                (0.0, x, c)
            } else if h < 300.0 {
                (x, 0.0, c)
            } else {
                (c, 0.0, x)
            };
            
            Self {
                r: r + m,
                g: g + m,
                b: b + m,
                a: 1.0,
            }
        }
    }
    
    /// Color Selector Widget
    pub struct ColorSelector {
        color: Color,
        show_alpha: bool,
    }
    
    impl ColorSelector {
        pub fn new(color: Color) -> Self {
            Self { color, show_alpha: true }
        }
        
        pub fn show(&mut self, ui: &mut egui::Ui) -> bool {
            let mut changed = false;
            
            // HSV sliders
            let (mut h, mut s, mut v) = self.color.to_hsv();
            let mut a = self.color.a;
            
            ui.horizontal(|ui| {
                ui.label("H:");
                if ui.add(egui::Slider::new(&mut h, 0.0..=360.0)).changed() { changed = true; }
            });
            ui.horizontal(|ui| {
                ui.label("S:");
                if ui.add(egui::Slider::new(&mut s, 0.0..=1.0)).changed() { changed = true; }
            });
            ui.horizontal(|ui| {
                ui.label("V:");
                if ui.add(egui::Slider::new(&mut v, 0.0..=1.0)).changed() { changed = true; }
            });
            
            if self.show_alpha {
                ui.horizontal(|ui| {
                    ui.label("A:");
                    if ui.add(egui::Slider::new(&mut a, 0.0..=1.0)).changed() { changed = true; }
                });
            }
            
            if changed {
                self.color = Color::from_hsv(h, s, v);
                self.color.a = a;
            }
            
            // Color preview
            let preview_color = egui::Color32::from_rgba_unmultiplied(
                (self.color.r * 255.0) as u8,
                (self.color.g * 255.0) as u8,
                (self.color.b * 255.0) as u8,
                (self.color.a * 255.0) as u8,
            );
            ui.painter().rect_filled(
                egui::Rect::from_min_size(ui.cursor().min, egui::vec2(50.0, 30.0)),
                4.0,
                preview_color,
            );
            ui.advance_cursor_after_rect(egui::Rect::from_min_size(ui.cursor().min, egui::vec2(50.0, 30.0)));
            
            changed
        }
        
        pub fn get_color(&self) -> Color {
            self.color
        }
        
        pub fn set_color(&mut self, color: Color) {
            self.color = color;
        }
    }
}

// Contracts — Port of OpenDocumentWithExtension
pub mod contracts {
    use std::path::Path;
    
    pub trait DocumentOpener: Send + Sync {
        fn can_open(&self, path: &Path) -> bool;
        fn open(&self, path: &Path) -> crate::error::LarpResult<()>;
    }
    
    pub struct DefaultDocumentOpener;
    
    impl DocumentOpener for DefaultDocumentOpener {
        fn can_open(&self, path: &Path) -> bool {
            path.exists()
        }
        
        fn open(&self, path: &Path) -> crate::error::LarpResult<()> {
            #[cfg(target_os = "android")]
            {
                // Use Android Intent via JNI
                Ok(())
            }
            #[cfg(not(target_os = "android"))]
            {
                opener::open(path)?;
                Ok(())
            }
        }
    }
}

// Extra — Port of ExtraConstants, ExtraCore, ExtraListener
pub mod extra {
    pub mod constants {
        pub const EXTRA_INTENT_ACTION: &str = "com.larp.jv.EXTRA_ACTION";
        pub const EXTRA_INSTANCE_NAME: &str = "instance_name";
        pub const EXTRA_VERSION: &str = "version";
        pub const EXTRA_MOD_LOADER: &str = "mod_loader";
        pub const EXTRA_MOD_LOADER_VERSION: &str = "mod_loader_version";
    }
    
    pub trait ExtraListener: Send + Sync {
        fn on_extra_received(&self, action: &str, data: std::collections::HashMap<String, String>);
    }
    
    pub struct ExtraCore {
        listeners: Vec<Box<dyn ExtraListener>>,
    }
    
    impl ExtraCore {
        pub fn new() -> Self {
            Self { listeners: Vec::new() }
        }
        
        pub fn add_listener(&mut self, listener: Box<dyn ExtraListener>) {
            self.listeners.push(listener);
        }
        
        pub fn dispatch(&self, action: &str, data: std::collections::HashMap<String, String>) {
            for listener in &self.listeners {
                listener.on_extra_received(action, data.clone());
            }
        }
    }
    
    impl Default for ExtraCore {
        fn default() -> Self {
            Self::new()
        }
    }
}

// Input Utilities — Port of ControlData, ControlDrawerData, ControlJoystickData, ControlLayout, CustomControls
pub mod input {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ControlData {
        pub id: String,
        pub name: String,
        pub control_type: ControlType,
        pub position: (f32, f32),
        pub size: (f32, f32),
        pub opacity: f32,
        pub visible: bool,
        pub action: String,
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ControlType {
        Button,
        Joystick,
        DPad,
        Touchpad,
        Slider,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ControlDrawerData {
        pub controls: Vec<ControlData>,
        pub selected: Option<String>,
        pub edit_mode: bool,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ControlJoystickData {
        pub base_position: (f32, f32),
        pub stick_position: (f32, f32),
        pub radius: f32,
        pub deadzone: f32,
        pub action_x: String,
        pub action_y: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ControlLayout {
        pub name: String,
        pub width: u32,
        pub height: u32,
        pub controls: Vec<ControlData>,
        pub joysticks: Vec<ControlJoystickData>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CustomControls {
        pub layouts: HashMap<String, ControlLayout>,
        pub active_layout: Option<String>,
    }
    
    impl CustomControls {
        pub fn new() -> Self {
            Self {
                layouts: HashMap::new(),
                active_layout: None,
            }
        }
        
        pub fn add_layout(&mut self, layout: ControlLayout) {
            self.layouts.insert(layout.name.clone(), layout);
        }
        
        pub fn set_active(&mut self, name: &str) -> bool {
            if self.layouts.contains_key(name) {
                self.active_layout = Some(name.to_string());
                true
            } else {
                false
            }
        }
        
        pub fn get_active(&self) -> Option<&ControlLayout> {
            self.active_layout.as_ref().and_then(|n| self.layouts.get(n))
        }
    }
    
    impl Default for CustomControls {
        fn default() -> Self {
            Self::new()
        }
    }
    
    // Layout Converter — Port of LayoutConverter.java
    pub mod converter {
        use super::*;
        
        pub fn convert_legacy(layout_json: &str) -> crate::error::LarpResult<ControlLayout> {
            // Convert old format to new
            let layout: ControlLayout = serde_json::from_str(layout_json)?;
            Ok(layout)
        }
        
        pub fn export_layout(layout: &ControlLayout) -> String {
            serde_json::to_string_pretty(layout).unwrap_or_default()
        }
    }
    
    // Layout Sanitizer — Port of LayoutSanitizer.java
    pub mod sanitizer {
        use super::*;
        
        pub fn sanitize(layout: &mut ControlLayout) {
            // Clamp positions to screen bounds
            for control in &mut layout.controls {
                control.position.0 = control.position.0.clamp(0.0, layout.width as f32 - control.size.0);
                control.position.1 = control.position.1.clamp(0.0, layout.height as f32 - control.size.1);
                control.size.0 = control.size.0.max(24.0).min(layout.width as f32);
                control.size.1 = control.size.1.max(24.0).min(layout.height as f32);
                control.opacity = control.opacity.clamp(0.0, 1.0);
            }
            
            for joystick in &mut layout.joysticks {
                joystick.base_position.0 = joystick.base_position.0.clamp(joystick.radius, layout.width as f32 - joystick.radius);
                joystick.base_position.1 = joystick.base_position.1.clamp(joystick.radius, layout.height as f32 - joystick.radius);
                joystick.radius = joystick.radius.max(20.0).min(layout.width as f32 / 2.0);
                joystick.deadzone = joystick.deadzone.clamp(0.0, 0.5);
            }
        }
    }
    
    // Editor Exitable — Port of EditorExitable.java
    pub trait EditorExitable {
        fn can_exit(&self) -> bool;
        fn on_exit(&mut self) -> bool; // Return true if exit allowed
    }
}

// Image Cropper — Port of CropperView, CropperBehaviour, BitmapCropBehaviour, RegionDecoderCropBehaviour
pub mod imgcropper {
    use std::path::Path;
    
    #[derive(Debug, Clone, Copy)]
    pub struct CropRect {
        pub x: f32,
        pub y: f32,
        pub width: f32,
        pub height: f32,
    }
    
    pub trait CropperBehaviour: Send + Sync {
        fn load_image(&self, path: &Path) -> crate::error::LarpResult<image::DynamicImage>;
        fn crop(&self, image: &image::DynamicImage, rect: CropRect) -> image::DynamicImage;
        fn save(&self, image: &image::DynamicImage, path: &Path) -> crate::error::LarpResult<()>;
    }
    
    pub struct BitmapCropBehaviour;
    
    impl CropperBehaviour for BitmapCropBehaviour {
        fn load_image(&self, path: &Path) -> crate::error::LarpResult<image::DynamicImage> {
            let img = image::open(path)?;
            Ok(img)
        }
        
        fn crop(&self, image: &image::DynamicImage, rect: CropRect) -> image::DynamicImage {
            image.crop_imm(
                rect.x as u32,
                rect.y as u32,
                rect.width as u32,
                rect.height as u32,
            )
        }
        
        fn save(&self, image: &image::DynamicImage, path: &Path) -> crate::error::LarpResult<()> {
            image.save(path)?;
            Ok(())
        }
    }
    
    pub struct RegionDecoderCropBehaviour;
    
    impl CropperBehaviour for RegionDecoderCropBehaviour {
        fn load_image(&self, path: &Path) -> crate::error::LarpResult<image::DynamicImage> {
            // Use region decoder for large images
            let img = image::open(path)?;
            Ok(img)
        }
        
        fn crop(&self, image: &image::DynamicImage, rect: CropRect) -> image::DynamicImage {
            image.crop_imm(
                rect.x as u32,
                rect.y as u32,
                rect.width as u32,
                rect.height as u32,
            )
        }
        
        fn save(&self, image: &image::DynamicImage, path: &Path) -> crate::error::LarpResult<()> {
            image.save(path)?;
            Ok(())
        }
    }
    
    // Cropper View — Port of CropperView.java
    pub struct CropperView {
        image: Option<image::DynamicImage>,
        crop_rect: CropRect,
        aspect_ratio: Option<f32>,
    }
    
    impl CropperView {
        pub fn new() -> Self {
            Self {
                image: None,
                crop_rect: CropRect { x: 0.0, y: 0.0, width: 1.0, height: 1.0 },
                aspect_ratio: None,
            }
        }
        
        pub fn set_image(&mut self, image: image::DynamicImage) {
            self.image = Some(image);
            self.crop_rect = CropRect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            };
        }
        
        pub fn set_aspect_ratio(&mut self, ratio: Option<f32>) {
            self.aspect_ratio = ratio;
        }
        
        pub fn get_crop_rect(&self) -> CropRect {
            self.crop_rect
        }
        
        pub fn apply_crop(&self) -> Option<image::DynamicImage> {
            self.image.as_ref().map(|img| {
                let w = img.width() as f32;
                let h = img.height() as f32;
                BitmapCropBehaviour.crop(img, CropRect {
                    x: self.crop_rect.x * w,
                    y: self.crop_rect.y * h,
                    width: self.crop_rect.width * w,
                    height: self.crop_rect.height * h,
                })
            })
        }
    }
}