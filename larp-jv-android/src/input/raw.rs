//! Android Input handling — Touch, Key, Motion events → egui RawInput

use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase};
use winit::keyboard::{KeyCode, PhysicalKey};
use larp_jv_core::utils::keycode::{android_to_minecraft, android::*, minecraft::*};
use std::sync::Arc;
use parking_lot::Mutex;

/// Input handler for Android events
pub struct InputHandler {
    jni_bridge: Arc<crate::platform::callback::JniBridge>,
    raw_input: Arc<Mutex<egui::RawInput>>,
    touch_state: Mutex<HashMap<i32, TouchState>>,
    key_modifiers: Mutex<KeyModifiers>,
}

#[derive(Debug, Clone, Default)]
struct TouchState {
    id: i32,
    x: f32,
    y: f32,
    pressure: f32,
    phase: TouchPhase,
}

#[derive(Debug, Clone, Default)]
struct KeyModifiers {
    shift: bool,
    ctrl: bool,
    alt: bool,
    meta: bool,
}

impl InputHandler {
    pub fn new(jni_bridge: Arc<crate::platform::callback::JniBridge>) -> Self {
        Self {
            jni_bridge,
            raw_input: Arc::new(Mutex::new(egui::RawInput::default())),
            touch_state: Mutex::new(HashMap::new()),
            key_modifiers: Mutex::new(KeyModifiers::default()),
        }
    }

    /// Get the raw input for egui
    pub fn take_raw_input(&self) -> egui::RawInput {
        self.raw_input.lock().take()
    }

    /// Handle Android input event
    pub fn handle_input(&self, event: android_activity::InputEvent) {
        use android_activity::InputEvent;
        
        match event {
            InputEvent::Motion(motion_event) => self.handle_motion(motion_event),
            InputEvent::Key(key_event) => self.handle_key(key_event),
            _ => {}
        }
    }

    fn handle_motion(&self, event: android_activity::MotionEvent) {
        use android_activity::MotionEvent;
        
        match event {
            MotionEvent::TouchDown { id, x, y, pressure, .. } => {
                self.handle_touch_down(id, x, y, pressure);
            }
            MotionEvent::TouchUp { id, x, y, pressure, .. } => {
                self.handle_touch_up(id, x, y, pressure);
            }
            MotionEvent::TouchMove { id, x, y, pressure, .. } => {
                self.handle_touch_move(id, x, y, pressure);
            }
            MotionEvent::TouchCancel { id, .. } => {
                self.handle_touch_cancel(id);
            }
            MotionEvent::MouseMove { x, y, .. } => {
                self.handle_mouse_move(x, y);
            }
            MotionEvent::MouseButton { button, state, x, y, .. } => {
                self.handle_mouse_button(button, state, x, y);
            }
            MotionEvent::Scroll { delta_x, delta_y, .. } => {
                self.handle_scroll(delta_x, delta_y);
            }
            _ => {}
        }
    }

    fn handle_touch_down(&self, id: i32, x: f32, y: f32, pressure: f32) {
        // Update touch state
        self.touch_state.lock().insert(id, TouchState {
            id,
            x,
            y,
            pressure,
            phase: TouchPhase::Started,
        });

        // Add to egui raw input
        let mut raw = self.raw_input.lock();
        raw.events.push(egui::Event::PointerButton {
            pos: egui::pos2(x, y),
            button: egui::PointerButton::Primary,
            down: true,
            modifiers: self.get_modifiers(),
        });
    }

    fn handle_touch_up(&self, id: i32, x: f32, y: f32, pressure: f32) {
        // Update touch state
        if let Some(touch) = self.touch_state.lock().get_mut(&id) {
            touch.phase = TouchPhase::Ended;
            touch.x = x;
            touch.y = y;
        }

        // Add to egui raw input
        let mut raw = self.raw_input.lock();
        raw.events.push(egui::Event::PointerButton {
            pos: egui::pos2(x, y),
            button: egui::PointerButton::Primary,
            down: false,
            modifiers: self.get_modifiers(),
        });

        self.touch_state.lock().remove(&id);
    }

    fn handle_touch_move(&self, id: i32, x: f32, y: f32, pressure: f32) {
        // Update touch state
        if let Some(touch) = self.touch_state.lock().get_mut(&id) {
            touch.x = x;
            touch.y = y;
            touch.pressure = pressure;
            touch.phase = TouchPhase::Moved;
        }

        // Add to egui raw input
        let mut raw = self.raw_input.lock();
        raw.events.push(egui::Event::PointerMoved(egui::pos2(x, y)));
    }

