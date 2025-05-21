use super::model::*;
use super::physics::*;
use crate::{game_state_generated::Color, player_commands_generated::PlayerCommand};
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::fs::File;

use super::{JUMP_CD, SCREEN_HEIGHT};

impl PlayerState {
    fn new(id: u32) -> PlayerState {
        PlayerState {
            id,
            pos: Vec2::zero(),
            vel: Vec2::zero(),
            acc: 20.0,
            jump_force: 600.0,
            jump_timer: 0.0,
            color: Color::Red,
            size: 16.0,
        }
    }
}

#[derive(Clone)]
pub struct GameState {
    pub players: HashMap<u32, PlayerState>,
    pub collidables: Vec<SceneObject>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SceneObject {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Deserialize)]
struct Scene {
    collidables: HashMap<u32, SceneObject>,
}

impl GameState {
    pub fn new(scene_name: &str) -> GameState {
        let file = File::open(format!("src/scenes/{}.json", scene_name)).expect("Scene file must open");
        let scene: Scene = serde_json::from_reader(file).expect("JSON must match Scene");
        let collidables: Vec<SceneObject> = scene.collidables.into_values().collect();

        GameState {
            players: HashMap::new(),
            collidables,
        }
    }

    pub fn mutate(&mut self, command_queue: &VecDeque<(u32, PlayerCommand)>, dt: f32) {
        for (player_id, command) in command_queue {
            // Get player, add to game state if not exists
            let player = match self.players.get_mut(player_id) {
                Some(player) => player,
                None => {
                    let new_player = PlayerState::new(*player_id);
                    self.players.insert(*player_id, new_player);
                    self.players.get_mut(player_id).unwrap()
                }
            };

            // Execute command
            match *command {
                PlayerCommand::Move_right => player.handle_move_right(),
                PlayerCommand::Move_left => player.handle_move_left(),
                PlayerCommand::Jump => self.players.get_mut(player_id).unwrap().handle_jump(),
                _ => {}
            }
        }

        // Physics
        let mut accumulator = dt;
        let fixed_dt = 0.016; // 16 ms

        while accumulator > 0.0 {
            let step = accumulator.min(fixed_dt);
            physics(self, step);
            accumulator -= step;
        }

        // Collision
        // let player_forces = collision(players);
        // for (i, force, pos) in player_forces {
        //     players[i].vel.x = force.x;
        //     players[i].vel.y = force.y;
        //     players[i].pos.x = pos.x;
        //     players[i].pos.y = pos.y;
        // }
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
    fn serialize(&self) {}
}
