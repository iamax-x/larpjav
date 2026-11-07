//! Game Input Processors — Port of InGameEventProcessor, InGUIEventProcessor, TouchEventProcessor

use crate::value::{KeyCode, MouseButton, GamepadButton, GamepadAxis};
use std::collections::HashSet;

/// In-Game Event Processor — Port of InGameEventProcessor.java
pub struct InGameEventProcessor {
    keys_pressed: HashSet<KeyCode>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_position: (f32, f32),
    mouse_delta: (f32, f32),
    gamepad_buttons: HashSet<(u32, GamepadButton)>, // (gamepad_id, button)
    gamepad_axes: HashMap<(u32, GamepadAxis), f32>, // (gamepad_id, axis) -> value
    touch_points: HashMap<u32, TouchPoint>, // pointer_id -> TouchPoint
}

#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
}

impl InGameEventProcessor {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_position: (0.0, 0.0),
            mouse_delta: (0.0, 0.0),
            gamepad_buttons: HashSet::new(),
            gamepad_axes: HashMap::new(),
            touch_points: HashMap::new(),
        }
    }

    pub fn handle_key_event(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            self.keys_pressed.insert(key);
        } else {
            self.keys_pressed.remove(&key);
        }
    }

    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        self.mouse_delta = (x - self.mouse_position.0, y - self.mouse_position.1);
        self.mouse_position = (x, y);
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        if pressed {
            self.mouse_buttons_pressed.insert(button);
        } else {
            self.mouse_buttons_pressed.remove(&button);
        }
    }

    pub fn handle_mouse_scroll(&mut self, delta_x: f32, delta_y: f32) {
        // Handle scroll
    }

    pub fn handle_gamepad_button(&mut self, gamepad_id: u32, button: GamepadButton, pressed: bool) {
        if pressed {
            self.gamepad_buttons.insert((gamepad_id, button));
        } else {
            self.gamepad_buttons.remove(&(gamepad_id, button));
        }
    }

    pub fn handle_gamepad_axis(&mut self, gamepad_id: u32, axis: GamepadAxis, value: f32) {
        self.gamepad_axes.insert((gamepad_id, axis), value);
    }

    pub fn handle_touch_down(&mut self, id: u32, x: f32, y: f32, pressure: f32) {
        self.touch_points.insert(id, TouchPoint { id, x, y, pressure });
    }

    pub fn handle_touch_move(&mut self, id: u32, x: f32, y: f32, pressure: f32) {
        if let Some(point) = self.touch_points.get_mut(&id) {
            point.x = x;
            point.y = y;
            point.pressure = pressure;
        }
    }

    pub fn handle_touch_up(&mut self, id: u32) {
        self.touch_points.remove(&id);
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    pub fn get_mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }

    pub fn get_mouse_delta(&self) -> (f32, f32) {
        self.mouse_delta
    }

    pub fn is_gamepad_button_pressed(&self, gamepad_id: u32, button: GamepadButton) -> bool {
        self.gamepad_buttons.contains(&(gamepad_id, button))
    }

    pub fn get_gamepad_axis(&self, gamepad_id: u32, axis: GamepadAxis) -> f32 {
        self.gamepad_axes.get(&(gamepad_id, axis)).copied().unwrap_or(0.0)
    }

    pub fn get_touch_points(&self) -> Vec<TouchPoint> {
        self.touch_points.values().copied().collect()
    }

    pub fn clear_deltas(&mut self) {
        self.mouse_delta = (0.0, 0.0);
    }
}

impl Default for InGameEventProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// In-GUI Event Processor — Port of InGUIEventProcessor.java
/// Handles input when GUI is open (inventory, menus, etc.)
pub struct InGUIEventProcessor {
    keys_pressed: HashSet<KeyCode>,
    mouse_position: (f32, f32),
    mouse_buttons: HashSet<MouseButton>,
    scroll_delta: (f32, f32),
    consumed_keys: HashSet<KeyCode>,
    consumed_mouse_buttons: HashSet<MouseButton>,
}

impl InGUIEventProcessor {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            mouse_position: (0.0, 0.0),
            mouse_buttons: HashSet::new(),
            scroll_delta: (0.0, 0.0),
            consumed_keys: HashSet::new(),
            consumed_mouse_buttons: HashSet::new(),
        }
    }

    pub fn handle_key_event(&mut self, key: KeyCode, pressed: bool) -> bool {
        if pressed {
            self.keys_pressed.insert(key);
        } else {
            self.keys_pressed.remove(&key);
        }
        // Return true if event was consumed by GUI
        self.consumed_keys.contains(&key)
    }

    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        self.mouse_position = (x, y);
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, pressed: bool) -> bool {
        if pressed {
            self.mouse_buttons.insert(button);
        } else {
            self.mouse_buttons.remove(&button);
        }
        self.consumed_mouse_buttons.contains(&button)
    }

    pub fn handle_scroll(&mut self, delta_x: f32, delta_y: f32) {
        self.scroll_delta = (delta_x, delta_y);
    }

    pub fn consume_key(&mut self, key: KeyCode) {
        self.consumed_keys.insert(key);
    }

    pub fn consume_mouse_button(&mut self, button: MouseButton) {
        self.consumed_mouse_buttons.insert(button);
    }

    pub fn release_consumed(&mut self) {
        self.consumed_keys.clear();
        self.consumed_mouse_buttons.clear();
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn get_mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }

    pub fn get_scroll_delta(&self) -> (f32, f32) {
        self.scroll_delta
    }
}

