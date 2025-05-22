use flatbuffers::FlatBufferBuilder;
use macroquad::math::f32;
use macroquad::prelude::*;
use serde::Deserialize;
use state::GameState;
use std::collections::HashMap;
use std::fs::File;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
#[allow(dead_code, unused_imports)]
#[path = "../game_state_generated.rs"]
mod game_state_generated;
use crate::game_state_generated::Color;
#[path = "../player_commands_generated.rs"]
mod player_commands_generated;
mod render;
mod state;

use crate::player_commands_generated::{PlayerCommand, PlayerCommands, PlayerCommandsArgs};
use crate::render::render;

const CLIENT_ADDR: &str = "127.0.0.1:0";
const SERVER_ADDR: &str = "127.0.0.1:9000";
const PLAYER_SIZE: f32 = 16.0;
const SCREEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 360.0;
const FONT_SIZE: f32 = 8.0;

const SCALE: f32 = 1.0;
const FULLSCREEN: bool = false;

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
    z: f32,
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
    let mut sequence: u32 = 0;

    let file = File::open("src/scenes/scene_1.json").expect("Scene file must open");
    let scene: Scene = serde_json::from_reader(file).expect("JSON must match Scene");

    let socket = Arc::new(UdpSocket::bind(CLIENT_ADDR).unwrap());
    let game_state: Arc<Mutex<GameState>> = Arc::new(Mutex::new(GameState::new("scene_1")));
    let mut commands: Vec<PlayerCommand> = Vec::new();

    let tick_game_state = Arc::clone(&game_state);
    let tick_socket = Arc::clone(&socket);

    thread::spawn(move || {
        let mut buf = [0u8; 2048];
        loop {
            let (amt, src_addr) = socket.recv_from(&mut buf).unwrap();
            if src_addr.to_string() != SERVER_ADDR {
                continue;
            };
            let mut game_state_guard = game_state.lock().unwrap();
            handle_packet(&buf[..amt], &mut game_state_guard);

            drop(game_state_guard);
        }
    });

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

        let game_state_guard = tick_game_state.lock().unwrap();

        render(&game_state_guard, &scene);
        drop(game_state_guard);
        next_frame().await;
    }
}

fn handle_packet(packet: &[u8], game_state: &mut GameState) {
    let (new_game_state, client_player) = GameState::deserialize(packet);
    *game_state = new_game_state;
    game_state.players.insert(client_player.id, client_player);
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
