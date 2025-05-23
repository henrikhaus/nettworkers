use crate::generated;
use flatbuffers::{root, FlatBufferBuilder, WIPOffset};
use std::collections::HashMap;

use super::{GameState, PlayerState, PlayerStateCommand, SpawnPoint, Vec2};

impl GameState {
    pub fn serialize<'a>(
        &self,
        builder: &'a mut FlatBufferBuilder,
        client_player_id: u32,
    ) -> &'a [u8] {
        let client_player = self
            .players
            .get(&client_player_id)
            .expect("Game state should always contain the client player");
        let players_offsets: Vec<_> = self
            .players
            .iter()
            .filter(|p| *p.0 != client_player_id)
            .map(|(&_, player_state)| player_state.offset_player(builder))
            .collect();

        let players_vec = builder.create_vector(&players_offsets);

        let client_player_offset = client_player.offset_client_player(builder);
        let players_list = generated::GameState::create(
            builder,
            &generated::GameStateArgs {
                players: Some(players_vec),
                client_player: Some(client_player_offset),
            },
        );
        builder.finish(players_list, None);
        let bytes = builder.finished_data();
        bytes
    }

    pub fn deserialize(packet: &[u8]) -> (GameState, PlayerState) {
        let game_state = root::<generated::GameState>(packet).expect("No players received.");

        let players: HashMap<u32, PlayerState> = game_state
            .players()
            .expect("Should have players array")
            .into_iter()
            .map(|p| {
                (
                    p.id(),
                    PlayerState {
                        id: p.id(),
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
                id: p.id(),
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

impl From<generated::Vector2> for Vec2 {
    fn from(value: generated::Vector2) -> Self {
        Vec2 {
            x: value.x(),
            y: value.y(),
        }
    }
}

impl PlayerState {
    pub fn offset_client_player<'fbb>(
        &self,
        builder: &mut flatbuffers::FlatBufferBuilder<'fbb>,
    ) -> WIPOffset<generated::ClientPlayer<'fbb>> {
        let name_offset = builder.create_string(&self.name);
        generated::ClientPlayer::create(
            builder,
            &generated::ClientPlayerArgs {
                id: self.id,
                name: Some(name_offset),
                pos: Some(&generated::Vector2::new(self.pos.x, self.pos.y)),
                vel: Some(&generated::Vector2::new(self.vel.x, self.vel.y)),
                grounded: self.grounded,
                jump_timer: self.jump_timer,
                size: self.size,
                color: self.color,
            },
        )
    }

    pub fn offset_player<'fbb>(
        &self,
        builder: &mut flatbuffers::FlatBufferBuilder<'fbb>,
    ) -> WIPOffset<generated::Player<'fbb>> {
        let name_offset = builder.create_string(&self.name);
        generated::Player::create(
            builder,
            &generated::PlayerArgs {
                id: self.id,
                name: Some(name_offset),
                pos: Some(&generated::Vector2::new(self.pos.x, self.pos.y)),
                size: self.size,
                color: self.color,
            },
        )
    }
}

impl PlayerStateCommand {
    pub fn serialize<'fbb>(
        &self,
        builder: &mut flatbuffers::FlatBufferBuilder<'fbb>,
    ) -> WIPOffset<generated::PlayerCommands<'fbb>> {
        let commands_vec = builder.create_vector(&self.commands);
        generated::PlayerCommands::create(
            builder,
            &generated::PlayerCommandsArgs {
                sequence: self.sequence,
                commands: Some(commands_vec),
                dt_micro: self.dt_micro,
                client_timestamp_micro: self.client_timestamp_micro,
            },
        )
    }
}