impl Default for InGUIEventProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Touch Event Processor — Port of TouchEventProcessor.java
/// Handles multi-touch gestures for mobile
pub struct TouchEventProcessor {
    touches: HashMap<u32, TouchPoint>,
    gestures: Vec<TouchGesture>,
    gesture_detector: GestureDetector,
}

#[derive(Debug, Clone)]
pub enum TouchGesture {
    Tap { x: f32, y: f32 },
    DoubleTap { x: f32, y: f32 },
    LongPress { x: f32, y: f32 },
    Pan { delta_x: f32, delta_y: f32 },
    Pinch { scale: f32, center_x: f32, center_y: f32 },
    Rotate { angle: f32, center_x: f32, center_y: f32 },
    Swipe { direction: SwipeDirection, velocity: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

struct GestureDetector {
    last_tap_time: Option<std::time::Instant>,
    last_tap_position: Option<(f32, f32)>,
    long_press_timer: Option<std::time::Instant>,
    initial_pinch_distance: Option<f32>,
    initial_pinch_center: Option<(f32, f32)>,
}

impl TouchEventProcessor {
    pub fn new() -> Self {
        Self {
            touches: HashMap::new(),
            gestures: Vec::new(),
            gesture_detector: GestureDetector::new(),
        }
    }

    pub fn process_touch_down(&mut self, id: u32, x: f32, y: f32, pressure: f32, timestamp: std::time::Instant) {
        self.touches.insert(id, TouchPoint { id, x, y, pressure });
        self.gesture_detector.handle_touch_down(id, x, y, timestamp);
    }

    pub fn process_touch_move(&mut self, id: u32, x: f32, y: f32, pressure: f32) {
        if let Some(point) = self.touches.get_mut(&id) {
            point.x = x;
            point.y = y;
            point.pressure = pressure;
        }
        self.gesture_detector.handle_touch_move(&self.touches);
    }

    pub fn process_touch_up(&mut self, id: u32, timestamp: std::time::Instant) {
        self.gesture_detector.handle_touch_up(id, timestamp, &mut self.gestures);
        self.touches.remove(&id);
    }

    pub fn take_gestures(&mut self) -> Vec<TouchGesture> {
        std::mem::take(&mut self.gestures)
    }

    pub fn get_active_touches(&self) -> Vec<TouchPoint> {
        self.touches.values().cloned().collect()
    }
}

impl Default for TouchEventProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureDetector {
    fn new() -> Self {
        Self {
            last_tap_time: None,
            last_tap_position: None,
            long_press_timer: None,
            initial_pinch_distance: None,
            initial_pinch_center: None,
        }
    }

    fn handle_touch_down(&mut self, id: u32, x: f32, y: f32, timestamp: std::time::Instant) {
        // Check for double tap
        if let (Some(last_time), Some(last_pos)) = (self.last_tap_time, self.last_tap_position) {
            let elapsed = timestamp.duration_since(last_time);
            let distance = ((x - last_pos.0).powi(2) + (y - last_pos.1).powi(2)).sqrt();
            if elapsed < std::time::Duration::from_millis(300) && distance < 50.0 {
                // Double tap detected
                self.last_tap_time = None;
                return;
            }
        }
        self.last_tap_time = Some(timestamp);
        self.last_tap_position = Some((x, y));
        self.long_press_timer = Some(timestamp);
    }

    fn handle_touch_move(&mut self, touches: &HashMap<u32, TouchPoint>) {
        self.long_press_timer = None; // Cancel long press on move
        
        // Check for pinch gesture (2 touches)
        if touches.len() == 2 {
            let points: Vec<_> = touches.values().collect();
            let dx = points[0].x - points[1].x;
            let dy = points[0].y - points[1].y;
            let distance = (dx.powi(2) + dy.powi(2)).sqrt();
            let center_x = (points[0].x + points[1].x) / 2.0;
            let center_y = (points[0].y + points[1].y) / 2.0;
            
            if let Some(initial) = self.initial_pinch_distance {
                let scale = distance / initial;
                // Pinch gesture would be emitted here
            } else {
                self.initial_pinch_distance = Some(distance);
                self.initial_pinch_center = Some((center_x, center_y));
            }
        } else {
            self.initial_pinch_distance = None;
            self.initial_pinch_center = None;
        }
    }

    fn handle_touch_up(&mut self, id: u32, timestamp: std::time::Instant, gestures: &mut Vec<TouchGesture>) {
        if let Some(timer) = self.long_press_timer {
            if timestamp.duration_since(timer) > std::time::Duration::from_millis(500) {
                // Long press
                if let Some(pos) = self.last_tap_position {
                    gestures.push(TouchGesture::LongPress { x: pos.0, y: pos.1 });
                }
            } else {
                // Single tap
                if let Some(pos) = self.last_tap_position {
                    gestures.push(TouchGesture::Tap { x: pos.0, y: pos.1 });
                }
            }
        }
        self.long_press_timer = None;
        
        // Check for swipe (single touch, fast movement)
        // Would need velocity tracking
    }
}