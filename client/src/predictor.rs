use std::time::{Instant, SystemTime, UNIX_EPOCH};

use shared::state::{CommandContent, GameState, PlayerStateCommand};

pub struct ReconciliationCommand {
    command: CommandContent,
    sequence: u32,
    client_timestamp: Instant,
}

pub struct Predictor {
    reconciliation_commands: Vec<ReconciliationCommand>,
    pub active_prediction: bool,
    pub active_reconciliation: bool,
    pub sequence: u32,
    // Maybe not necessary
    last_reconciliation_timestamp: u64,
}

impl Predictor {
    pub fn new() -> Self {
        Predictor {
            reconciliation_commands: Vec::new(),
            active_prediction: true,
            active_reconciliation: true,
            sequence: 0,
            last_reconciliation_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as u64,
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
            game_state.mutate(
                &[(client_player_id, command.clone(), 0)],
                dt_micros,
                Some(client_player_id),
            );
            let command_content = (client_player_id, command.clone(), 0);

            if self.active_reconciliation {
                self.reconciliation_commands.push(ReconciliationCommand {
                    command: command_content,
                    sequence: self.sequence,
                    client_timestamp: Instant::now(),
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
    ) {
        if !self.active_reconciliation {
            return;
        }

        self.reconciliation_commands
            .retain(|c| c.sequence > server_sequence);

        let dt_micros = self
            .reconciliation_commands
            .first()
            .map_or(1000000, |c| c.client_timestamp.elapsed().as_micros() as u64);

        println!("{}", dt_micros);

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
