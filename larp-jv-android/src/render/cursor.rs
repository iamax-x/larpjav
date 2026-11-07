//! Custom Cursor Rendering — port of GameCursorView.java

use eframe::egui;

pub struct GameCursor {
    visible: bool,
    custom_texture: Option<egui::TextureHandle>,
    position: egui::Vec2,
    hotspot: egui::Vec2,
}

impl GameCursor {
    pub fn new() -> Self {
        Self {
            visible: true,
            custom_texture: None,
            position: egui::Vec2::ZERO,
            hotspot: egui::Vec2::new(0.0, 0.0),
        }
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn set_position(&mut self, pos: egui::Vec2) {
        self.position = pos;
    }

    pub fn set_custom_texture(&mut self, texture: egui::TextureHandle, hotspot: egui::Vec2) {
        self.custom_texture = Some(texture);
        self.hotspot = hotspot;
    }

    pub fn render(&self, ctx: &egui::Context) {
        if !self.visible {
            return;
        }

        if let Some(texture) = &self.custom_texture {
            let rect = egui::Rect::from_min_size(
                self.position - self.hotspot,
                texture.size_vec2(),
            );
            egui::Area::new(egui::Id::new("game_cursor"))
                .fixed_pos(self.position - self.hotspot)
                .show(&ctx, |ui| {
                    ui.image((texture.id(), texture.size_vec2()));
                });
        } else {
            // Default system cursor
            ctx.set_cursor_icon(egui::CursorIcon::Default);
        }
    }

    pub fn position(&self) -> egui::Vec2 {
        self.position
    }
}

impl Default for GameCursor {
    fn default() -> Self {
        Self::new()
    }
}