use flatbuffers::{root, FlatBufferBuilder};
use state::GameState;
use std::collections::{HashMap, VecDeque};
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

mod state;

#[allow(dead_code, unused_imports)]
#[path = "../game_state_generated.rs"]
mod game_state_generated;
use crate::game_state_generated::{
    GameState as GameStateSchema, Player as PlayerSchema, PlayerArgs, Vector2,
};
#[path = "../player_commands_generated.rs"]
mod player_commands_generated;
use crate::player_commands_generated::{PlayerCommand, PlayerCommands};

const SCENE_NAME: &str = "scene_1";
const TICK_DURATION: Duration = Duration::from_millis(16);
const SERVER_ADDR: &str = "127.0.0.1:9000";

struct Server {
    command_queue: Arc<Mutex<VecDeque<(u32, PlayerCommand)>>>,
    ip_to_player_id: Arc<Mutex<HashMap<SocketAddr, u32>>>,
    socket: UdpSocket,
}

impl Server {
    fn new() -> io::Result<Server> {
        let socket = UdpSocket::bind(SERVER_ADDR)?;
        println!("UDP running on {}...", SERVER_ADDR);
        Ok(Server {
            command_queue: Arc::new(Mutex::new(VecDeque::new())),
            ip_to_player_id: Arc::new(Mutex::new(HashMap::new())),
            socket,
        })
    }

    fn handle_packet(&self, packet: &[u8], src_addr: SocketAddr) {
        let player_commands = root::<PlayerCommands>(packet).expect("No command received");
        let player_id = self.get_or_add_player_id(&src_addr);

        let mut command_queue_guard = self.command_queue.lock().unwrap();
        if let Some(cmd_list) = player_commands.commands() {
            for cmd in cmd_list {
                command_queue_guard.push_back((player_id, cmd));
            }
        }
    }

    fn get_or_add_player_id(&self, client_addr: &SocketAddr) -> u32 {
        let mut ip_to_player_id_guard = self.ip_to_player_id.lock().unwrap();

        if let Some(id) = ip_to_player_id_guard.get(client_addr) {
            // Found player ID
            return id.to_owned();
        }

        // Else add the player
        let highest_id = ip_to_player_id_guard
            .iter()
            .map(|pair| pair.1.to_owned())
            .max()
            .unwrap_or(0);
        let new_player_id = highest_id + 1;

        ip_to_player_id_guard.insert(*client_addr, new_player_id);
        new_player_id
    }

    fn broadcast_state(&self, game_state: &GameState, ip_to_player_id: &HashMap<SocketAddr, u32>) {
        // Send data to client
        let mut builder = FlatBufferBuilder::with_capacity(2048);
        let players_offsets: Vec<_> = game_state
            .players
            .iter()
            .map(|p| {
                PlayerSchema::create(
                    &mut builder,
                    &PlayerArgs {
                        id: p.1.id,
                        pos: Some(&Vector2::new(p.1.pos.x, p.1.pos.y)),
                        vel: Some(&Vector2::new(p.1.vel.x, p.1.vel.y)),
                        color: p.1.color,
                        jump_timer: p.1.jump_timer,
                        size: p.1.size,
                    },
                )
            })
            .collect();

        let players_vec = builder.create_vector(&players_offsets);
        let players_list = GameStateSchema::create(
            &mut builder,
            &game_state_generated::GameStateArgs {
                players: Some(players_vec),
            },
        );
        builder.finish(players_list, None);
        let bytes = builder.finished_data();

        for (ip, _) in ip_to_player_id {
            let _ = self.socket.send_to(bytes, ip);
        }
    }

    fn start_tick_thread(self: &Arc<Self>) {
        println!("Starting tick thread!");

        let mut game_state = GameState::new(SCENE_NAME);
        let tick_server = Arc::clone(self);

        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let start = Instant::now();
                let now = Instant::now();
                let dt = now.duration_since(last_tick).as_secs_f32();
                last_tick = now;

                let ip_to_player_guard = tick_server.ip_to_player_id.lock().unwrap();

                tick_server.tick(&mut game_state, dt);
                tick_server.broadcast_state(&game_state, &ip_to_player_guard);
                drop(ip_to_player_guard);

                for (player_id, player) in &game_state.players {
                    println!("Player {}", player_id);
                    println!("{:?}", player.vel.y);
                    println!("{:?}", player.vel.x);
                }

                let sleep_time = TICK_DURATION.checked_sub(start.elapsed());
                if let Some(sleep_time) = sleep_time {
                    sleep(sleep_time)
                }
            }
        });
    }

    pub fn run(self: Arc<Self>) -> io::Result<()> {
        self.start_tick_thread();

        // Listen for commands
        let socket = self.socket.try_clone()?;
        loop {
            let mut buf = [0u8; 2048];
            let (amt, src_addr) = socket.recv_from(&mut buf)?;

            self.handle_packet(&buf[..amt], src_addr);
        }
    }

    fn tick(&self, game_state: &mut GameState, dt: f32) {
        let mut command_queue_guard = self.command_queue.lock().unwrap();
        game_state.mutate(&command_queue_guard, dt);
        command_queue_guard.clear();
    }
}

fn main() -> io::Result<()> {
    let server = Arc::new(Server::new()?);
    server.run()
}
