use flatbuffers::{root, FlatBufferBuilder};
use macroquad::color::*;
use macroquad::math::f32;
use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
#[allow(dead_code, unused_imports)]
#[path = "../game_state_generated.rs"]
mod game_state_generated;
use crate::game_state_generated::Color;
#[path = "../player_commands_generated.rs"]
mod player_commands_generated;
use crate::player_commands_generated::{PlayerCommand, PlayerCommands, PlayerCommandsArgs};

const CLIENT_ADDR: &str = "127.0.0.1:0";
const SERVER_ADDR: &str = "127.0.0.1:9000";
const PLAYER_SIZE: f32 = 16.0;
const SCREEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 360.0;
const FONT_SIZE: f32 = 8.0;

const SCALE: f32 = 1.0;
const FULLSCREEN: bool = true;

#[derive(Debug, Deserialize, Clone)]
struct RgbaColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}
#[derive(Debug, Deserialize, Clone)]
struct SceneObject {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: RgbaColor,
    z: i32,
}

#[derive(Debug, Deserialize)]
struct SpawnPoint {
    x: f32,
    y: f32,
}

#[derive(Debug, Deserialize)]
struct Scene {
    decorations: HashMap<u32, SceneObject>,
    collidables: HashMap<u32, SceneObject>,
    width: f32,
    height: f32,
    background_color: RgbaColor,
    border_color: RgbaColor,
}

struct ClientPlayer {
    id: Option<usize>,
    pos: Vec2,
    color: Color,
}

struct OwnedPlayer {
    id: u32,
    x: f32,
    y: f32,
    name: String,
    color: Color,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Multi".to_owned(),
        window_width: (SCREEN_WIDTH * SCALE) as i32,
        window_height: (SCREEN_HEIGHT * SCALE) as i32,
        high_dpi: false,
        fullscreen: FULLSCREEN,
        sample_count: 1,
        window_resizable: true,
        icon: None,
        platform: Default::default(),
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut player = ClientPlayer {
        id: Some(1),
        pos: Vec2::ZERO,
        color: Color::Red,
    };

    let mut sequence: u32 = 0;

    let file = File::open("src/scenes/scene_1.json").expect("Scene file must open");
    let scene: Scene = serde_json::from_reader(file).expect("JSON must match Scene");

    let socket = Arc::new(UdpSocket::bind(CLIENT_ADDR).unwrap());
    let players: Arc<Mutex<Vec<OwnedPlayer>>> = Arc::new(Mutex::new(Vec::new()));
    let mut commands: Vec<PlayerCommand> = Vec::new();

    let tick_players: Arc<Mutex<Vec<OwnedPlayer>>> = Arc::clone(&players);
    let tick_socket = Arc::clone(&socket);

    thread::spawn(move || {
        let mut buf = [0u8; 2048];
        loop {
            let (amt, src_addr) = socket.recv_from(&mut buf).unwrap();
            if src_addr.to_string() != SERVER_ADDR {
                continue;
            };
            let mut players_guard = players.lock().unwrap();
            handle_packet(&buf[..amt], &mut players_guard);

            drop(players_guard);
        }
    });

    let mut scale;
    let mut objects: Vec<SceneObject> = scene
        .decorations
        .values()
        .chain(scene.collidables.values())
        .cloned()
        .collect();
    objects.sort_by(|a, b| b.z.cmp(&a.z));

