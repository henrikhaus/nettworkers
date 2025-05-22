use flatbuffers::{root, FlatBufferBuilder, UOffsetT, WIPOffset};
use std::collections::HashMap;
use std::str::FromStr;

use crate::game_state_generated::{self};

use super::{GameState, PlayerState, SpawnPoint, Vec2};

impl GameState {
    pub fn serialize<'a>(
        &self,
        builder: &'a mut FlatBufferBuilder,
        client_player_id: u32,
    ) -> &'a [u8] {
        let client_player = self.players.get(&client_player_id).unwrap();
        let players_offsets: Vec<_> = self
            .players
            .iter()
            .filter(|p| *p.0 != client_player_id)
            .map(|(&player_id, player_state)| player_state.offset_player(builder, player_id))
            .collect();

        let players_vec = builder.create_vector(&players_offsets);

        let client_player_offset = client_player.offset_client_player(builder, client_player_id);
        let players_list = game_state_generated::GameState::create(
            builder,
            &game_state_generated::GameStateArgs {
                players: Some(players_vec),
                client_player: Some(client_player_offset),
            },
        );
        builder.finish(players_list, None);
        let bytes = builder.finished_data();
        bytes
    }

    pub fn deserialize(packet: &[u8]) -> (GameState, PlayerState) {
        let game_state =
            root::<game_state_generated::GameState>(packet).expect("No players received.");

        let players: HashMap<u32, PlayerState> = game_state
            .players()
            .expect("Should have players array")
            .into_iter()
            .map(|p| {
                (
                    p.id(),
                    PlayerState {
                        name: p.name().unwrap().to_string(),
                        pos: p.pos().unwrap().to_owned().into(),
                        vel: Vec2::zero(),
                        grounded: false,
                        jump_timer: 0.,
                        color: p.color(),
                        size: p.size(),
                    },
                )
            })
            .collect();

        let client_player: PlayerState = game_state
            .client_player()
            .map(|p| PlayerState {
                name: p.name().unwrap().to_string(),
                pos: p.pos().unwrap().to_owned().into(),
                vel: p.vel().unwrap().to_owned().into(),
                grounded: p.grounded(),
                jump_timer: p.jump_timer(),
                color: p.color(),
                size: p.size(),
            })
            .expect("Should have client player");

        (
            GameState {
                players,
                collidables: vec![],
                width: 0.0,
                height: 0.0,
                spawn_point: SpawnPoint { x: 0.0, y: 0.0 },
            },
            client_player,
        )
    }
}

impl From<game_state_generated::Vector2> for Vec2 {
    fn from(value: game_state_generated::Vector2) -> Self {
        Vec2 {
            x: value.x(),
            y: value.y(),
        }
    }
}

impl PlayerState {
    pub fn offset_client_player<'a, 'fbb>(
        &self,
        builder: &'a mut flatbuffers::FlatBufferBuilder<'fbb>,
        player_id: u32,
    ) -> WIPOffset<game_state_generated::ClientPlayer<'fbb>> {
        let name_offset = builder.create_string(&self.name);
        game_state_generated::ClientPlayer::create(
            builder,
            &game_state_generated::ClientPlayerArgs {
                id: player_id,
                name: Some(name_offset),
                pos: Some(&game_state_generated::Vector2::new(self.pos.x, self.pos.y)),
                vel: Some(&game_state_generated::Vector2::new(self.vel.x, self.vel.y)),
                grounded: self.grounded,
                jump_timer: self.jump_timer,
                size: self.size,
                color: self.color,
            },
        )
    }

    pub fn offset_player<'a, 'b>(
        &self,
        builder: &'a mut flatbuffers::FlatBufferBuilder<'b>,
        player_id: u32,
    ) -> WIPOffset<game_state_generated::Player<'b>> {
        let name_offset = builder.create_string(&self.name);
        game_state_generated::Player::create(
            builder,
            &game_state_generated::PlayerArgs {
                id: player_id,
                name: Some(name_offset),
                pos: Some(&game_state_generated::Vector2::new(self.pos.x, self.pos.y)),
                size: self.size,
                color: self.color,
            },
        )
    }
}
