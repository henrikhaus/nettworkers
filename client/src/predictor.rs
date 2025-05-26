use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
    last_command_timestamp: Instant,
    last_reconciliation: Instant,
}

impl Predictor {
    pub fn new() -> Self {
        Predictor {
            reconciliation_commands: Vec::new(),
            active_prediction: true,
            active_reconciliation: true,
            sequence: 0,
            last_command_timestamp: Instant::now(),
            last_reconciliation: Instant::now(),
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
                    client_timestamp: Instant::now(),
                });
                self.sequence += 1;

                self.last_command_timestamp = Instant::now();
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

        let dt_micros = match self.reconciliation_commands.first() {
            Some(command) => command.client_timestamp.elapsed().as_micros() as u64,
            None => {
                let elapsed = self.last_command_timestamp.elapsed().as_micros() as u64;
                self.last_command_timestamp +=
                    Instant::now().duration_since(self.last_reconciliation);
                elapsed
            }
        };

        // let dt_micros = 600000;

        game_state.mutate(
            &self
                .reconciliation_commands
                .iter()
                .map(|c| c.command.clone())
                .collect::<Vec<_>>(),
            dt_micros,
            Some(client_player_id),
        );

        self.last_reconciliation = Instant::now();
    }
}
