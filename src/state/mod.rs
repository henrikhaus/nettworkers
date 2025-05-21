mod connector;
mod mutate;
mod physics;

use std::collections::HashMap;

pub use mutate::*;

use crate::game_state_generated::Color;

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

#[derive(Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn zero() -> Vec2 {
        Vec2 { x: 0.0, y: 0.0 }
    }
}

#[derive(Clone, Copy)]
pub struct PlayerState {
    pub pos: Vec2,
    pub vel: Vec2,
    pub jump_timer: f32,
    pub color: Color,
    pub size: f32,
}

impl PlayerState {
    fn new(id: u32) -> PlayerState {
        PlayerState {
            id,
            pos: Vec2::zero(),
            vel: Vec2::zero(),
            jump_timer: 0.0,
            color: Color::Red,
            size: 16.0,
        }
    }
}

#[derive(Clone)]
pub struct GameState {
    pub players: HashMap<u32, PlayerState>,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            players: HashMap::new(),
        }
    }
}
