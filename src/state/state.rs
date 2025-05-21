#[derive(Clone, Copy)]
struct Vec2 {
    x: f32,
    y: f32,
}

pub struct PlayerState {
    pos: Vec2,
    vel: Vec2,
    acc: f32,
    jump_force: f32,
    jump_timer: f32,
    color: Color,
    size: f32,
}

pub struct GameState {
    players: Vec<PlayerState>,
}

impl GameState {
    pub fn mutate(&mut self, commands[])
}