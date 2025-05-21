mod connector;
mod model;
mod physics;
mod state;

pub use connector::*;
pub use model::*;
pub use physics::*;
pub use state::*;

// Settings
pub const SCREEN_HEIGHT: usize = 360;
pub const SCREEN_WIDTH: usize = 640;

// Player
pub const JUMP_CD: f32 = 0.3;

// Physics
pub const GROUND_FRICTION: f32 = 0.0001;
pub const AIR_FRICTION: f32 = 0.9;
pub const GRAVITY: f32 = 2500.0;
pub const JUMP_FORCE: f32 = 600.0;
pub const PLAYER_ACCELERATION: f32 = 20.0;
