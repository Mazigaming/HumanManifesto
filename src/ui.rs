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
            "Speed: {}  |  {}  |  Agents: {}  |  Grid: {}x{}  |  Seed: {}  |  Land: {}/{} ({}%)  {}",
            sim.speed_label(),
            sim.formatted_date(),
            sim.evo.agents.len(),
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

        // Chronicle section in sidebar
        let chronicle_title_y = top_h + 80.0;
        draw_text(
            "CHRONICLE",
            10.0,
            chronicle_title_y,
            16.0,
            Color::from_rgba(255, 220, 100, 255),
        );

        // Show last 8 chronicle entries
        let chronicle_start_y = chronicle_title_y + 20.0;
        let max_entries = 8;
        let entries_to_show = sim.evo.chronicle.len().min(max_entries);
        let start_idx = sim.evo.chronicle.len().saturating_sub(max_entries);

        for (i, idx) in (start_idx..sim.evo.chronicle.len()).enumerate() {
            let y = chronicle_start_y + (i as f32 * 18.0);
            let entry = &sim.evo.chronicle[idx];
            // Truncate long entries
            let display_text = if entry.len() > 28 {
                format!("{}...", &entry[..25])
            } else {
                entry.clone()
            };
            draw_text(
                &display_text,
                10.0,
                y,
                12.0,
                Color::from_rgba(200, 200, 200, 200),
            );
        }

        // Lineage count
        let lineage_y = chronicle_start_y + (max_entries as f32 * 18.0) + 10.0;
        let lineage_text = format!("Lineages: {}", sim.evo.lineages.len());
        draw_text(
            &lineage_text,
            10.0,
            lineage_y,
            14.0,
            Color::from_rgba(150, 200, 255, 255),
        );
    }
}
