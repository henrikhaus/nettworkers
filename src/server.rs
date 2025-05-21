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

const MAX_PLAYERS: usize = 10;
const TICK_DURATION: Duration = Duration::from_millis(1000);
const SERVER_ADDR: &str = "127.0.0.1:9000";

struct Server {
    state: GameState,
    command_queue: Arc<Mutex<VecDeque<(u32, PlayerCommand)>>>,
    ip_to_player_id: Arc<Mutex<HashMap<SocketAddr, u32>>>,
    socket: UdpSocket,
}

impl Server {
    fn new() -> io::Result<Server> {
        let socket = UdpSocket::bind(SERVER_ADDR)?;
        println!("UDP running on {}...", SERVER_ADDR);
        Ok(Server {
            state: GameState::new(),
            command_queue: Arc::new(Mutex::new(VecDeque::new())),
            ip_to_player_id: Arc::new(Mutex::new(HashMap::new())),
            socket,
        })
    }

    fn handle_packet(&self, packet: &[u8], src_addr: SocketAddr) {
        let player_commands = root::<PlayerCommands>(packet).expect("No command received");

        let mut command_queue_guard = self.command_queue.lock().unwrap();
        if let Some(cmd_list) = player_commands.commands() {
            for cmd in cmd_list {
                let player_id = self.get_or_add_player_id(&src_addr);

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

    fn broadcast_state(&self, ip_to_player_id: &HashMap<SocketAddr, u32>) {
        // Send data to client
        let mut builder = FlatBufferBuilder::with_capacity(2048);
        let players_offsets: Vec<_> = self
            .state
            .players
            .iter()
            .map(|p| {
                PlayerSchema::create(
                    &mut builder,
                    &PlayerArgs {
                        pos: Some(&Vector2::new(p.pos.x, p.pos.y)),
                        vel: None,
                        acc: None,
                        color: p.color,
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

        let tick_server = Arc::clone(self);

        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let start = Instant::now();
                let now = Instant::now();
                let dt = now.duration_since(last_tick).as_secs_f32();
                last_tick = now;

                let mut command_queue_guard = tick_server.command_queue.lock().unwrap();
                let ip_to_player_guard = tick_server.ip_to_player_id.lock().unwrap();

                // Let tick only mutate state
                tick_server.tick(&mut command_queue_guard, dt);
                tick_server.broadcast_state(&ip_to_player_guard);
                drop(command_queue_guard);
                drop(ip_to_player_guard);

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

    fn tick(&self, command_queue: &mut VecDeque<(u32, PlayerCommand)>, dt: f32) {
        // Update state with client commands
        for (addr, cmd) in command_queue.iter() {
            if let Some(player) = get_player_by_ip(addr, players) {
                match cmd {
                    &PlayerCommand::Move_right => handle_move_right(player),
                    &PlayerCommand::Move_left => handle_move_left(player),
                    &PlayerCommand::Jump => handle_jump(player),
                    _ => {}
                }
            } else {
                println!("New player connected: {}", addr);
                players.push(Player::new(*addr));
            }
        }

        // Physics
        let mut accumulator = dt;
        let fixed_dt = 0.016; // 16 ms

        while accumulator > 0.0 {
            let step = accumulator.min(fixed_dt);
            physics(players, step);
            accumulator -= step;
        }

        let player_forces = collision(players);
        for (i, force, pos) in player_forces {
            players[i].vel.x = force.x;
            players[i].vel.y = force.y;
            players[i].pos.x = pos.x;
            players[i].pos.y = pos.y;
        }

        command_queue.clear();
    }
}

fn main() -> io::Result<()> {
    let server = Arc::new(Server::new()?);
    server.run()
}
