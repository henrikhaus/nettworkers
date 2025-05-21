use flatbuffers::{root, FlatBufferBuilder, Push};
use macroquad::color::hsl_to_rgb;
use macroquad::prelude::*;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

#[allow(dead_code, unused_imports)]
#[path = "../game_state_generated.rs"]
mod game_state_generated;
use crate::game_state_generated::{Color, GameState};
#[path = "../player_commands_generated.rs"]
mod player_commands_generated;
use crate::player_commands_generated::{PlayerCommand, PlayerCommands, PlayerCommandsArgs};

const SERVER_ADDR: &str = "127.0.0.1:9000";
const PLAYER_SIZE: f32 = 16.0;
const SCREEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 360.0;

const SCALE: f32 = 1.0;
const FULLSCREEN: bool = false;

struct ClientPlayer {
    id: Option<usize>,
    pos: Vec2,
    color: Color,
}

struct OwnedPlayer {
    x: f32,
    y: f32,
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

fn find_available_client_addr(start_port: u16, max_port: u16) -> (String, std::net::UdpSocket) {
    let ip = "127.0.0.1";
    for port in start_port..=max_port {
        let addr = format!("{}:{}", ip, port);
        match UdpSocket::bind(&addr) {
            Ok(socket) => {
                return (addr, socket);
            }
            Err(_) => continue,
        }
    }
    panic!("No available ports in range {}-{}", start_port, max_port);
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut player = ClientPlayer {
        id: Some(0),
        pos: Vec2::ZERO,
        color: Color::Red,
    };

    let mut sequence: u32 = 0;

    let (_client_addr, socket) = find_available_client_addr(3001, 3010);
    let socket = Arc::new(socket);
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

        render(&players_guard, scale, offset);
        drop(players_guard);
        next_frame().await;
    }
}

fn handle_packet(packet: &[u8], players: &mut Vec<OwnedPlayer>) {
    let players_list = root::<GameState>(packet).expect("No players received.");
    if let Some(player_vec) = players_list.players() {
        players.clear();
        for p in player_vec {
            players.push(OwnedPlayer {
                x: p.pos().unwrap().x(),
                y: p.pos().unwrap().y(),
                color: p.color(),
            });
        }
    }
}

fn render(players: &MutexGuard<Vec<OwnedPlayer>>, scale: f32, offset: Vec2) {
    clear_background(BLACK);
    draw_rectangle(
        offset.x,
        offset.y,
        SCREEN_WIDTH * scale,
        SCREEN_HEIGHT * scale,
        hsl_to_rgb(0.0, 0.0, 0.1),
    );
    let colors = [RED, BLUE, GREEN, PURPLE, ORANGE, BEIGE, PINK];
    for (i, p) in players.iter().enumerate() {
        draw_rectangle(
            offset.x + p.x * scale,
            offset.y + p.y * scale,
            PLAYER_SIZE * scale,
            PLAYER_SIZE * scale,
            colors[i % colors.len()],
        );
    }
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
