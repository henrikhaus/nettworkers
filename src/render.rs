use crate::state::{GameState, PlayerState};
use crate::{Scene, SceneObject, FONT_SIZE, PLAYER_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH};
use macroquad::color::{BEIGE, BLUE, GREEN, ORANGE, PINK, PURPLE, RED, WHITE};
use macroquad::math::{vec2, Vec2};
use macroquad::shapes::draw_rectangle;
use macroquad::text::draw_text;
use macroquad::window::{clear_background, screen_height, screen_width};

pub fn render(game_state: &GameState, client_player: &Option<PlayerState>, scene: &Scene) {
    let w = screen_width();
    let h = screen_height();
    let scale_x = w / SCREEN_WIDTH;
    let scale_y = h / SCREEN_HEIGHT;
    let scale = scale_x.min(scale_y);
    let draw_w = SCREEN_WIDTH * scale;
    let draw_h = SCREEN_HEIGHT * scale;
    let offset = vec2((w - draw_w) / 2.0, (h - draw_h) / 2.0);

    let client_player_id = match client_player {
        Some(player) => player.id,
        None => 0,
    };

    let (px, py) = game_state
        .players
        .iter()
        .find(|p| *p.0 == client_player_id)
        .map(|p| (p.1.pos.x, p.1.pos.y))
        .unwrap_or((0.0, 0.0));

    let cam_pos = vec2(px.clamp(20.0, w - 20.0), py.clamp(20.0, w - 20.0));
    let world_offset = vec2(
        offset.x + SCREEN_WIDTH * scale / 2.0 - cam_pos.x * scale,
        offset.y + SCREEN_HEIGHT * scale / 2.0 - cam_pos.y * scale,
    );
    let screen_center_scaled = vec2(SCREEN_WIDTH * scale / 2.0, SCREEN_HEIGHT * scale / 2.0);

    let mut objects: Vec<SceneObject> = scene
        .decorations
        .values()
        .chain(scene.collidables.values())
        .cloned()
        .collect();
    objects.sort_by(|a, b| b.z.total_cmp(&a.z));

    // frame
    let border_color = macroquad::prelude::Color::from_rgba(
        scene.border_color.r,
        scene.border_color.g,
        scene.border_color.b,
        scene.border_color.a,
    );
    clear_background(border_color);

    // background
    let bg_color = macroquad::prelude::Color::from_rgba(
        scene.background_color.r,
        scene.background_color.g,
        scene.background_color.b,
        scene.background_color.a,
    );
    draw_rectangle(
        world_offset.x,
        world_offset.y,
        scene.width * scale,
        scene.height * scale,
        bg_color,
    );
    for obj in objects.iter().filter(|o| o.z >= 0.0) {
        draw_scene_obj(obj, scale, offset, screen_center_scaled, cam_pos);
    }

    // players
    for p in &game_state.players {
        let col = [RED, BLUE, GREEN, PURPLE, ORANGE, BEIGE, PINK][*p.0 as usize % 7];
        draw_rectangle(
            world_offset.x + p.1.pos.x * scale,
            world_offset.y + p.1.pos.y * scale,
            PLAYER_SIZE * scale,
            PLAYER_SIZE * scale,
            col,
        );
        draw_text(
            &p.1.name[..],
            world_offset.x
                + (p.1.pos.x + PLAYER_SIZE / 2.0 - FONT_SIZE * p.1.name.len() as f32 / 4.9) * scale,
            world_offset.y + (p.1.pos.y - 4.0) * scale,
            FONT_SIZE * scale,
            WHITE,
        );
    }

    // foreground
    for obj in objects.iter().filter(|o| o.z < 0.0) {
        draw_scene_obj(obj, scale, offset, screen_center_scaled, cam_pos);
    }
}

fn draw_scene_obj(
    obj: &SceneObject,
    scale: f32,
    offset: Vec2,
    screen_center_scaled: Vec2,
    cam_pos: Vec2,
) {
    let col =
        macroquad::prelude::Color::from_rgba(obj.color.r, obj.color.g, obj.color.b, obj.color.a);

    let parallax_strength_factor = 0.03;

    let pms = if obj.z == 0.0 {
        1.0
    } else {
        (1.0 - obj.z * parallax_strength_factor).max(0.0)
    };
    let screen_x = offset.x + screen_center_scaled.x + (obj.x - cam_pos.x * pms) * scale;
    let screen_y = offset.y + screen_center_scaled.y + (obj.y - cam_pos.y * pms) * scale;

    draw_rectangle(screen_x, screen_y, obj.w * scale, obj.h * scale, col);
}