    loop {
        input_handler(&mut commands);
        if !commands.is_empty() {
            let mut builder = FlatBufferBuilder::with_capacity(2048);
            let commands_vec = builder.create_vector(&commands);
            let player_command = PlayerCommands::create(
                &mut builder,
                &PlayerCommandsArgs {
                    sequence,
                    dt_sec: 0.0,
                    commands: Some(commands_vec),
                    client_timestamp: 0.0,
                },
            );
            sequence += 1;
            builder.finish(player_command, None);
            let bytes = builder.finished_data();
            tick_socket
                .send_to(bytes, SERVER_ADDR)
                .expect("Packet couldn't send.");
        }
        commands.clear();

        let players_guard = tick_players.lock().unwrap();

        let w = screen_width();
        let h = screen_height();
        let scale_x = w / SCREEN_WIDTH;
        let scale_y = h / SCREEN_HEIGHT;
        scale = scale_x.min(scale_y);
        let draw_w = SCREEN_WIDTH * scale;
        let draw_h = SCREEN_HEIGHT * scale;
        let offset = vec2((w - draw_w) / 2.0, (h - draw_h) / 2.0);

        let my_id = player.id.unwrap() as u32;
        let (px, py) = players_guard
            .iter()
            .find(|p| p.id == my_id)
            .map(|p| (p.x, p.y))
            .unwrap_or((player.pos.x, player.pos.y));
        let half_w = SCREEN_WIDTH / scale;
        let half_h = SCREEN_HEIGHT / scale;
        let cam_x = px.clamp(half_w, scene.width - half_w);
        let cam_y = py.clamp(half_h, scene.height - half_h);
        let world_offset = vec2(
            offset.x + SCREEN_WIDTH * scale / 2.0 - cam_x * scale,
            offset.y + SCREEN_HEIGHT * scale / 2.0 - cam_y * scale,
        );

        render(&players_guard, scale, world_offset, &scene);
        drop(players_guard);
        next_frame().await;
    }
}

fn handle_packet(packet: &[u8], players: &mut Vec<OwnedPlayer>) {
    let players_list =
        root::<game_state_generated::GameState>(packet).expect("No players received.");
    if let Some(player_vec) = players_list.players() {
        players.clear();
        for p in player_vec {
            players.push(OwnedPlayer {
                id: p.id(),
                name: p.name().unwrap().to_string(),
                x: p.pos().unwrap().x(),
                y: p.pos().unwrap().y(),
                color: p.color(),
            });
        }
    }
}

fn render(players: &MutexGuard<Vec<OwnedPlayer>>, scale: f32, offset: Vec2, scene: &Scene) {
    let mut objects: Vec<SceneObject> = scene
        .decorations
        .values()
        .chain(scene.collidables.values())
        .cloned()
        .collect();
    objects.sort_by(|a, b| b.z.cmp(&a.z));

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
        offset.x,
        offset.y,
        scene.width * scale,
        scene.height * scale,
        bg_color,
    );
    for obj in objects.iter().filter(|o| o.z >= 0) {
        draw_scene_obj(obj, scale, offset);
    }

    // players
    for (i, p) in players.iter().enumerate() {
        let col = [RED, BLUE, GREEN, PURPLE, ORANGE, BEIGE, PINK][i % 7];
        draw_rectangle(
            offset.x + p.x * scale,
            offset.y + p.y * scale,
            PLAYER_SIZE * scale,
            PLAYER_SIZE * scale,
            col,
        );
        draw_text(
            &p.name[..],
            offset.x + (p.x + PLAYER_SIZE / 2.0 - FONT_SIZE * p.name.len() as f32 / 4.9) * scale,
            offset.y + (p.y - 4.0) * scale,
            FONT_SIZE * scale,
            WHITE,
        );
    }

    // foreground
    for obj in objects.iter().filter(|o| o.z < 0) {
        draw_scene_obj(obj, scale, offset);
    }
}

fn draw_scene_obj(obj: &SceneObject, scale: f32, offset: Vec2) {
    let col =
        macroquad::prelude::Color::from_rgba(obj.color.r, obj.color.g, obj.color.b, obj.color.a);
    draw_rectangle(
        offset.x + obj.x * scale,
        offset.y + obj.y * scale,
        obj.w * scale,
        obj.h * scale,
        col,
    );
}

fn input_handler(commands: &mut Vec<PlayerCommand>) {
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        commands.push(PlayerCommand::Move_right);
    }
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        commands.push(PlayerCommand::Move_left);
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) || is_key_down(KeyCode::Space) {
        commands.push(PlayerCommand::Jump);
    }
}
