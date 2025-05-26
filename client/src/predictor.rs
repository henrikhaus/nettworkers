use std::time::Instant;

use shared::state::{CommandContent, GameState, PlayerStateCommand};

pub struct ReconciliationCommand {
    command: CommandContent,
    sequence: u32,
}

pub struct Predictor {
    reconciliation_commands: Vec<ReconciliationCommand>,
    pub active_prediction: bool,
    pub active_reconciliation: bool,
    pub sequence: u32,
}

impl Predictor {
    pub fn new() -> Self {
        Predictor {
            reconciliation_commands: Vec::new(),
            active_prediction: true,
            active_reconciliation: true,
            sequence: 0,
        }
    }

    pub fn predict(
        &mut self,
        game_state: &mut GameState,
        client_player_id: u32,
        player_state_command: Option<&PlayerStateCommand>,
        dt_micros: u64,
    ) {
        if !self.active_prediction {
            return;
        }

        if let Some(command) = player_state_command {
            let command_content = CommandContent {
                player_id: client_player_id,
                player_state_command: command.clone(),
                client_delay_micros: 0,
            };
            game_state.mutate(
                &[command_content.clone()],
                dt_micros,
                Some(client_player_id),
            );

            if self.active_reconciliation {
                self.reconciliation_commands.push(ReconciliationCommand {
                    command: command_content,
                    sequence: self.sequence,
                });
                self.sequence += 1;
            }
        } else {
            game_state.mutate(&[], dt_micros, Some(client_player_id));
        }
    }

    pub fn reconciliation(
        &mut self,
        game_state: &mut GameState,
        server_sequence: u32,
        client_player_id: u32,
        server_delay: u64,
    ) {
        if !self.active_reconciliation {
            return;
        }

        self.reconciliation_commands
            .retain(|c| c.sequence > server_sequence);

        let dt_micros = server_delay * 2;

        println!("dt_micros: {}", dt_micros);

        game_state.clear_cache();
        game_state.mutate(
            &self
                .reconciliation_commands
                .iter()
                .map(|c| c.command.clone())
                .collect::<Vec<_>>(),
            dt_micros,
            Some(client_player_id),
        );
    }
}
