use std::time::{SystemTime, UNIX_EPOCH};

use super::{CommandContent, GameState, PlayerState, physics::*};
use crate::generated::PlayerCommand;

use super::{JUMP_CD, JUMP_FORCE, PLAYER_ACCELERATION};

#[derive(Debug, Clone)]
pub struct ScheduledCommand {
    execute_at_timestamp: u64,
    player_id: u32,
    client_dt: f32,
    command: PlayerCommand,
}

impl Eq for ScheduledCommand {}

impl PartialEq for ScheduledCommand {
    fn eq(&self, other: &Self) -> bool {
        self.execute_at_timestamp == other.execute_at_timestamp
    }
}

impl PartialOrd for ScheduledCommand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledCommand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.execute_at_timestamp.cmp(&self.execute_at_timestamp)
    }
}

const FIXED_DT_MICROS: u64 = 16000; // 16 ms

impl GameState {
    pub fn mutate(
        &mut self,
        commands: &[CommandContent],
        dt_micros: u64,
        client_player_id: Option<u32>,
    ) {
        let dt_micros = self.cached_dt_micros + dt_micros;
        let end_tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        for mutate_command in commands {
            let client_dt = (mutate_command.player_state_command.dt_micros / 1000) as f32;
            // Get player, add to game state if not exists

            // Add commands to scheduled binary heap
            for command in &mutate_command.player_state_command.commands {
                let scheduled_command = ScheduledCommand {
                    execute_at_timestamp: mutate_command
                        .player_state_command
                        .client_timestamp_micros
                        + mutate_command.client_delay_micros,
                    player_id: mutate_command.player_id,
                    client_dt,
                    command: *command,
                };

                self.scheduled_commands.push(scheduled_command);
            }
        }

        // Physics
        let mut accumulator = dt_micros;

        while accumulator >= FIXED_DT_MICROS {
            let step = FIXED_DT_MICROS;
            let execute_time = end_tick - accumulator;

            self.execute_commands(execute_time);

            let dt = step as f32 / 1000000.0;
            physics(self, dt, client_player_id);

            accumulator -= step;
        }

        // Execute remaining commands
        while let Some(command) = self.scheduled_commands.pop() {
            self.execute_scheduled_command(command);
        }

        self.cached_dt_micros = accumulator;

        // Collision
        // let player_forces = collision(players);
        // for (i, force, pos) in player_forces {
        //     players[i].vel.x = force.x;
        //     players[i].vel.y = force.y;
        //     players[i].pos.x = pos.x;
        //     players[i].pos.y = pos.y;
        // }
    }

    fn execute_commands(&mut self, execute_time: u64) {
        while let Some(scheduled_command) = self.scheduled_commands.peek() {
            if scheduled_command.execute_at_timestamp <= execute_time {
                if let Some(scheduled_command) = self.scheduled_commands.pop() {
                    self.execute_scheduled_command(scheduled_command);
                }
            } else {
                break;
            }
        }
    }

    fn execute_scheduled_command(&mut self, scheduled: ScheduledCommand) {
        let player_id = scheduled.player_id;
        let player = match self.players.get_mut(&player_id) {
            Some(player) => player,
            None => {
                println!("Player {} not found, creating player", player_id);
                let new_player = PlayerState::new(player_id, &self.spawn_point);
                self.players.insert(player_id, new_player);
                self.players.get_mut(&player_id).unwrap()
            }
        };

        match scheduled.command {
            PlayerCommand::MoveRight => player.handle_move_right(scheduled.client_dt),
            PlayerCommand::MoveLeft => player.handle_move_left(scheduled.client_dt),
            PlayerCommand::Jump => player.handle_jump(),
            _ => {}
        }
    }

    pub fn clear_cache(&mut self) {
        self.cached_dt_micros = 0;
        self.scheduled_commands.clear();
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
