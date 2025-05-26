use flatbuffers::FlatBufferBuilder;
use shared::generated::{PlayerCommand, PlayerCommands, PlayerCommandsArgs};
use shared::state::{CommandContent, GameState, PlayerStateCommand};
use std::collections::HashMap;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SCENE_NAME: &str = "scene_3";
const TICK_DURATION: Duration = Duration::from_millis(16);
const SERVER_ADDR: &str = "127.0.0.1:9000";

struct Server {
    command_sender: Sender<CommandContent>,
    ip_to_player_id: Arc<Mutex<HashMap<SocketAddr, u32>>>,
    socket: UdpSocket,
}

type NewServerResult = io::Result<(Server, Receiver<CommandContent>)>;

impl Server {
    fn new() -> NewServerResult {
        Self::with_addr(SERVER_ADDR)
    }

    fn with_addr(addr: &str) -> NewServerResult {
        let socket = UdpSocket::bind(addr)?;
        let (command_sender, command_receiver) = mpsc::channel();

        println!("UDP running on {}...", addr);
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

        let system_time_micro = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        let player_delay_ms = system_time_micro - player_commands.client_timestamp_micro;
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

    fn broadcast_state(&self, game_state: &GameState, sequence: u32) {
        // Send data to client
        for (ip, player_id) in self.read_ip_id() {
            let mut builder = FlatBufferBuilder::with_capacity(2048);
            let bytes = game_state.serialize(&mut builder, player_id, sequence);
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

    fn start_tick_thread(self: Arc<Self>, command_receiver: Receiver<CommandContent>) {
        println!("Starting tick thread!");

        let mut game_state = GameState::new(SCENE_NAME);

        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let start = Instant::now();
                let dt_micros = start.duration_since(last_tick).as_micros() as u64;
                last_tick = start;

                let mut sequence = 0;
                let mut commands = Vec::new();
                while let Ok((player_id, command, player_delay_ms)) = command_receiver.try_recv() {
                    sequence = sequence.max(command.sequence);
                    commands.push((player_id, command, player_delay_ms));
                }

                self.tick(&mut game_state, &commands, dt_micros);
                self.broadcast_state(&game_state, sequence);

                let sleep_time = TICK_DURATION.checked_sub(start.elapsed());
                if let Some(sleep_time) = sleep_time {
                    sleep(sleep_time)
                }
            }
        });
    }

    pub fn run(self: Arc<Self>, command_receiver: Receiver<CommandContent>) -> io::Result<()> {
        Arc::clone(&self).start_tick_thread(command_receiver);

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
    let (server, command_receiver) = Server::new()?;
    let server_arc = Arc::new(server);
    server_arc.run(command_receiver)
}

#[cfg(test)]
mod tests {
    use shared::generated::PlayerCommand;
    use shared::generated::{PlayerCommands, PlayerCommandsArgs};

    use super::*;
    use std::net::UdpSocket;
    use std::time::Duration;

    const TEST_SERVER_PORT_START: u16 = 9100;
    static mut NEXT_TEST_PORT: u16 = TEST_SERVER_PORT_START;

    fn get_test_server_addr() -> String {
        // This is safe because tests run sequentially in a single thread
        unsafe {
            let port = NEXT_TEST_PORT;
            NEXT_TEST_PORT += 1;
            format!("127.0.0.1:{}", port)
        }
    }

    #[test]
    fn test_server_creation() {
        let addr = get_test_server_addr();
        let (server, _receiver) = Server::with_addr(&addr).expect("Server should be created");
        assert_eq!(server.socket.local_addr().unwrap().to_string(), addr);
    }

    #[test]
    fn test_player_id_assignment() {
        let addr = get_test_server_addr();
        let (server, _receiver) = Server::with_addr(&addr).expect("Server should be created");
        let addr1 = "127.0.0.1:8001".parse().unwrap();
        let addr2 = "127.0.0.1:8002".parse().unwrap();

        let id1 = server.get_or_add_player_id(&addr1);
        let id2 = server.get_or_add_player_id(&addr2);
        let id1_again = server.get_or_add_player_id(&addr1);

        assert_eq!(id1, 1); // First player gets ID 1
        assert_eq!(id2, 2); // Second player gets ID 2
        assert_eq!(id1_again, id1); // Same player gets same ID
    }

    #[test]
    fn test_handle_packet() {
        let addr = get_test_server_addr();
        let (server, receiver) = Server::with_addr(&addr).expect("Server should be created");
        let client_addr = "127.0.0.1:8001".parse().unwrap();

        // Create a test packet
        let mut builder = FlatBufferBuilder::new();
        let commands = vec![PlayerCommand::MoveRight];
        let commands_vec = builder.create_vector(&commands);
        let player_commands = PlayerCommands::create(
            &mut builder,
            &PlayerCommandsArgs {
                sequence: 1,
                dt_micro: 16667, // ~60fps
                commands: Some(commands_vec),
                client_timestamp_micro: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as u64,
            },
        );
        builder.finish(player_commands, None);
        let packet = builder.finished_data();

        // Handle the packet
        server.handle_packet(packet, client_addr);

        // Check if command was received
        let (player_id, received_commands, _) = receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("Should receive command");

        assert_eq!(player_id, 1); // First player gets ID 1
        assert_eq!(received_commands.sequence, 1);
        assert_eq!(received_commands.commands.len(), 1);
        assert_eq!(received_commands.commands[0], PlayerCommand::MoveRight);
    }
}
