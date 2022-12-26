mod keyboard;
mod mouse;

pub use keyboard::*;
pub use mouse::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputState {
    state: ButtonState,
    just_changed: bool,
}

impl InputState {
    pub const PRESSED: InputState = InputState {
        state: ButtonState::Pressed,
        just_changed: false,
    };

    pub const RELEASED: InputState = InputState {
        state: ButtonState::Released,
        just_changed: false,
    };

    pub const JUST_PRESSED: InputState = InputState {
        state: ButtonState::Pressed,
        just_changed: true,
    };

    pub const JUST_RELEASED: InputState = InputState {
        state: ButtonState::Released,
        just_changed: true,
    };

    pub fn state(&self) -> ButtonState {
        self.state
    }

    pub fn just_changed(&self) -> bool {
        self.just_changed
    }

    pub fn pressed(&self) -> bool {
        self.state == ButtonState::Pressed
    }

    pub fn released(&self) -> bool {
        self.state == ButtonState::Released
    }

    pub fn just_pressed(&self) -> bool {
        self.state == ButtonState::Pressed && self.just_changed
    }

    pub fn just_released(&self) -> bool {
        self.state == ButtonState::Released && self.just_changed
    }
}
