use flatbuffers::{root, FlatBufferBuilder};
use state::GameState;
use std::collections::{HashMap, VecDeque};
use std::io::Result;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex, MutexGuard};
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
    command_queue: Arc<VecDeque<PlayerCommand>>,
    ip_to_player_id: HashMap<SocketAddr, u32>,
    socket: Arc<UdpSocket>,
}

impl Server {
    fn new() -> Result<Server> {
        let socket = Arc::new(UdpSocket::bind(SERVER_ADDR)?);
        println!("UDP running on {}...", SERVER_ADDR);
        Ok(Server {
            state: GameState::new(),
            command_queue: Arc::new(VecDeque::new()),
            ip_to_player_id: HashMap::new(),
            socket,
        })
    }

    fn handle_packet(
        packet: &[u8],
        src_addr: SocketAddr,
        commands: &mut MutexGuard<Vec<(SocketAddr, PlayerCommand)>>,
    ) {
        let player_commands = root::<PlayerCommands>(packet).expect("No command received");
        if let Some(cmd_list) = player_commands.commands() {
            for cmd in cmd_list {
                commands.push((src_addr, cmd));
            }
        }
    }

    // TODO: Replace with ip_to_player
    fn get_player_by_ip<'a>(
        ip: &SocketAddr,
        players: &'a mut MutexGuard<Vec<Player>>,
    ) -> Option<&'a mut Player> {
        players.iter_mut().find(|p| p.ip == *ip)
    }

    fn start_tick_thread(&self) {
        println!("Starting tick thread!");

        let tick_command_queue = Arc::clone(&self.command_queue);
        let tick_socket = Arc::clone(&self.socket);

        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let start = Instant::now();
                let now = Instant::now();
                let dt = now.duration_since(last_tick).as_secs_f32();
                last_tick = now;

                let mut commands_guard = tick_command_queue.lock().unwrap();
                let mut socket = tick_socket.lock().unwrap();
                // Let tick only mutate state
                tick(&mut commands_guard, dt);
                broadcast_state(tick_socket);
                drop(players_guard);
                drop(commands_guard);

                let sleep_time = TICK_DURATION.checked_sub(start.elapsed());
                if let Some(sleep_time) = sleep_time {
                    sleep(sleep_time)
                }
            }
        });
    }

    fn broadcast_state(&self, socket: UdpSocket) {
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
        for p in players.iter() {
            let _ = socket.send_to(bytes, p.ip);
        }
    }

    pub fn run(self) -> Result<()> {
        let players: Arc<Mutex<Vec<Player>>> = Arc::new(Mutex::new(Vec::new()));
        let commands: Arc<Mutex<Vec<(SocketAddr, PlayerCommand)>>> =
            Arc::new(Mutex::new(Vec::new()));

        loop {
            let mut buf = [0u8; 2048];
            let (amt, src_addr) = socket.recv_from(&mut buf)?;

            let mut commands_guard = commands.lock().unwrap();
            handle_packet(&buf[..amt], src_addr, &mut commands_guard);
            drop(commands_guard)
        }
    }

    fn tick(commands: &mut Vec<(SocketAddr, PlayerCommand)>, dt: f32) {
        // Update state with client commands
        for (addr, cmd) in commands.iter() {
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

        commands.clear();
    }
}

fn main() -> Result<()> {
    let server = Server::new()?;
    server.run()
}
