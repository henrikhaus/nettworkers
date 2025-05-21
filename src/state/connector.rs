use flatbuffers::FlatBufferBuilder;

use crate::game_state_generated;

use super::GameState;

impl GameState {
    pub fn serialize<'a>(&self, builder: &'a mut FlatBufferBuilder) -> &'a [u8] {
        let players_offsets: Vec<_> = self
            .players
            .iter()
            .map(|p| {
                game_state_generated::Player::create(
                    builder,
                    &game_state_generated::PlayerArgs {
                        pos: Some(&game_state_generated::Vector2::new(p.1.pos.x, p.1.pos.y)),
                        vel: None,
                        acc: None,
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
}
