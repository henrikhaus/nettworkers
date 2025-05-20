use flatbuffers;
use flatbuffers::{root, FlatBufferBuilder, Push};
use macroquad::prelude::*;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

#[allow(dead_code, unused_imports)]
#[path = "../players_list_generated.rs"]
mod players_list_generated;
use crate::players_list_generated::{Color, PlayersList};
#[path = "../player_commands_generated.rs"]
mod player_commands_generated;
use crate::player_commands_generated::{PlayerCommand, PlayerCommands, PlayerCommandsArgs};

const SERVER_ADDR: &str = "127.0.0.1:9000";
// const CLIENT_ADDR: &str = "127.0.0.1:3001";
const PLAYER_SIZE: f32 = 16.0;

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

struct Resolution {
    width: f32,
    height: f32,
    scale: f32,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Multi".to_owned(),
        window_width: 640,
        window_height: 360,
        high_dpi: false,
        fullscreen: false,
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

    let mut scale = 1.0;
    change_resolution(
        Resolution {
            width: 640.0,
            height: 360.0,
            scale: 1.0,
        },
        &mut scale,
    );

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

    loop {
        input_handler(&mut commands);
        if !commands.is_empty() {
            let mut builder = FlatBufferBuilder::with_capacity(2048);
            let commands_vec = builder.create_vector(&commands);
            let player_command = PlayerCommands::create(
                &mut builder,
                &PlayerCommandsArgs {
                    commands: Some(commands_vec),
                },
            );
            builder.finish(player_command, None);
            let bytes = builder.finished_data();
            tick_socket
                .send_to(bytes, SERVER_ADDR)
                .expect("Packet couldn't send.");
        }
        commands.clear();

        let players_guard = tick_players.lock().unwrap();
        render(&players_guard, scale);
        drop(players_guard);
        next_frame().await;
    }
}

fn handle_packet(packet: &[u8], players: &mut Vec<OwnedPlayer>) {
    let players_list = root::<PlayersList>(packet).expect("No players received.");
    if let Some(player_vec) = players_list.players() {
        players.clear();
        for p in player_vec {
            players.push(OwnedPlayer {
                x: p.pos_x(),
                y: p.pos_y(),
                color: p.color(),
            });
        }
    }
}

fn render(players: &MutexGuard<Vec<OwnedPlayer>>, scale: f32) {
    clear_background(BLACK);
    let colors = [RED, BLUE, GREEN, PURPLE, ORANGE, BEIGE, PINK];
    for (index, p) in players.iter().enumerate() {
        draw_rectangle(
            p.x * scale,
            p.y * scale,
            PLAYER_SIZE * scale,
            PLAYER_SIZE * scale,
            colors[index % colors.len()],
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

fn change_resolution(resolution: Resolution, scale: &mut f32) {
    request_new_screen_size(resolution.width, resolution.height);
    *scale = resolution.scale;
}
