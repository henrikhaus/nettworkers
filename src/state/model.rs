use crate::game_state_generated::Color;

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

pub struct PlayerState {
    pub id: u32,
    pub pos: Vec2,
    pub vel: Vec2,
    pub acc: f32,
    pub jump_force: f32,
    pub jump_timer: f32,
    pub color: Color,
    pub size: f32,
}
