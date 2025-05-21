use super::model::*;
use crate::game_state_generated::Color;

use super::{JUMP_CD, SCREEN_HEIGHT};

impl PlayerState {
    fn new(id: u32) -> PlayerState {
        PlayerState {
            id,
            pos: Vec2::zero(),
            vel: Vec2::zero(),
            acc: 0.75,
            jump_force: 400.0,
            jump_timer: 0.0,
            color: Color::Red,
            size: 16.0,
        }
    }
}

pub struct GameState {
    players: Vec<PlayerState>,
}

impl GameState {
    pub fn new() -> GameState {
        GameState { players: vec![] }
    }

    pub fn mutate(&mut self) {
        println!("Mutating!")
    }
}

impl PlayerState {
    fn handle_move_right(&mut self) {
        self.vel.x += self.acc;
    }

    fn handle_move_left(&mut self) {
        self.vel.x -= self.acc;
    }

    fn handle_jump(&mut self) {
        if self.pos.y >= SCREEN_HEIGHT as f32 - self.size && self.jump_timer > JUMP_CD {
            self.vel.y -= self.jump_force;
            self.jump_timer = 0.0;
        };
    }
}

impl GameState {
    fn serialize(&self) {
        
    }
}