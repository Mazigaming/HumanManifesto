use crate::evolution::EvolutionSim;
use crate::world::World;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Sim {
    pub world: World,
    pub evo: EvolutionSim,
    pub paused: bool,
    pub speed_idx: usize,
    pub accumulator: f64,
    pub sim_time: f64,
    pub day_count: u64,
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
        let mut evo = EvolutionSim::new();

        // Spawn initial population at origin points
        for i in 0..world.tiles.len() {
            if world.tiles[i].origin_point.is_some() {
                let col = world.tiles[i].col;
                let row = world.tiles[i].row;
                evo.spawn_population(col, row, 20);
            }
        }

        // If no origin points, spawn in center
        if evo.agents.is_empty() {
            let center_col = grid_width / 2;
            let center_row = grid_height / 2;
            evo.spawn_population(center_col, center_row, 50);
        }

        Sim {
            world,
            evo,
            paused: false,
            speed_idx: 0,
            accumulator: 0.0,
            sim_time: 0.0,
            day_count: 0,
            is_loading: false,
            loading_message: String::new(),
        }
    }

    pub fn regenerate(&mut self, grid_width: i32, grid_height: i32, seed: u64) {
        self.is_loading = true;
        self.loading_message = format!("Generating world ({}x{})...", grid_width, grid_height);
        self.world = World::generate(grid_width, grid_height, seed);
        self.evo = EvolutionSim::new();

        // Spawn initial population
        for i in 0..self.world.tiles.len() {
            if self.world.tiles[i].origin_point.is_some() {
                let col = self.world.tiles[i].col;
                let row = self.world.tiles[i].row;
                self.evo.spawn_population(col, row, 20);
            }
        }

        if self.evo.agents.is_empty() {
            let center_col = grid_width / 2;
            let center_row = grid_height / 2;
            self.evo.spawn_population(center_col, center_row, 50);
        }

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
        // 1 second of real time = 1 day at 1x speed
        self.day_count = self.sim_time.floor() as u64;
        self.evo.tick(&mut self.world);
    }

    pub fn formatted_date(&self) -> String {
        let years = self.day_count / 365;
        let months = (self.day_count % 365) / 30;
        let days = self.day_count % 30;
        format!("Year {}, Month {}, Day {}", years + 1, months + 1, days + 1)
    }
}
