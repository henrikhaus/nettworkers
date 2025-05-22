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
#[path = "../player_commands_generated.rs"]
mod player_commands_generated;
use crate::player_commands_generated::{PlayerCommand, PlayerCommands};

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

        if let Some(&id) = ip_to_player_id_guard.get(client_addr) {
            // Found player ID
            return id;
        }

        // Else add the player
        let highest_id = ip_to_player_id_guard.values().max().copied().unwrap_or(0);
        let new_player_id = highest_id + 1;

        ip_to_player_id_guard.insert(*client_addr, new_player_id);
        new_player_id
    }

    fn broadcast_state(&self, game_state: &GameState) {
        let mut builder = FlatBufferBuilder::with_capacity(2048);
        let bytes = game_state.serialize(&mut builder);
        // Send data to client
        for ip in self.get_client_ips() {
            if let Err(e) = self.socket.send_to(bytes, ip) {
                eprintln!("Failed to send data to client: {}", e);
            }
        }
    }

    fn get_client_ips(&self) -> Vec<SocketAddr> {
        let ip_to_player_guard = self.ip_to_player_id.lock().unwrap();
        let client_ips: Vec<_> = ip_to_player_guard.keys().copied().collect();
        client_ips
    }

    fn start_tick_thread(self: &Arc<Self>) {
        println!("Starting tick thread!");

        let mut game_state = GameState::new();
        let tick_server = Arc::clone(self);

        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let start = Instant::now();
                let dt = start.duration_since(last_tick).as_secs_f32();
                last_tick = start;

                tick_server.tick(&mut game_state, dt);
                tick_server.broadcast_state(&game_state);

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
