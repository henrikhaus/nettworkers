use super::{GRAVITY, GROUND_FRICTION};
use super::{GameState, PlayerState, SceneObject, SpawnPoint, Vec2};
use crate::generated::Color;

pub fn physics(state: &mut GameState, dt: f32) {
    for (_player_id, player) in &mut state.players {
        player.vel.x *= GROUND_FRICTION.powf(dt);
        player.vel.y += GRAVITY * dt;
        player.pos.x += player.vel.x * dt;
        player.pos.y += player.vel.y * dt;

        player.jump_timer += dt;
        player.grounded = false;

        if player.pos.y > state.height - player.size {
            player.pos.y = state.height - player.size;
            player.vel.y = 0.0;
            player.grounded = true;
        }
        if player.pos.y < 0.0 {
            player.pos.y = 0.0;
            player.vel.y = 0.0;
        }
        if player.pos.x > state.width - player.size {
            player.pos.x = state.width - player.size;
            player.vel.x = 0.0;
        }
        if player.pos.x < 0.0 {
            player.pos.x = 0.0;
            player.vel.x = 0.0;
        }
    }

    let collidables = &state.collidables;

    for (_id, player) in &mut state.players {
        // player's AABB
        let px1 = player.pos.x;
        let py1 = player.pos.y;
        let px2 = px1 + player.size;
        let py2 = py1 + player.size;

        for col in collidables {
            let cx1 = col.x;
            let cy1 = col.y;
            let cx2 = cx1 + col.w;
            let cy2 = cy1 + col.h;

            // overlap test
            if px1 < cx2 && px2 > cx1 && py1 < cy2 && py2 > cy1 {
                // compute penetration depths on each axis
                let pen_x = if player.vel.x > 0.0 {
                    px2 - cx1
                } else {
                    cx2 - px1
                };
                let pen_y = if player.vel.y > 0.0 {
                    py2 - cy1
                } else {
                    cy2 - py1
                };

                if pen_x < pen_y {
                    // resolve in X
                    if player.vel.x > 0.0 {
                        player.pos.x = cx1 - player.size;
                    } else {
                        player.pos.x = cx2;
                    }
                    player.vel.x = 0.0;
                } else {
                    // resolve in Y
                    if player.vel.y > 0.0 {
                        player.pos.y = cy1 - player.size;
                    } else {
                        player.pos.y = cy2;
                    }
                    player.vel.y = 0.0;
                    player.grounded = true;
                }
            }
        }
    }

    // Add a collidable block in the player's path
    state.collidables.push(SceneObject {
        x: 150.0,
        y: 90.0,
        w: 32.0,
        h: 32.0,
    });
}

// pub fn collision(state: &mut GameState) -> Vec<(usize, Vec2, Vec2)> {
//     let mut player_forces = vec![];
//     for (player_id_1, player) in &mut state.players {
//         for p2 in players {
//             if p1.id == p2.id {
//                 continue;
//             }

//             let v_overlap = p1.pos.y <= p2.pos.y + p2.size && p2.pos.y <= p1.pos.y + p1.size;
//             let h_overlap = p1.pos.x <= p2.pos.x + p2.size && p2.pos.x <= p1.pos.x + p1.size;
//             let overlap = v_overlap && h_overlap;

//             let p1_top = overlap && p1.vel.y > p2.vel.y;
//             let p1_bottom = overlap && p1.vel.y < p2.vel.y;
//             let p1_left = overlap && p1.vel.x > p2.vel.x;
//             let p1_right = overlap && p1.vel.x < p2.vel.x;

