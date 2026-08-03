use crate::world::World;
use macroquad::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Sim {
    pub world: World,
    pub paused: bool,
    pub speed_idx: usize,
    pub accumulator: f64,
    pub sim_time: f64,
    pub is_loading: bool,
    pub loading_message: String,
}

impl Sim {
    pub fn new(grid_width: i32, grid_height: i32) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let world = World::generate(grid_width, grid_height, seed);
        Sim {
            world,
            paused: false,
            speed_idx: 0,
            accumulator: 0.0,
            sim_time: 0.0,
            is_loading: false,
            loading_message: String::new(),
        }
    }

    pub fn regenerate(&mut self, grid_width: i32, grid_height: i32, seed: u64) {
        self.is_loading = true;
        self.loading_message = format!("Generating world ({}x{})...", grid_width, grid_height);
        self.world = World::generate(grid_width, grid_height, seed);
        self.sim_time = 0.0;
        self.accumulator = 0.0;
        self.is_loading = false;
    }

    pub fn random_seed(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    pub fn speed_multiplier(&self) -> f64 {
        match self.speed_idx {
            0 => 1.0,
            1 => 10.0,
            2 => 100.0,
            _ => 1.0,
        }
    }

    pub fn speed_label(&self) -> &'static str {
        match self.speed_idx {
            0 => "1x",
            1 => "10x",
            2 => "100x",
            _ => "?",
        }
    }

    pub fn tick(&mut self, _dt: f64) {
        self.sim_time += _dt;
    }
}
