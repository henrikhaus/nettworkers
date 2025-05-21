mod model;
mod physics;
mod state;

pub use model::*;
pub use state::*;
pub use physics::*;

// Settings
pub const SCREEN_HEIGHT: usize = 360;
pub const SCREEN_WIDTH: usize = 640;

// Player
pub const JUMP_CD: f32 = 0.3;

// Physics
pub const GROUND_FRICTION: f32 = 0.8;
pub const AIR_FRICTION: f32 = 0.95;
pub const GRAVITY: f32 = 1000.0;
