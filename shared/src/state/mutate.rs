use std::{
    collections::BinaryHeap,
    time::{Duration, Instant},
};

use super::{CommandContent, GameState, PlayerState, PlayerStateCommand, physics::*};
use crate::generated::PlayerCommand;

use super::{JUMP_CD, JUMP_FORCE, PLAYER_ACCELERATION};

struct EffectToExecute {
    execute_at: u64,
    effect: Box<dyn FnMut()>,
}

impl GameState {
    pub fn mutate(&mut self, commands: &[CommandContent], tick_micro: u64) {
        let end_tick = Instant::now();
        let start_tick = end_tick - Duration::from_micros(tick_micro);

        let mut when_to_execute = BinaryHeap::new();

        for (player_id, player_state_command, relative_delay) in commands {
            let client_dt = (player_state_command.dt_micro / 1000) as f32;
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
            for command in &player_state_command.commands {
                let effect = EffectToExecute {
                    execute_at: player_state_command.client_timestamp_micro + *relative_delay,
                    effect: Box::new(move || match *command {
                        PlayerCommand::Move_right => player.handle_move_right(client_dt),
                        PlayerCommand::Move_left => player.handle_move_left(client_dt),
                        PlayerCommand::Jump => player.handle_jump(),
                        _ => {}
                    }),
                };

                // REMEBER TO ADD EFFECT TO QUEUE, MAKE IT MIN by exectue_at
                // ALSO EXECUTE ALL EFFECT THAT HAVE TIME LESS THAN START TICK
                // AND EXECUTE ALL EFFECTS THAT HAVE TIME MORE THAN END TICK AT THE END OF PHYSICS
            }
        }

        // Physics
        let mut accumulator = tick_micro as f64 / 1000000.0;
        let fixed_dt = 0.016; // 16 ms

        while accumulator > 0.0 {
            let step = accumulator.min(fixed_dt);
            physics(self, step as f32);
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
    fn handle_move_right(&mut self, dt: f32) {
        self.vel.x += PLAYER_ACCELERATION * dt;
    }

    fn handle_move_left(&mut self, dt: f32) {
        self.vel.x -= PLAYER_ACCELERATION * dt;
    }

    fn handle_jump(&mut self) {
        if self.grounded && self.jump_timer > JUMP_CD {
            self.vel.y -= JUMP_FORCE;
            self.jump_timer = 0.0;
        };
    }
}
