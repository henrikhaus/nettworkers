mod connector;
mod mutate;
mod physics;

use crate::game_state_generated::Color;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;

// Settings
pub const SCREEN_HEIGHT: usize = 360;
pub const SCREEN_WIDTH: usize = 640;

// Player
pub const JUMP_CD: f32 = 0.3;

// Physics
pub const GROUND_FRICTION: f32 = 0.0001;
pub const AIR_FRICTION: f32 = 0.9;
pub const GRAVITY: f32 = 2500.0;
pub const JUMP_FORCE: f32 = 600.0;
pub const PLAYER_ACCELERATION: f32 = 20.0;

#[derive(Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }
    pub fn zero() -> Vec2 {
        Vec2 { x: 0.0, y: 0.0 }
    }
}

#[derive(Clone)]
pub struct PlayerState {
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
}
impl PlayerState {
    fn new(spawn_point: &SpawnPoint) -> PlayerState {
        PlayerState {
            name: "player".to_string(),
            pos: Vec2::new(spawn_point.x, spawn_point.y),
            vel: Vec2::zero(),
            grounded: false,
            jump_timer: 0.0,
            color: Color::Red,
            size: 16.0,
        }
    }
}

#[derive(Clone)]
pub struct GameState {
    pub players: HashMap<u32, PlayerState>,
    pub collidables: Vec<SceneObject>,
    pub width: f32,
    pub height: f32,
    pub spawn_point: SpawnPoint,
}

impl GameState {
    pub fn new(scene_name: &str) -> GameState {
        let file =
            File::open(format!("src/scenes/{}.json", scene_name)).expect("Scene file must open");
        let scene: Scene = serde_json::from_reader(file).expect("JSON must match Scene");
        let collidables: Vec<SceneObject> = scene.collidables.into_values().collect();
        let spawn_point: SpawnPoint = scene.spawn_point.clone();

        GameState {
            players: HashMap::new(),
            collidables,
            width: scene.width,
            height: scene.height,
            spawn_point,
        }
    }

    pub fn update_state(&mut self, new_state: GameState) {
        self.players = new_state.players;
    }
}
