use flatbuffers::{root, FlatBufferBuilder};
use std::io::Result;
use std::net::{SocketAddr, UdpSocket};
use std::ops::Index;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[allow(dead_code, unused_imports)]
#[path = "../players_list_generated.rs"]
mod players_list_generated;
use crate::players_list_generated::{Player as SchemaPlayer, PlayerArgs, Color, PlayersList, Vector2};
#[path = "../player_commands_generated.rs"]
mod player_commands_generated;
use crate::player_commands_generated::{PlayerCommand, PlayerCommands};

const MAX_PLAYERS: usize = 10;
const GRAVITY: f32 = 1000.0;
const FRICTION: f32 = 0.8;
const JUMP_CD: f32 = 0.3;
const SCREEN_HEIGHT: usize = 360;
const SCREEN_WIDTH: usize = 640;
const TICK_DURATION: Duration = Duration::from_millis(16);
const SERVER_ADDR: &str = "127.0.0.1:9000";

#[derive(Clone, Copy)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn zero() -> Vec2 {
        Vec2 { x: 0.0, y: 0.0 }
    }
}

struct Player {
    ip: SocketAddr,
    pos: Vec2,
    vel: Vec2,
    acc: f32,
    jump_force: f32,
    jump_timer: f32,
    color: Color,
    size: f32,
}

impl Player {
    fn new(ip: SocketAddr) -> Player {
        Player {
            ip,
            pos: Vec2::zero(),
            vel: Vec2::zero(),
            acc: 0.75,
            jump_force: 400.0,
            jump_timer: 0.0,
            color: Color::Red,
            size: 16.0,
        }
    }
}

fn main() -> Result<()> {
    let socket = Arc::new(UdpSocket::bind(SERVER_ADDR)?);
    println!("UDP running on {}...", SERVER_ADDR);
    let players: Arc<Mutex<Vec<Player>>> = Arc::new(Mutex::new(Vec::new()));
    let commands: Arc<Mutex<Vec<(SocketAddr, PlayerCommand)>>> = Arc::new(Mutex::new(Vec::new()));

    let tick_players = Arc::clone(&players);
    let tick_commands = Arc::clone(&commands);
    let tick_socket = Arc::clone(&socket);

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let start = Instant::now();
            let now = Instant::now();
            let dt = now.duration_since(last_tick).as_secs_f32();
            last_tick = now;

            let mut players_guard = tick_players.lock().unwrap();
            let mut commands_guard = tick_commands.lock().unwrap();
            tick(&mut players_guard, &mut commands_guard, &tick_socket, dt);
            drop(players_guard);
            drop(commands_guard);

            let sleep_time = TICK_DURATION.checked_sub(start.elapsed());
            if let Some(sleep_time) = sleep_time {
                sleep(sleep_time)
            }
        }
    });

    loop {
        let mut buf = [0u8; 2048];
        let (amt, src_addr) = socket.recv_from(&mut buf)?;

        let mut commands_guard = commands.lock().unwrap();
        handle_packet(&buf[..amt], src_addr, &mut commands_guard);
        drop(commands_guard)
    }
}