//             if overlap {
//                 /*if p1_top {
//                     let force = Vec2 { x: (p1.vel.x + p2.vel.x) / 2.0, y: 0.0 };
//                     let pos = Vec2 { x: p1.pos.x, y: p2.pos.y - p1.size };
//                     player_forces.push((i, force, pos));
//                 } else*/
//                 if p1_left {
//                     let force = Vec2 {
//                         x: (p1.vel.x + p2.vel.x) / 2.0,
//                         y: (p1.vel.y + p2.vel.y) / 2.0,
//                     };
//                     let pos = Vec2 {
//                         x: p2.pos.x - p1.size,
//                         y: p1.pos.y,
//                     };
//                     player_forces.push((i, force, pos));
//                 } else if p1_right {
//                     let force = Vec2 {
//                         x: (p1.vel.x + p2.vel.x) / 2.0,
//                         y: (p1.vel.y + p2.vel.y) / 2.0,
//                     };
//                     let pos = Vec2 {
//                         x: p1.pos.x,
//                         y: p1.pos.y,
//                     };
//                     player_forces.push((i, force, pos));
//                 }
//             }
//         }
//     }
//     player_forces
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_state() -> GameState {
        GameState {
            players: HashMap::new(),
            collidables: vec![],
            width: 800.0,
            height: 600.0,
            spawn_point: SpawnPoint { x: 0.0, y: 0.0 },
        }
    }

    fn create_test_player(id: u32, x: f32, y: f32) -> (u32, PlayerState) {
        (
            id,
            PlayerState {
                id,
                name: format!("Player {}", id),
                pos: Vec2 { x, y },
                vel: Vec2 { x: 0.0, y: 0.0 },
                grounded: false,
                jump_timer: 0.0,
                color: Color::Red,
                size: 32.0,
            },
        )
    }

    #[test]
    fn test_basic_movement() {
        let mut state = create_test_state();
        let (id, mut player) = create_test_player(1, 100.0, 100.0);
        player.vel = Vec2 { x: 10.0, y: 5.0 };
        state.players.insert(id, player);

        physics(&mut state, 1.0);

        let player = state.players.get(&1).unwrap();
        assert!(player.pos.x > 100.0, "Player should move right");
        assert!(player.pos.y > 100.0, "Player should move down");
    }

    #[test]
    fn test_ground_collision() {
        let mut state = create_test_state();
        let (id, mut player) = create_test_player(1, 100.0, state.height - 16.0);
        player.vel = Vec2 { x: 0.0, y: 50.0 };
        state.players.insert(id, player);

        physics(&mut state, 1.0);

        let player = state.players.get(&1).unwrap();
        assert_eq!(
            player.pos.y,
            state.height - player.size,
            "Player should stop at ground"
        );
        assert_eq!(
            player.vel.y, 0.0,
            "Vertical velocity should be zero on ground"
        );
        assert!(player.grounded, "Player should be marked as grounded");
    }

    #[test]
    fn test_wall_collisions() {
        let mut state = create_test_state();
        let (id, mut player) = create_test_player(1, 0.0, 100.0);
        player.vel = Vec2 { x: -10.0, y: 0.0 };
        state.players.insert(id, player);

        physics(&mut state, 1.0);

        let player = state.players.get(&1).unwrap();
        assert_eq!(player.pos.x, 0.0, "Player should stop at left wall");
        assert_eq!(
            player.vel.x, 0.0,
            "Horizontal velocity should be zero at wall"
        );

        // Test right wall
        let mut player = state.players.get_mut(&1).unwrap();
        player.pos.x = state.width;
        player.vel.x = 10.0;

        physics(&mut state, 1.0);

        let player = state.players.get(&1).unwrap();
        assert_eq!(
            player.pos.x,
            state.width - player.size,
            "Player should stop at right wall"
        );
        assert_eq!(
            player.vel.x, 0.0,
            "Horizontal velocity should be zero at wall"
        );
    }

    #[test]
    fn test_ground_friction() {
        let mut state = create_test_state();
        let (id, mut player) = create_test_player(1, 100.0, 100.0);
        player.vel = Vec2 { x: 10.0, y: 0.0 };
        state.players.insert(id, player);

        let initial_vel_x = state.players.get(&1).unwrap().vel.x;
        physics(&mut state, 1.0);

        let player = state.players.get(&1).unwrap();
        assert!(
            player.vel.x < initial_vel_x,
            "Ground friction should reduce horizontal velocity"
        );
    }

    #[test]
    fn test_jump_timer() {
        let mut state = create_test_state();
        let (id, player) = create_test_player(1, 100.0, 100.0);
        state.players.insert(id, player);

        let initial_timer = state.players.get(&1).unwrap().jump_timer;
        physics(&mut state, 1.0);

        let player = state.players.get(&1).unwrap();
        assert!(
            player.jump_timer > initial_timer,
            "Jump timer should increase over time"
        );
    }
}