    fn handle_touch_cancel(&self, id: i32) {
        if let Some(touch) = self.touch_state.lock().get_mut(&id) {
            touch.phase = TouchPhase::Cancelled;
        }
        self.touch_state.lock().remove(&id);
    }

    fn handle_mouse_move(&self, x: f32, y: f32) {
        let mut raw = self.raw_input.lock();
        raw.events.push(egui::Event::PointerMoved(egui::pos2(x, y)));
    }

    fn handle_mouse_button(&self, button: android_activity::MouseButton, state: android_activity::ElementState, x: f32, y: f32) {
        let egui_button = match button {
            android_activity::MouseButton::Left => egui::PointerButton::Primary,
            android_activity::MouseButton::Right => egui::PointerButton::Secondary,
            android_activity::MouseButton::Middle => egui::PointerButton::Middle,
            android_activity::MouseButton::Back => egui::PointerButton::Extra1,
            android_activity::MouseButton::Forward => egui::PointerButton::Extra2,
            _ => egui::PointerButton::Primary,
        };

        let down = matches!(state, android_activity::ElementState::Down);

        let mut raw = self.raw_input.lock();
        raw.events.push(egui::Event::PointerButton {
            pos: egui::pos2(x, y),
            button: egui_button,
            down,
            modifiers: self.get_modifiers(),
        });
    }

    fn handle_scroll(&self, delta_x: f32, delta_y: f32) {
        let mut raw = self.raw_input.lock();
        raw.events.push(egui::Event::Scroll(egui::vec2(delta_x, delta_y)));
    }

    fn handle_key(&self, event: android_activity::KeyEvent) {
        use android_activity::KeyEvent;
        
        match event {
            KeyEvent::KeyDown { key_code, repeat, .. } => {
                self.handle_key_down(key_code, repeat);
            }
            KeyEvent::KeyUp { key_code, .. } => {
                self.handle_key_up(key_code);
            }
            _ => {}
        }
    }

    fn handle_key_down(&self, android_keycode: i32, repeat: bool) {
        // Update modifiers
        self.update_modifiers(android_keycode, true);

        // Convert to egui key
        if let Some(egui_key) = self.android_keycode_to_egui(android_keycode) {
            let mut raw = self.raw_input.lock();
            raw.events.push(egui::Event::Key {
                key: egui_key,
                physical_key: None,
                pressed: true,
                repeat,
                modifiers: self.get_modifiers(),
            });
        }

        // Also convert to Minecraft keycode for game input
        let mc_key = android_to_minecraft(android_keycode);
        if mc_key != minecraft::KEY_UNKNOWN {
            // Send to game input handler
            self.send_game_key(mc_key, true);
        }
    }

    fn handle_key_up(&self, android_keycode: i32) {
        // Update modifiers
        self.update_modifiers(android_keycode, false);

        // Convert to egui key
        if let Some(egui_key) = self.android_keycode_to_egui(android_keycode) {
            let mut raw = self.raw_input.lock();
            raw.events.push(egui::Event::Key {
                key: egui_key,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: self.get_modifiers(),
            });
        }

        // Also convert to Minecraft keycode for game input
        let mc_key = android_to_minecraft(android_keycode);
        if mc_key != minecraft::KEY_UNKNOWN {
            self.send_game_key(mc_key, false);
        }
    }

    fn update_modifiers(&self, android_keycode: i32, pressed: bool) {
        let mut modifiers = self.key_modifiers.lock();
        match android_keycode {
            KEYCODE_SHIFT_LEFT | KEYCODE_SHIFT_RIGHT => modifiers.shift = pressed,
            KEYCODE_CTRL_LEFT | KEYCODE_CTRL_RIGHT => modifiers.ctrl = pressed,
            KEYCODE_ALT_LEFT | KEYCODE_ALT_RIGHT => modifiers.alt = pressed,
            KEYCODE_META_LEFT | KEYCODE_META_RIGHT => modifiers.meta = pressed,
            _ => {}
        }
    }

    fn get_modifiers(&self) -> egui::Modifiers {
        let modifiers = self.key_modifiers.lock();
        egui::Modifiers {
            shift: modifiers.shift,
            ctrl: modifiers.ctrl,
            alt: modifiers.alt,
            mac_cmd: modifiers.meta,
            command: modifiers.ctrl, // On Android, Ctrl = Command
            ..Default::default()
        }
    }

