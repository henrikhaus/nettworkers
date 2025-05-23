use super::{physics::*, GameState, PlayerState, PlayerStateCommand};
use crate::generated::PlayerCommand;

use super::{JUMP_CD, JUMP_FORCE, PLAYER_ACCELERATION};

impl GameState {
    pub fn mutate(&mut self, commands: &[(u32, PlayerStateCommand)], dt: f32) {
        for (player_id, player_state_command) in commands {
            println!("PlayerID: {}", player_id);
            // Get player, add to game state if not exists
            let player = match self.players.get_mut(player_id) {
                Some(player) => player,
                None => {
                    println!("Player {} not found, creating player", player_id);
                    let new_player = PlayerState::new(*player_id, &self.spawn_point);
                    self.players.insert(*player_id, new_player);
                    self.players.get_mut(player_id).unwrap()
                }
            };

            // Execute commands
            for command in player_state_command.commands {
                match *command {
                    PlayerCommand::Move_right => player.handle_move_right(),
                    PlayerCommand::Move_left => player.handle_move_left(),
                    PlayerCommand::Jump => self.players.get_mut(player_id).unwrap().handle_jump(),
                    _ => {}
                }
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
        self.vel.x += PLAYER_ACCELERATION;
    }

    fn handle_move_left(&mut self) {
        self.vel.x -= PLAYER_ACCELERATION;
    }

    fn handle_jump(&mut self) {
        if self.grounded && self.jump_timer > JUMP_CD {
            self.vel.y -= JUMP_FORCE;
            self.jump_timer = 0.0;
        };
    }
}
