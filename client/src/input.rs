// Input handling. Tracks keyboard/mouse state per frame.

use winit::event::{WindowEvent, ElementState, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};
use std::collections::HashSet;

/// Tracks current input state.
pub struct InputState {
    /// Currently pressed keys.
    pub keys_down: HashSet<KeyCode>,
    /// Keys pressed this frame (rising edge).
    pub keys_pressed: HashSet<KeyCode>,
    /// Mouse buttons currently held.
    pub mouse_down: HashSet<MouseButton>,
    /// Mouse buttons clicked this frame.
    pub mouse_clicked: HashSet<MouseButton>,
    /// Mouse position in pixels.
    pub mouse_x: f64,
    pub mouse_y: f64,
    /// Mouse movement delta this frame.
    pub mouse_dx: f64,
    pub mouse_dy: f64,
    /// Scroll wheel delta this frame.
    pub scroll_delta: f32,
    /// Whether mouse is captured (in-game cursor lock).
    pub mouse_captured: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            mouse_down: HashSet::new(),
            mouse_clicked: HashSet::new(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_dx: 0.0,
            mouse_dy: 0.0,
            scroll_delta: 0.0,
            mouse_captured: false,
        }
    }

    /// Call at the start of each frame to clear per-frame state.
    pub fn begin_frame(&mut self) {
        self.keys_pressed.clear();
        self.mouse_clicked.clear();
        self.mouse_dx = 0.0;
        self.mouse_dy = 0.0;
        self.scroll_delta = 0.0;
    }

    /// Process a winit window event.
    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            if !self.keys_down.contains(&key) {
                                self.keys_pressed.insert(key);
                            }
                            self.keys_down.insert(key);
                        }
                        ElementState::Released => {
                            self.keys_down.remove(&key);
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed => {
                        self.mouse_clicked.insert(*button);
                        self.mouse_down.insert(*button);
                    }
                    ElementState::Released => {
                        self.mouse_down.remove(button);
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_dx += position.x - self.mouse_x;
                self.mouse_dy += position.y - self.mouse_y;
                self.mouse_x = position.x;
                self.mouse_y = position.y;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        self.scroll_delta += y;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(p) => {
                        self.scroll_delta += p.y as f32 / 100.0;
                    }
                }
            }
            _ => {}
        }
    }

    // ── Convenience methods ─────────────────────────────────────────────────

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn is_forward(&self) -> bool { self.is_key_down(KeyCode::KeyW) }
    pub fn is_backward(&self) -> bool { self.is_key_down(KeyCode::KeyS) }
    pub fn is_left(&self) -> bool { self.is_key_down(KeyCode::KeyA) }
    pub fn is_right(&self) -> bool { self.is_key_down(KeyCode::KeyD) }
    pub fn is_jump(&self) -> bool { self.is_key_pressed(KeyCode::Space) }
    pub fn is_sneak(&self) -> bool { self.is_key_down(KeyCode::ShiftLeft) }
    pub fn is_sprint(&self) -> bool { self.is_key_down(KeyCode::ShiftLeft) }

    /// Get hotbar slot from number keys (1-9 -> 0-8), or None.
    pub fn hotbar_key_pressed(&self) -> Option<u8> {
        for (i, key) in [
            KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3,
            KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6,
            KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9,
        ].iter().enumerate() {
            if self.is_key_pressed(*key) {
                return Some(i as u8);
            }
        }
        None
    }

    pub fn is_inventory_pressed(&self) -> bool { self.is_key_pressed(KeyCode::KeyE) }
    pub fn is_chat_pressed(&self) -> bool { self.is_key_pressed(KeyCode::KeyT) }
    pub fn is_escape_pressed(&self) -> bool { self.is_key_pressed(KeyCode::Escape) }
    pub fn is_debug_pressed(&self) -> bool { self.is_key_pressed(KeyCode::F3) }
    pub fn is_drop_pressed(&self) -> bool { self.is_key_pressed(KeyCode::KeyQ) }
}
