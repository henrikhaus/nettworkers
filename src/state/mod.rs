mod model;
mod physics;
mod state;

pub use model::*;
pub use state::*;

pub const JUMP_CD: f32 = 0.3;
pub const SCREEN_HEIGHT: usize = 360;
pub const SCREEN_WIDTH: usize = 640;
