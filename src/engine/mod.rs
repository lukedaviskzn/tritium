mod colour;

pub use colour::*;

use crate::{resource::Handle, input::{KeyboardManager, MouseManager}};

pub struct FrameCounter {
    pub current_frame: usize,
    pub last_frame_time: std::time::Instant,

    pub current_tick: usize,
    pub last_tick_time: std::time::Instant,

    pub accumulator: std::time::Duration,
}

impl FrameCounter {
    pub fn new() -> FrameCounter {
        FrameCounter {
            current_frame: 0,
            last_frame_time: std::time::Instant::now(),

            current_tick: 0,
            last_tick_time: std::time::Instant::now(),

            accumulator: std::time::Duration::ZERO,
        }
    }
}

pub struct UpdateContext {
    pub window_size: glam::Vec2,
    pub window_focused: bool,
    pub delta_time: f32,
    pub keyboard: Handle<KeyboardManager>,
    pub mouse: Handle<MouseManager>,
}
