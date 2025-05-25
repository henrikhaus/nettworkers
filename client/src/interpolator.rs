use std::time::{Duration, Instant};

use shared::state::{self, GameState};

pub struct Interpolator {
    old_server_state: GameState,
    new_server_state: GameState,
    received_new_state_at: Instant,
    interpolation_time: Duration,
    t: f32,
    pub active: bool,
}

impl Interpolator {
    pub fn new(game_state: &GameState) -> Self {
        Self {
            old_server_state: game_state.clone(),
            new_server_state: game_state.clone(),
            received_new_state_at: Instant::now(),
            interpolation_time: Duration::ZERO,
            t: 0.0,
            active: true,
        }
    }

    pub fn set_new_state(&mut self, new_state: GameState) {
        self.old_server_state = self.new_server_state.clone();
        self.new_server_state = new_state;
        self.interpolation_time = Instant::now().duration_since(self.received_new_state_at);
        self.received_new_state_at = Instant::now();
    }

    pub fn interpolate(&mut self, game_state: &mut GameState, client_player_id: u32) {
        if !self.active {
            return;
        }

        self.update_t();
        for player in game_state.players.values_mut() {
            if player.id == client_player_id {
                continue;
            }

            let old_position = self
                .old_server_state
                .players
                .get(&player.id)
                .map_or(state::Vec2::ZERO, |p| p.pos);
            let new_position = self
                .new_server_state
                .players
                .get(&player.id)
                .map_or(state::Vec2::ZERO, |p| p.pos);

            player.pos = self.lerp_position(old_position, new_position);
        }
    }

    pub fn update_t(&mut self) {
        self.t = self.received_new_state_at.elapsed().as_millis() as f32
            / self.interpolation_time.as_millis() as f32;
    }

    pub fn lerp_position(&self, start: state::Vec2, end: state::Vec2) -> state::Vec2 {
        start + (end - start) * self.t
    }
}
