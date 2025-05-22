use flatbuffers::FlatBufferBuilder;
use macroquad::math::f32;
use macroquad::prelude::*;
use serde::Deserialize;
use state::{GameState, PlayerState, PlayerStateCommand};
use std::collections::HashMap;
use std::fs::File;
use std::net::UdpSocket;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::{io, thread};

#[allow(dead_code, unused_imports)]
#[path = "../game_state_generated.rs"]
mod game_state_generated;
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
struct Scene {
    decorations: HashMap<u32, SceneObject>,
    collidables: HashMap<u32, SceneObject>,
    width: f32,
    height: f32,
    background_color: RgbaColor,
    border_color: RgbaColor,
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

struct Client {
    socket: UdpSocket,
    command_sender: Sender<PlayerStateCommand>,
    state_sender: Sender<(GameState, PlayerState)>,
}

type NewClientResult = io::Result<(
    Client,
    Receiver<PlayerStateCommand>,
    Receiver<(GameState, PlayerState)>,
)>;

impl Client {
    fn new() -> NewClientResult {
        let socket = UdpSocket::bind(CLIENT_ADDR)?;
        let (command_sender, command_receiver) = mpsc::channel();
        let (state_sender, state_receiver) = mpsc::channel();

        Ok((
            Client {
                socket,
                command_sender,
                state_sender,
            },
            command_receiver,
            state_receiver,
        ))
    }

    fn start_game_loop(
        self: Arc<Self>,
        state_receiver: Receiver<(GameState, PlayerState)>,
    ) -> io::Result<()> {
        let mut sequence: u32 = 0;

        let mut game_state = GameState::new("scene_1");

        let file = File::open("src/scenes/scene_1.json").expect("Scene file must open");
        let scene: Scene = serde_json::from_reader(file).expect("JSON must match Scene");

        thread::spawn(async move || loop {
            // Get new game state (if available)
            if let Ok((server_game_state, client_player_state)) = state_receiver.try_recv() {
                game_state = server_game_state;
                game_state
                    .players
                    .insert(client_player_state.id, client_player_state);
            }

            // Get commands and send to network thread
            let commands = input_handler();
            let player_state_command = PlayerStateCommand {
                sequence,
                dt_sec: 0.0,
                commands,
                client_timestamp: 0,
            };

            if let Err(e) = self.command_sender.send(player_state_command) {
                eprintln!("Error sending player state command: {}", e);
            } else {
                sequence += 1;
            }

            // Rendering game
            render(&game_state, &scene);

            next_frame().await;
        });

        Ok(())
    }

    fn start_network_thread(
        self: Arc<Client>,
        command_receiver: Receiver<PlayerStateCommand>,
    ) -> io::Result<thread::JoinHandle<()>> {
        self.socket.set_nonblocking(true)?;

        Ok(thread::spawn(move || {
            let mut buf = [0u8; 2048];
            loop {
                let (amt, src_addr) = self.socket.recv_from(&mut buf).unwrap();
                if src_addr.to_string() != SERVER_ADDR {
                    continue;
                };

                let server_game_state = GameState::deserialize(&buf[..amt]);

                // Send commands to server
                while let Ok(player_state_command) = command_receiver.try_recv() {
                    let mut builder = FlatBufferBuilder::with_capacity(2048);
                    let commands_vec = builder.create_vector(&player_state_command.commands);
                    let player_command = PlayerCommands::create(
                        &mut builder,
                        &PlayerCommandsArgs {
                            sequence: player_state_command.sequence,
                            dt_sec: player_state_command.dt_sec,
                            commands: Some(commands_vec),
                            client_timestamp: 0.,
                        },
                    );
                    builder.finish(player_command, None);
                    let bytes = builder.finished_data();
                    self.socket
                        .send_to(bytes, SERVER_ADDR)
                        .expect("Packet couldn't send.");
                }

                self.state_sender.send(server_game_state).unwrap();
            }
        }))
    }
}

#[macroquad::main(window_conf)]
async fn main() -> io::Result<()> {
    let (client, command_receiver, state_receiver) = Client::new()?;
    let client_arc = Arc::new(client);

    client_arc.clone().start_game_loop(state_receiver)?;
    client_arc
        .clone()
        .start_network_thread(command_receiver)
        .expect("Failed to start network thread")
        .join()
        .unwrap();

    Ok(())
}

fn input_handler() -> Vec<PlayerCommand> {
    let mut commands = Vec::new();
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        commands.push(PlayerCommand::Move_right);
    }
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        commands.push(PlayerCommand::Move_left);
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) || is_key_down(KeyCode::Space) {
        commands.push(PlayerCommand::Jump);
    }

    commands
}
