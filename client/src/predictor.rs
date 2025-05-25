use shared::state::{CommandContent, GameState, PlayerStateCommand};

pub struct ReconciliationCommand {
    command: Option<CommandContent>,
    frame_dt_micros: u64,
    sequence: u32,
}

pub struct Predictor {
    unconfirmed_frames: Vec<ReconciliationCommand>,
    active: bool,
    pub sequence: u32,
}

impl Predictor {
    pub fn new() -> Self {
        Predictor {
            unconfirmed_frames: Vec::new(),
            active: true,
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
        if !self.active {
            return;
        }

        let mut command_content = None;
        if let Some(command) = player_state_command {
            game_state.mutate(&[(client_player_id, command.clone(), 0)], dt_micros);
            command_content = Some((client_player_id, command.clone(), 0));
        } else {
            game_state.mutate(&[], dt_micros);
        }

        self.unconfirmed_frames.push(ReconciliationCommand {
            command: command_content,
            frame_dt_micros: dt_micros,
            sequence: self.sequence,
        });

        self.sequence += 1;
    }

    pub fn reconciliation(&mut self, game_state: &mut GameState, server_sequence: u32) {
        if !self.active {
            return;
        }

        self.unconfirmed_frames
            .retain(|frame| frame.sequence > server_sequence);

        for reconciliation_frame in &self.unconfirmed_frames {
            let dt_micros = reconciliation_frame.frame_dt_micros;

            if let Some(command) = &reconciliation_frame.command {
                game_state.mutate(&[command.clone()], dt_micros);
            } else {
                game_state.mutate(&[], dt_micros);
            };
        }
    }
}