    fn android_keycode_to_egui(&self, android_keycode: i32) -> Option<egui::Key> {
        use egui::Key;
        
        Some(match android_keycode {
            KEYCODE_A => Key::A,
            KEYCODE_B => Key::B,
            KEYCODE_C => Key::C,
            KEYCODE_D => Key::D,
            KEYCODE_E => Key::E,
            KEYCODE_F => Key::F,
            KEYCODE_G => Key::G,
            KEYCODE_H => Key::H,
            KEYCODE_I => Key::I,
            KEYCODE_J => Key::J,
            KEYCODE_K => Key::K,
            KEYCODE_L => Key::L,
            KEYCODE_M => Key::M,
            KEYCODE_N => Key::N,
            KEYCODE_O => Key::O,
            KEYCODE_P => Key::P,
            KEYCODE_Q => Key::Q,
            KEYCODE_R => Key::R,
            KEYCODE_S => Key::S,
            KEYCODE_T => Key::T,
            KEYCODE_U => Key::U,
            KEYCODE_V => Key::V,
            KEYCODE_W => Key::W,
            KEYCODE_X => Key::X,
            KEYCODE_Y => Key::Y,
            KEYCODE_Z => Key::Z,
            KEYCODE_0 => Key::Num0,
            KEYCODE_1 => Key::Num1,
            KEYCODE_2 => Key::Num2,
            KEYCODE_3 => Key::Num3,
            KEYCODE_4 => Key::Num4,
            KEYCODE_5 => Key::Num5,
            KEYCODE_6 => Key::Num6,
            KEYCODE_7 => Key::Num7,
            KEYCODE_8 => Key::Num8,
            KEYCODE_9 => Key::Num9,
            KEYCODE_ENTER => Key::Enter,
            KEYCODE_ESCAPE => Key::Escape,
            KEYCODE_TAB => Key::Tab,
            KEYCODE_DEL => Key::Backspace,
            KEYCODE_FORWARD_DEL => Key::Delete,
            KEYCODE_DPAD_UP => Key::ArrowUp,
            KEYCODE_DPAD_DOWN => Key::ArrowDown,
            KEYCODE_DPAD_LEFT => Key::ArrowLeft,
            KEYCODE_DPAD_RIGHT => Key::ArrowRight,
            KEYCODE_DPAD_CENTER => Key::Enter,
            KEYCODE_SHIFT_LEFT => Key::Shift,
            KEYCODE_SHIFT_RIGHT => Key::Shift,
            KEYCODE_CTRL_LEFT => Key::Control,
            KEYCODE_CTRL_RIGHT => Key::Control,
            KEYCODE_ALT_LEFT => Key::Alt,
            KEYCODE_ALT_RIGHT => Key::Alt,
            KEYCODE_META_LEFT => Key::Meta,
            KEYCODE_META_RIGHT => Key::Meta,
            KEYCODE_SPACE => Key::Space,
            KEYCODE_CAPS_LOCK => Key::CapsLock,
            KEYCODE_NUM_LOCK => Key::NumLock,
            KEYCODE_SCROLL_LOCK => Key::ScrollLock,
            KEYCODE_F1 => Key::F1,
            KEYCODE_F2 => Key::F2,
            KEYCODE_F3 => Key::F3,
            KEYCODE_F4 => Key::F4,
            KEYCODE_F5 => Key::F5,
            KEYCODE_F6 => Key::F6,
            KEYCODE_F7 => Key::F7,
            KEYCODE_F8 => Key::F8,
            KEYCODE_F9 => Key::F9,
            KEYCODE_F10 => Key::F10,
            KEYCODE_F11 => Key::F11,
            KEYCODE_F12 => Key::F12,
            KEYCODE_COMMA => Key::Comma,
            KEYCODE_PERIOD => Key::Period,
            KEYCODE_MINUS => Key::Minus,
            KEYCODE_EQUALS => Key::Equals,
            KEYCODE_LEFT_BRACKET => Key::OpenBracket,
            KEYCODE_RIGHT_BRACKET => Key::CloseBracket,
            KEYCODE_BACKSLASH => Key::Backslash,
            KEYCODE_SEMICOLON => Key::Semicolon,
            KEYCODE_APOSTROPHE => Key::Quote,
            KEYCODE_GRAVE => Key::Backquote,
            KEYCODE_SLASH => Key::Slash,
            _ => return None,
        })
    }

    fn send_game_key(&self, mc_key: i32, pressed: bool) {
        // Send to game input handler (would be implemented via JNI or channel)
        // For now, just log
        log::debug!("Game key: {} pressed={}", mc_key, pressed);
    }
}

use std::collections::HashMap;