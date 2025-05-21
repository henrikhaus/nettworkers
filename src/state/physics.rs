use super::{GameState, SceneObject};
use super::{GRAVITY, GROUND_FRICTION, SCREEN_HEIGHT, SCREEN_WIDTH};

pub fn physics(state: &mut GameState, dt: f32) {
    for (_player_id, player) in &mut state.players {
        player.pos.x += player.vel.x * dt;
        player.pos.y += player.vel.y * dt;
        player.vel.x *= GROUND_FRICTION.powf(dt);
        player.vel.y += GRAVITY * dt;
        player.jump_timer += dt;

        if player.pos.y > SCREEN_HEIGHT as f32 - player.size {
            player.pos.y = SCREEN_HEIGHT as f32 - player.size;
            player.vel.y = 0.0;
        }
        if player.pos.y < 0.0 {
            player.pos.y = 0.0;
            player.vel.y = 0.0;
        }
        if player.pos.x > SCREEN_WIDTH as f32 - player.size {
            player.pos.x = SCREEN_WIDTH as f32 - player.size;
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
                let pen_x = if player.vel.x > 0.0 { px2 - cx1 } else { cx2 - px1 };
                let pen_y = if player.vel.y > 0.0 { py2 - cy1 } else { cy2 - py1 };

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
                }
            }
        }
    }
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