fn tick(
    players: &mut MutexGuard<Vec<Player>>,
    commands: &mut Vec<(SocketAddr, PlayerCommand)>,
    socket: &UdpSocket,
    dt: f32,
) {
    let mut prev_pos: Vec<(usize, Vec2)> = vec![];
    for (index, p) in players.iter().enumerate() {
        prev_pos.push((index, p.pos))
    }
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

    let mut builder = FlatBufferBuilder::with_capacity(2048);
    let players_offsets: Vec<_> = players
        .iter()
        .map(|p| {
            SchemaPlayer::create(&mut builder,&PlayerArgs {
                pos: Some(&Vector2::new(p.pos.x, p.pos.y)),
                vel: None,
                acc: None,
                color: p.color,
            })
        })
        .collect();

    let players_vec = builder.create_vector(&players_offsets);
    let players_list = PlayersList::create(
        &mut builder,
        &players_list_generated::PlayersListArgs {
            players: Some(players_vec),
        },
    );
    builder.finish(players_list, None);
    let bytes = builder.finished_data();
    for p in players.iter() {
        let _ = socket.send_to(bytes, p.ip);
    }

    commands.clear();
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

fn collision(players: &[Player]) -> Vec<(usize, Vec2, Vec2)> {
    let mut player_forces = vec![];
    for (i, p1) in players.iter().enumerate() {
        for p2 in players {
            if p1.ip == p2.ip {
                continue;
            }

            let v_overlap = p1.pos.y <= p2.pos.y + p2.size && p2.pos.y <= p1.pos.y + p1.size;
            let h_overlap = p1.pos.x <= p2.pos.x + p2.size && p2.pos.x <= p1.pos.x + p1.size;
            let overlap = v_overlap && h_overlap;

            let p1_top = overlap && p1.vel.y > p2.vel.y;
            let p1_bottom = overlap && p1.vel.y < p2.vel.y;
            let p1_left = overlap && p1.vel.x > p2.vel.x;
            let p1_right = overlap && p1.vel.x < p2.vel.x;

            if overlap {
                /*if p1_top {
                    let force = Vec2 { x: (p1.vel.x + p2.vel.x) / 2.0, y: 0.0 };
                    let pos = Vec2 { x: p1.pos.x, y: p2.pos.y - p1.size };
                    player_forces.push((i, force, pos));
                } else*/
                if p1_left {
                    let force = Vec2 {
                        x: (p1.vel.x + p2.vel.x) / 2.0,
                        y: (p1.vel.y + p2.vel.y) / 2.0,
                    };
                    let pos = Vec2 {
                        x: p2.pos.x - p1.size,
                        y: p1.pos.y,
                    };
                    player_forces.push((i, force, pos));
                } else if p1_right {
                    let force = Vec2 {
                        x: (p1.vel.x + p2.vel.x) / 2.0,
                        y: (p1.vel.y + p2.vel.y) / 2.0,
                    };
                    let pos = Vec2 {
                        x: p1.pos.x,
                        y: p1.pos.y,
                    };
                    player_forces.push((i, force, pos));
                }
            }
        }
    }
    player_forces
}

fn physics(players: &mut [Player], dt: f32) {
    for player in players {
        player.pos.x += player.vel.x * dt;
        player.pos.y += player.vel.y * dt;
        player.vel.x *= FRICTION.powf(dt);
        player.vel.y += GRAVITY * dt;
        player.jump_timer += dt;

        if player.pos.y > SCREEN_HEIGHT as f32 - player.size {
            player.pos.y = SCREEN_HEIGHT as f32 - player.size;
            player.vel.y = 0.0;
        }
        if player.pos.y < 0.0 {
            player.pos.y = 0.0;
            player.vel.y = 0.0;
        }
        if player.pos.x > SCREEN_WIDTH as f32 - player.size {
            player.pos.x = SCREEN_WIDTH as f32 - player.size;
            player.vel.x = 0.0;
        }
        if player.pos.x < 0.0 {
            player.pos.x = 0.0;
            player.vel.x = 0.0;
        }
    }
}

fn get_player_by_ip<'a>(
    ip: &SocketAddr,
    players: &'a mut MutexGuard<Vec<Player>>,
) -> Option<&'a mut Player> {
    players.iter_mut().find(|p| p.ip == *ip)
}

fn handle_move_right(player: &mut Player) {
    player.vel.x += player.acc;
}

fn handle_move_left(player: &mut Player) {
    player.vel.x -= player.acc;
}

fn handle_jump(player: &mut Player) {
    if player.pos.y >= SCREEN_HEIGHT as f32 - player.size && player.jump_timer > JUMP_CD {
        player.vel.y -= player.jump_force;
        player.jump_timer = 0.0;
    };
}
