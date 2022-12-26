use std::collections::HashMap;

use winit::{event::{ElementState, MouseButton, MouseScrollDelta}, dpi::PhysicalPosition};

use super::InputState;

pub struct MouseManager {
    state: HashMap<MouseButton, InputState>,
    position: glam::Vec2,
    delta: glam::Vec2,
    scroll_delta: glam::Vec2,
}

impl MouseManager {
    pub const LINE_HEIGHT: f32 = 16.0;
    
    pub fn new() -> MouseManager {
        MouseManager {
            state: hashmap! {},
            position: glam::Vec2::ZERO,
            delta: glam::Vec2::ZERO,
            scroll_delta: glam::Vec2::ZERO,
        }
    }

    pub fn reset_input(&mut self) {
        for (_, state) in self.state.iter_mut() {
            state.just_changed = false;
        }
        self.delta = glam::Vec2::ZERO;
        self.scroll_delta = glam::Vec2::ZERO;
    }
    
    pub fn input(&mut self, state: ElementState, button: MouseButton) -> bool {
        let mut new_state = if state == ElementState::Pressed {
            InputState::PRESSED
        } else {
            InputState::RELEASED
        };
        
        if let Some(old_state) = self.state.get(&button) {
            if old_state.state != new_state.state {
                new_state.just_changed = true;
            }
        }

        self.state.insert(button, new_state);

        false
    }

    pub fn update_position(&mut self, position: glam::Vec2) {
        self.position = position;
    }

    pub fn update_delta(&mut self, delta: glam::Vec2) {
        self.delta += delta;
    }

    pub fn update_scroll_delta(&mut self, delta: MouseScrollDelta) {
        self.scroll_delta += match delta {
            MouseScrollDelta::LineDelta(x, y) => glam::vec2(x, y) * MouseManager::LINE_HEIGHT,
            MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => glam::vec2(x as f32, y as f32),
        };
    }

    pub fn key_state(&self, button: MouseButton) -> InputState {
        if let Some(state) = self.state.get(&button) {
            *state
        } else {
            InputState::RELEASED
        }
    }

    pub fn key_pressed(&self, button: MouseButton) -> bool {
        self.key_state(button).pressed()
    }

    pub fn key_released(&self, button: MouseButton) -> bool {
        self.key_state(button).released()
    }

    pub fn key_just_pressed(&self, button: MouseButton) -> bool {
        self.key_state(button).just_pressed()
    }

    pub fn key_just_released(&self, button: MouseButton) -> bool {
        self.key_state(button).just_released()
    }

    pub fn position(&self) -> glam::Vec2 {
        self.position
    }

    /// Mouse delta in pixels
    pub fn motion(&self) -> glam::Vec2 {
        self.delta
    }

    /// In pixels, line deltas converted to pixel deltas using the LINE_HEIGHT constant.
    pub fn scroll_delta(&self) -> glam::Vec2 {
        self.scroll_delta
    }
}
