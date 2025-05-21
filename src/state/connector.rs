use std::collections::HashMap;

use flatbuffers::{root, FlatBufferBuilder};

use crate::game_state_generated::{self};

use super::{GameState, PlayerState, Vec2};

impl GameState {
    pub fn serialize<'a>(&self, builder: &'a mut FlatBufferBuilder) -> &'a [u8] {
        let players_offsets: Vec<_> = self
            .players
            .iter()
            .map(|p| {
                game_state_generated::Player::create(
                    builder,
                    &game_state_generated::PlayerArgs {
                        id: p.0,
                        pos: Some(&game_state_generated::Vector2::new(p.1.pos.x, p.1.pos.y)),
                        vel: Some(&game_state_generated::Vector2::new(p.1.vel.x, p.1.vel.y)),
                        jump_timer: p.1.jump_timer,
                        size: p.1.size,
                        color: p.1.color,
                    },
                )
            })
            .collect();

        let players_vec = builder.create_vector(&players_offsets);
        let players_list = game_state_generated::GameState::create(
            builder,
            &game_state_generated::GameStateArgs {
                players: Some(players_vec),
            },
        );
        builder.finish(players_list, None);
        let bytes = builder.finished_data();
        bytes
    }

    pub fn deserialize(packet: &[u8]) -> GameState {
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
                        id: p.id(),
                        pos: p.pos().unwrap().to_owned().into(),
                        vel: p.vel().unwrap().to_owned().into(),
                        jump_timer: p.jump_timer,
                        color: p.color,
                        size: p.size,
                    },
                )
            })
            .collect();

        GameState { players }
    }
}

impl From<game_state_generated::Vector2> for Vec2 {
    fn from(value: game_state_generated::Vector2) -> Self {
        Vec2 { x: 0., y: 0. }
    }
}
