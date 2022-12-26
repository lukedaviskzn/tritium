use std::collections::HashMap;

use winit::event::{VirtualKeyCode, KeyboardInput, ElementState};

use super::InputState;

pub struct KeyboardManager {
    state: HashMap<VirtualKeyCode, InputState>,
}

impl KeyboardManager {
    pub fn new() -> KeyboardManager {
        KeyboardManager {
            state: hashmap! {},
        }
    }

    pub fn reset_input(&mut self) {
        for (_, state) in self.state.iter_mut() {
            state.just_changed = false;
        }
    }
    
    pub fn input(&mut self, input: &KeyboardInput) -> bool {
        if let Some(code) = input.virtual_keycode {
            let mut new_state = if input.state == ElementState::Pressed {
                InputState::PRESSED
            } else {
                InputState::RELEASED
            };
            
            if let Some(old_state) = self.state.get(&code) {
                if old_state.state != new_state.state {
                    new_state.just_changed = true;
                }
            }

            self.state.insert(code, new_state);
        }
        false
    }

    pub fn key_state(&self, key: VirtualKeyCode) -> InputState {
        if let Some(state) = self.state.get(&key) {
            *state
        } else {
            InputState::RELEASED
        }
    }

    pub fn key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.key_state(key).pressed()
    }

    pub fn key_released(&self, key: VirtualKeyCode) -> bool {
        self.key_state(key).released()
    }

    pub fn key_just_pressed(&self, key: VirtualKeyCode) -> bool {
        self.key_state(key).just_pressed()
    }

    pub fn key_just_released(&self, key: VirtualKeyCode) -> bool {
        self.key_state(key).just_released()
    }
}
