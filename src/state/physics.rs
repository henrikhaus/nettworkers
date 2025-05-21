use super::model::*;
use super::{SCREEN_HEIGHT, SCREEN_WIDTH};

const GRAVITY: f32 = 1000.0;
const FRICTION: f32 = 0.8;

fn physics(players: &mut [PlayerState], dt: f32) {
    for player in players {
        player.pos.x += player.vel.x * dt;
        player.pos.y += player.vel.y * dt;
        player.vel.x *= FRICTION.powf(dt);
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
}

fn collision(players: &[PlayerState]) -> Vec<(usize, Vec2, Vec2)> {
    let mut player_forces = vec![];
    for (i, p1) in players.iter().enumerate() {
        for p2 in players {
            if p1.id == p2.id {
                continue;
            }

            let v_overlap = p1.pos.y <= p2.pos.y + p2.size && p2.pos.y <= p1.pos.y + p1.size;
            let h_overlap = p1.pos.x <= p2.pos.x + p2.size && p2.pos.x <= p1.pos.x + p1.size;
            let overlap = v_overlap && h_overlap;

            let p1_top = overlap && p1.vel.y > p2.vel.y;
            let p1_bottom = overlap && p1.vel.y < p2.vel.y;
            let p1_left = overlap && p1.vel.x > p2.vel.x;
            let p1_right = overlap && p1.vel.x < p2.vel.x;

            if overlap {
                /*if p1_top {
                    let force = Vec2 { x: (p1.vel.x + p2.vel.x) / 2.0, y: 0.0 };
                    let pos = Vec2 { x: p1.pos.x, y: p2.pos.y - p1.size };
                    player_forces.push((i, force, pos));
                } else*/
                if p1_left {
                    let force = Vec2 {
                        x: (p1.vel.x + p2.vel.x) / 2.0,
                        y: (p1.vel.y + p2.vel.y) / 2.0,
                    };
                    let pos = Vec2 {
                        x: p2.pos.x - p1.size,
                        y: p1.pos.y,
                    };
                    player_forces.push((i, force, pos));
                } else if p1_right {
                    let force = Vec2 {
                        x: (p1.vel.x + p2.vel.x) / 2.0,
                        y: (p1.vel.y + p2.vel.y) / 2.0,
                    };
                    let pos = Vec2 {
                        x: p1.pos.x,
                        y: p1.pos.y,
                    };
                    player_forces.push((i, force, pos));
                }
            }
        }
    }
    player_forces
}
