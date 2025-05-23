use flatbuffers::FlatBufferBuilder;
use shared::state::{CommandContent, GameState, PlayerStateCommand};
use std::collections::HashMap;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

const SCENE_NAME: &str = "scene_1";
const TICK_DURATION: Duration = Duration::from_millis(1000);
const SERVER_ADDR: &str = "127.0.0.1:9000";

struct Server {
    command_sender: Sender<CommandContent>,
    ip_to_player_id: Arc<Mutex<HashMap<SocketAddr, u32>>>,
    socket: UdpSocket,
}

type NewServerResult = io::Result<(Server, Receiver<CommandContent>)>;

impl Server {
    fn new() -> NewServerResult {
        let socket = UdpSocket::bind(SERVER_ADDR)?;
        let (command_sender, command_receiver) = mpsc::channel();

        println!("UDP running on {}...", SERVER_ADDR);
        Ok((
            Server {
                command_sender,
                ip_to_player_id: Arc::new(Mutex::new(HashMap::new())),
                socket,
            },
            command_receiver,
        ))
    }

    fn handle_packet(&self, packet: &[u8], src_addr: SocketAddr) {
        let player_commands = PlayerStateCommand::deserialize(packet);
        let player_id = self.get_or_add_player_id(&src_addr);

        let player_delay_ms: u64 = 0;
        if !player_commands.commands.is_empty() {
            if let Err(e) = self
                .command_sender
                .send((player_id, player_commands, player_delay_ms))
            {
                eprintln!("Failed to send command: {}", e);
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
        // Send data to client
        for (ip, player_id) in self.read_ip_id() {
            let mut builder = FlatBufferBuilder::with_capacity(2048);
            let bytes = game_state.serialize(&mut builder, player_id);
            if let Err(e) = self.socket.send_to(bytes, ip) {
                eprintln!("Failed to send data to client: {}", e);
            }
        }
    }

    fn read_ip_id(&self) -> Vec<(SocketAddr, u32)> {
        let ip_to_player_guard = self.ip_to_player_id.lock().unwrap();
        ip_to_player_guard
            .iter()
            .map(|(&ip, &id)| (ip, id))
            .collect()
    }

    fn start_tick_thread(
        self: Arc<Self>,
        command_receiver: Receiver<CommandContent>,
        player_delay_receiver: Receiver<u64>,
    ) {
        println!("Starting tick thread!");

        let mut game_state = GameState::new(SCENE_NAME);

        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let start = Instant::now();
                let dt_micros = start.duration_since(last_tick).as_micros() as u64;
                last_tick = start;

                let mut commands = Vec::new();
                while let Ok((player_id, command, player_delay_ms)) = command_receiver.try_recv() {
                    commands.push((player_id, command, player_delay_ms));
                }

                self.tick(&mut game_state, &commands, dt_micros);
                self.broadcast_state(&game_state);

                for (player_id, player) in &game_state.players {
                    println!("Player {}", player_id);
                    println!("{:?}", player.pos.y);
                    println!("{:?}", player.pos.x);
                }

                let sleep_time = TICK_DURATION.checked_sub(start.elapsed());
                if let Some(sleep_time) = sleep_time {
                    sleep(sleep_time)
                }
            }
        });
    }

    pub fn run(
        self: Arc<Self>,
        command_receiver: Receiver<CommandContent>,
        player_delay_receiver: Receiver<u64>,
    ) -> io::Result<()> {
        Arc::clone(&self).start_tick_thread(command_receiver, player_delay_receiver);

        // Listen for commands
        let socket = self.socket.try_clone()?;
        loop {
            let mut buf = [0u8; 2048];
            let (amt, src_addr) = socket.recv_from(&mut buf)?;

            self.handle_packet(&buf[..amt], src_addr);
        }
    }

    fn tick(&self, game_state: &mut GameState, commands: &[CommandContent], dt_micros: u64) {
        game_state.mutate(commands, dt_micros);
    }
}

fn main() -> io::Result<()> {
    let (server, command_receiver, player_delay_receiver) = Server::new()?;
    let server_arc = Arc::new(server);
    server_arc.run(command_receiver, player_delay_receiver)
}
