mod mapper;
mod mutate;
mod physics;

use crate::generated::{self, Color};
use serde::Deserialize;
use std::collections::{BinaryHeap, HashMap};
use std::fmt::Display;
use std::fs::File;
use std::ops::{Add, Mul, Sub};

// Settings
pub const SCREEN_HEIGHT: usize = 360;
pub const SCREEN_WIDTH: usize = 640;

// Player
pub const JUMP_CD: f32 = 0.3;

// Physics
pub const GROUND_FRICTION: f32 = 0.0001;
pub const AIR_FRICTION: f32 = 0.9;
pub const GRAVITY: f32 = 2200.0;
pub const JUMP_FORCE: f32 = 800.0;
pub const PLAYER_ACCELERATION: f32 = 3.0;

#[derive(Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, other: f32) -> Vec2 {
        Vec2 {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

#[derive(Clone)]
pub struct PlayerState {
    pub id: u32,
    pub name: String,
    pub pos: Vec2,
    pub vel: Vec2,
    pub grounded: bool,
    pub jump_timer: f32,
    pub color: Color,
    pub size: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SceneObject {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SpawnPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Deserialize)]
struct Scene {
    collidables: HashMap<u32, SceneObject>,
    width: f32,
    height: f32,
    spawn_point: SpawnPoint,
    win_point: SceneObject,
}

impl PlayerState {
    fn new(id: u32, spawn_point: &SpawnPoint) -> PlayerState {
        PlayerState {
            id,
            name: "player".to_string(),
            pos: Vec2::new(spawn_point.x, spawn_point.y),
            vel: Vec2::ZERO,
            grounded: false,
            jump_timer: 0.0,
            color: Color::Red,
            size: 16.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerStateCommand {
    pub sequence: u32,
    pub dt_micros: u64,
    // Mutliple commands because the player can for example jump and move in the same frame
    pub commands: Vec<generated::PlayerCommand>,
    pub client_timestamp_micros: u64,
}

#[derive(Clone)]
pub struct GameState {
    pub players: HashMap<u32, PlayerState>,
    pub collidables: Vec<SceneObject>,
    pub width: f32,
    pub height: f32,
    pub spawn_point: SpawnPoint,
    pub win_point: SceneObject,
    pub cached_dt_micros: u64,
    pub scheduled_commands: BinaryHeap<mutate::ScheduledCommand>,
}

#[derive(Clone)]
pub struct CommandContent {
    pub player_id: u32,
    pub player_state_command: PlayerStateCommand,
    pub client_delay_micros: u64,
}

impl GameState {
    pub fn new(scene_name: &str) -> GameState {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let file = File::open(format!("{}/../scenes/{}.json", project_root, scene_name))
            .expect("Scene file must open");
        let scene: Scene = serde_json::from_reader(file).expect("JSON must match Scene");
        let collidables: Vec<SceneObject> = scene.collidables.into_values().collect();
        let spawn_point: SpawnPoint = scene.spawn_point.clone();
        let win_point: SceneObject = scene.win_point.clone();

        GameState {
            players: HashMap::new(),
            collidables,
            width: scene.width,
            height: scene.height,
            spawn_point,
            win_point,
            cached_dt_micros: 0,
            scheduled_commands: BinaryHeap::new(),
        }
    }

    pub fn update_state(&mut self, new_state: GameState) {
        self.players = new_state.players;
    }
}
