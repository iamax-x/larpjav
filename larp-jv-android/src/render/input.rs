//! Game Input Processing — port of InGameEventProcessor.java / InGUIEventProcessor.java / TouchEventProcessor.java

use eframe::egui;

pub struct GameInputProcessor {
    mouse_x: f32,
    mouse_y: f32,
    mouse_down: bool,
    keys_pressed: std::collections::HashSet<egui::Key>,
}

impl GameInputProcessor {
    pub fn new() -> Self {
        Self {
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_down: false,
            keys_pressed: std::collections::HashSet::new(),
        }
    }

    pub fn handle_egui_event(&mut self, event: &egui::Event) {
        match event {
            egui::Event::PointerMoved(pos) => {
                self.mouse_x = pos.x;
                self.mouse_y = pos.y;
            }
            egui::Event::PointerButton { pos, button, down, .. } => {
                self.mouse_x = pos.x;
                self.mouse_y = pos.y;
                self.mouse_down = down;
            }
            egui::Event::Key { key, pressed, .. } => {
                if pressed {
                    self.keys_pressed.insert(key);
                } else {
                    self.keys_pressed.remove(&key);
                }
            }
            _ => {}
        }
    }

    pub fn mouse_position(&self) -> (f32, f32) {
        (self.mouse_x, self.mouse_y)
    }

    pub fn is_mouse_down(&self) -> bool {
        self.mouse_down
    }

    pub fn is_key_pressed(&self, key: egui::Key) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn clear_keys(&mut self) {
        self.keys_pressed.clear();
    }
}

impl Default for GameInputProcessor {
    fn default() -> Self {
        Self::new()
    }
}