use std::{
    collections::BinaryHeap,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use super::{CommandContent, GameState, PlayerState, physics::*};
use crate::generated::PlayerCommand;

use super::{JUMP_CD, JUMP_FORCE, PLAYER_ACCELERATION};

struct ScheduledCommand {
    execute_at: u64,
    player_id: u32,
    client_dt: f32,
    command: PlayerCommand,
}

impl Eq for ScheduledCommand {}

impl PartialEq for ScheduledCommand {
    fn eq(&self, other: &Self) -> bool {
        self.execute_at == other.execute_at
    }
}

impl PartialOrd for ScheduledCommand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledCommand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.execute_at.cmp(&self.execute_at)
    }
}

impl GameState {
    pub fn mutate(&mut self, commands: &[CommandContent], tick_time_micro: u64) {
        let end_tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        let start_tick = end_tick - tick_time_micro;

        let mut scheduled_commands = BinaryHeap::new();

        for (player_id, player_state_command, client_delay_micros) in commands {
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

            // Add commands to scheduled binary heap
            for command in &player_state_command.commands {
                let effect = ScheduledCommand {
                    execute_at: start_tick
                        .max(player_state_command.client_timestamp_micro + *client_delay_micros)
                        - start_tick,
                    player_id: *player_id,
                    client_dt,
                    command: *command,
                };

                // REMEBER TO ADD EFFECT TO QUEUE, MAKE IT MIN by exectue_at
                // ALSO EXECUTE ALL EFFECT THAT HAVE TIME LESS THAN START TICK
                // AND EXECUTE ALL EFFECTS THAT HAVE TIME MORE THAN END TICK AT THE END OF PHYSICS
                scheduled_commands.push(effect);
            }
        }

        // Physics
        let mut accumulator = tick_time_micro;
        let fixed_dt = 16000; // 16 ms

        while accumulator > 0 {
            let step = accumulator.min(fixed_dt);

            let execute_time = if step == accumulator {
                u64::MAX
            } else {
                tick_time_micro - accumulator
            };

            self.execute_commands(&mut scheduled_commands, execute_time);

            let dt = step as f32 / 1000000.0;
            physics(self, dt);

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

    fn execute_commands(
        &mut self,
        scheduled_commands: &mut BinaryHeap<ScheduledCommand>,
        execute_time: u64,
    ) {
        while let Some(scheduled_command) = scheduled_commands.pop() {
            if scheduled_command.execute_at <= execute_time {
                println!("Command executed at {}", execute_time);
                self.execute_scheduled_command(scheduled_command);
            } else {
                scheduled_commands.push(scheduled_command);
                break;
            }
        }
    }

    fn execute_scheduled_command(&mut self, scheduled: ScheduledCommand) {
        if let Some(player) = self.players.get_mut(&scheduled.player_id) {
            match scheduled.command {
                PlayerCommand::MoveRight => player.handle_move_right(scheduled.client_dt),
                PlayerCommand::MoveLeft => player.handle_move_left(scheduled.client_dt),
                PlayerCommand::Jump => player.handle_jump(),
                _ => {}
            }
        }
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
