mod game_logic;
mod interpolator;
mod predictor;
mod render;
mod ui;

use flatbuffers::FlatBufferBuilder;
use interpolator::Interpolator;
use macroquad::math::f32;
use macroquad::prelude::*;
use predictor::Predictor;
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
use ui::screens::settings_menu;

use crate::game_logic::{Screen, SettingsState, UiState};
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
const DELAY_MILLIS: u64 = 300;
const SCENE_NAME: &str = "scene_3";

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
    state_sender: Sender<StateData>,
    settings_sender: Sender<SettingsState>,
}

type StateData = (GameState, PlayerState, u32, u64);

type NewClientResult = io::Result<(
    Client,
    Receiver<PlayerStateCommand>,
    Receiver<StateData>,
    Receiver<SettingsState>,
)>;

impl Client {
    fn new() -> NewClientResult {
        let socket = UdpSocket::bind(CLIENT_ADDR)?;
        let (command_sender, command_receiver) = mpsc::channel();
        let (state_sender, state_receiver) = mpsc::channel();
        let (settings_sender, settings_receiver) = mpsc::channel();

        Ok((
            Client {
                socket,
                command_sender,
                state_sender,
                settings_sender,
            },
            command_receiver,
            state_receiver,
            settings_receiver,
        ))
    }

    async fn start_game_loop(
        self: Arc<Self>,
        state_receiver: Receiver<StateData>,
    ) -> io::Result<()> {
        let mut client_player_id = 1;

        // Prediction + reconciliation
        let mut game_state = GameState::new(SCENE_NAME);
        let mut predictor = Predictor::new();

        // Interpolation
        let mut interpolator = Interpolator::new(&game_state);

        // Loading game scene
        let project_root = env!("CARGO_MANIFEST_DIR");
        let file = File::open(format!("{}/../scenes/{}.json", project_root, SCENE_NAME))
            .expect("Scene file must open");
        let scene: Scene = serde_json::from_reader(file).expect("JSON must match Scene");
        let mut last_frame = Instant::now();

        let mut ui = UiContext::new();
        let mut ui_state = UiState::new();
        let mut delay_enabled = true;

        loop {
            // Get Unix epoch timestamp (absolute time)
            let unix_timestamp_micro = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as u64;

            // Get new game state (if available)
            if let Ok((
                server_game_state,
                server_client_player,
                server_sequence,
                server_timestamp,
            )) = state_receiver.try_recv()
            {
                interpolator.set_new_state(server_game_state.clone());

                client_player_id = server_client_player.id;
                game_state.players = server_game_state.players;
                game_state
                    .players
                    .insert(server_client_player.id, server_client_player);

                let server_delay = unix_timestamp_micro.max(server_timestamp) - server_timestamp;

                // reconciliation
                predictor.reconciliation(
                    &mut game_state,
                    server_sequence,
                    client_player_id,
                    server_delay,
                );
            }

            // Get accurate frame timing
            let now = Instant::now();
            let dt_micros = now.duration_since(last_frame).as_micros() as u64;
            last_frame = now;

            // Get commands
            let commands = input_handler(&mut ui_state);
            let player_state_command = match commands.is_empty() {
                true => None,
                false => Some(PlayerStateCommand {
                    sequence: predictor.sequence,
                    dt_micros,
                    commands,
                    client_timestamp_micros: unix_timestamp_micro,
                }),
            };

            // Mutate local state
            predictor.predict(
                &mut game_state,
                client_player_id,
                player_state_command.as_ref(),
                dt_micros,
            );

            // Send command to network thread if exists
            if let Some(player_state_command) = player_state_command {
                if let Err(e) = self.command_sender.send(player_state_command) {
                    eprintln!("Error sending player state command: {}", e);
                }
            }

            // Interpolation
            interpolator.interpolate(&mut game_state, client_player_id);

            // Rendering game
            render(&game_state, client_player_id, &scene);

            // begin UI frame
            ui.begin_frame();

            match ui_state.current_screen() {
                Screen::MainMenu => main_menu(&mut ui, &mut ui_state),
                Screen::InGame => hud(&mut ui, &mut ui_state, &game_state, &scene),
                Screen::PauseMenu => pause_menu(&mut ui, &mut ui_state),
                Screen::Settings => settings_menu(
                    &mut ui,
                    &mut ui_state,
                    delay_enabled,
                    predictor.active_reconciliation,
                    predictor.active_prediction,
                    || {
                        delay_enabled = !delay_enabled;
                        self.settings_sender
                            .send(SettingsState {
                                delay: if delay_enabled { 300 } else { 0 },
                            })
                            .unwrap();
                    },
                    || {
                        predictor.active_reconciliation = !predictor.active_reconciliation;
                    },
                    || {
                        predictor.active_prediction = !predictor.active_prediction;
                    },
                ),
            }
            ui.end_frame();

            next_frame().await;
        }
    }

    fn start_network_thread(
        self: Arc<Client>,
        command_receiver: Receiver<PlayerStateCommand>,
        settings_receiver: Receiver<SettingsState>,
    ) -> io::Result<thread::JoinHandle<()>> {
        self.socket.set_nonblocking(true)?;

        let mut server_state_queue = VecDeque::new();
        let mut command_queue = VecDeque::new();
        let mut delay = DELAY_MILLIS;

        Ok(thread::spawn(move || {
            let mut buf = [0u8; 2048];
            loop {
                while let Ok(new_settings) = settings_receiver.try_recv() {
                    delay = new_settings.delay;
                }

                if let Ok((amt, src_addr)) = self.socket.recv_from(&mut buf) {
                    if src_addr.to_string() != SERVER_ADDR {
                        continue;
                    };

                    let apply_when = Instant::now() + Duration::from_millis(delay);
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
                        Instant::now() + Duration::from_millis(delay),
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
    let (client, command_receiver, state_receiver, settings_receiver) = Client::new()?;
    let client_arc: Arc<Client> = Arc::new(client);

    client_arc
        .clone()
        .start_network_thread(command_receiver, settings_receiver)
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
        commands.push(generated::PlayerCommand::MoveRight);
    }
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        commands.push(generated::PlayerCommand::MoveLeft);
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) || is_key_down(KeyCode::Space) {
        commands.push(generated::PlayerCommand::Jump);
    }

    commands
}
