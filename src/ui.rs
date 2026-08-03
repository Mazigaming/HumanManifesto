use crate::sim::Sim;
use macroquad::prelude::*;

pub struct UI {
    pub map_width: i32,
    pub map_height: i32,
    pub use_same_seed: bool,
}

impl UI {
    pub fn new() -> Self {
        UI {
            map_width: 200,
            map_height: 200,
            use_same_seed: false,
        }
    }

    pub fn draw(&self, sim: &Sim) {
        let pause_text = if sim.paused { "PAUSED" } else { "" };

        let land_count = sim
            .world
            .tiles
            .iter()
            .filter(|t| t.biome != crate::world::Biome::Ocean)
            .count();

        let total = sim.world.tiles.len();

        let status = format!(
            "Speed: {}  |  Time: {:.1}s  |  Grid: {}x{}  |  Seed: {}  |  Land: {}/{} ({}%)  {}",
            sim.speed_label(),
            sim.sim_time,
            sim.world.width,
            sim.world.height,
            sim.world.seed,
            land_count,
            total,
            if total > 0 {
                land_count * 100 / total
            } else {
                0
            },
            pause_text,
        );

        let sw = screen_width();
        let sh = screen_height();
        let top_h = 50.0;
        let side_w = 200.0;
        // Top status bar
        draw_rectangle(0.0, 0.0, sw, top_h, Color::from_rgba(20, 20, 20, 200));
        // Left sidebar panel
        draw_rectangle(
            0.0,
            top_h,
            side_w,
            sh - top_h,
            Color::from_rgba(30, 30, 30, 200),
        );
        // Status text
        draw_text(&status, 10.0, 30.0, 22.0, WHITE);
        // Controls/help in sidebar
        let help = "[Space] Pause [0/1/2] Speed [WASD/Arrows] Pan [Shift] Fast pan [Scroll] Zoom [C] Center";
        let gen_help = format!(
            "[R] Regenerate (seed: {})  [[/]] W:{} H:{}  [Tab] same-seed:{}",
            if self.use_same_seed { "SAME" } else { "NEW" },
            self.map_width,
            self.map_height,
            if self.use_same_seed { "ON" } else { "OFF" },
        );
        draw_text(
            help,
            10.0,
            top_h + 30.0,
            16.0,
            Color::from_rgba(200, 200, 200, 200),
        );
        draw_text(
            &gen_help,
            10.0,
            top_h + 50.0,
            14.0,
            Color::from_rgba(180, 180, 180, 180),
        );
    }
}
