use flatbuffers::FlatBufferBuilder;
use macroquad::math::f32;
use macroquad::prelude::*;
use serde::Deserialize;
use shared::generated;
use shared::state;
use state::{GameState, PlayerState, PlayerStateCommand};
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{io, thread};

mod game_logic;
mod render;
mod ui;

use crate::game_logic::{Screen, UiState};
use crate::render::render;
use crate::ui::{UiContext, pause_menu, screens::hud, screens::main_menu};

const CLIENT_ADDR: &str = "127.0.0.1:0";
const SERVER_ADDR: &str = "127.0.0.1:9000";
const PLAYER_SIZE: f32 = 16.0;
const SCREEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 360.0;
const SCREEN_CLAMP_DISTANCE_X: f32 = 200.0;
const SCREEN_CLAMP_DISTANCE_Y: f32 = 400.0;
const FONT_SIZE: f32 = 8.0;
const DELAY_MS: u64 = 300;
const SCENE_NAME: &str = "scene_1";

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

    async fn start_game_loop(
        self: Arc<Self>,
        state_receiver: Receiver<(GameState, PlayerState)>,
    ) -> io::Result<()> {
        let mut sequence: u32 = 0;

        let mut game_state = GameState::new(SCENE_NAME);
        let mut client_player = None;

        let project_root = env!("CARGO_MANIFEST_DIR");
        let file = File::open(format!("{}/../scenes/{}.json", project_root, SCENE_NAME))
            .expect("Scene file must open");
        let scene: Scene = serde_json::from_reader(file).expect("JSON must match Scene");
        let mut last_frame = Instant::now();

        let mut ui = UiContext::new();
        let mut ui_state = UiState::new();

        loop {
            // Get new game state (if available)
            if let Ok((server_game_state, server_client_player)) = state_receiver.try_recv() {
                game_state = server_game_state;

                client_player = Some(server_client_player);
            }

            // Get accurate frame timing
            let now = Instant::now();
            let dt_micro = now.duration_since(last_frame).as_micros() as u64;
            last_frame = now;

            // Get Unix epoch timestamp (absolute time)
            let unix_timestamp_micro = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as u64;

            // Get commands and send to network thread
            let commands = input_handler(&mut ui_state);
            if !commands.is_empty() {
                let player_state_command = PlayerStateCommand {
                    sequence,
                    dt_micro,
                    commands,
                    client_timestamp_micro: unix_timestamp_micro,
                };

                if let Err(e) = self.command_sender.send(player_state_command) {
                    eprintln!("Error sending player state command: {}", e);
                } else {
                    sequence += 1;
                }
            };

            // Rendering game
            render(&game_state, &client_player, &scene);

            // begin UI frame
            ui.begin_frame();
            match ui_state.current_screen() {
                Screen::MainMenu => main_menu(&mut ui, &mut ui_state),
                Screen::InGame => hud(&mut ui, &mut ui_state, &game_state, &scene),
                Screen::PauseMenu => pause_menu(&mut ui, &mut ui_state),
                _ => {}
            }
            ui.end_frame();

            next_frame().await;
        }
    }

    fn start_network_thread(
        self: Arc<Client>,
        command_receiver: Receiver<PlayerStateCommand>,
    ) -> io::Result<thread::JoinHandle<()>> {
        self.socket.set_nonblocking(true)?;

        let mut server_state_queue = VecDeque::new();
        let mut command_queue = VecDeque::new();

        Ok(thread::spawn(move || {
            let mut buf = [0u8; 2048];
            loop {
                if let Ok((amt, src_addr)) = self.socket.recv_from(&mut buf) {
                    if src_addr.to_string() != SERVER_ADDR {
                        continue;
                    };

                    let apply_when = Instant::now() + Duration::from_millis(DELAY_MS);
                    server_state_queue.push_back((apply_when, GameState::deserialize(&buf[..amt])));
                };

                let mut last_valid_state = None;
                while let Some((apply_when, _)) = server_state_queue.front() {
                    if Instant::now() >= *apply_when {
                        if let Some((_, game_state)) = server_state_queue.pop_front() {
                            last_valid_state = Some(game_state);
                        }
                    } else {
                        break;
                    }
                }

                // Send to game loop if new state is popped from the queue
                if let Some(game_state) = last_valid_state {
                    if let Err(e) = self.state_sender.send(game_state) {
                        eprintln!("Error sending game state: {}", e);
                    }
                }

                // Add commands to queue
                while let Ok(player_state_command) = command_receiver.try_recv() {
                    command_queue.push_back((
                        Instant::now() + Duration::from_micros(DELAY_MS),
                        player_state_command,
                    ));
                }

                // Send commands to server when ready
                while let Some((apply_when, _)) = command_queue.front() {
                    if Instant::now() >= *apply_when {
                        if let Some((_, player_state_command)) = command_queue.pop_front() {
                            let mut builder = FlatBufferBuilder::with_capacity(2048);
                            let serialized_commands = player_state_command.serialize(&mut builder);
                            builder.finish(serialized_commands, None);
                            let bytes = builder.finished_data();
                            self.socket
                                .send_to(bytes, SERVER_ADDR)
                                .expect("Packet couldn't send.");
                        }
                    } else {
                        break;
                    }
                }
            }
        }))
    }
}

#[macroquad::main(window_conf)]
async fn main() -> io::Result<()> {
    let (client, command_receiver, state_receiver) = Client::new()?;
    let client_arc = Arc::new(client);

    client_arc
        .clone()
        .start_network_thread(command_receiver)
        .expect("Failed to start network thread");
    client_arc.clone().start_game_loop(state_receiver).await
}

fn input_handler(ui_state: &mut UiState) -> Vec<generated::PlayerCommand> {
    // --- CLIENT/UI INPUT ---
    if is_key_pressed(KeyCode::Escape) {
        match ui_state.current_screen() {
            Screen::InGame => {
                ui_state.push(Screen::PauseMenu);
            }
            Screen::MainMenu => {
                // Do nothing
            }
            _ => {
                ui_state.pop();
            }
        }
    }

    // --- NETWORK INPUT ---
    let mut commands = Vec::new();
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        commands.push(generated::PlayerCommand::Move_right);
    }
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        commands.push(generated::PlayerCommand::Move_left);
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) || is_key_down(KeyCode::Space) {
        commands.push(generated::PlayerCommand::Jump);
    }

    commands
}
