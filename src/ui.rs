use crate::belief::Belief;
use crate::civilization::build_focus_tree;
use crate::sim::Sim;
use crate::tribe::Tribe;
use macroquad::prelude::*;

pub struct UI {
    pub map_width: i32,
    pub map_height: i32,
    pub use_same_seed: bool,
    pub show_gene_pool: bool,
    pub show_tribes: bool,
    pub show_civilization: bool,
}

impl UI {
    pub fn new() -> Self {
        UI {
            map_width: 200,
            map_height: 200,
            use_same_seed: false,
            show_gene_pool: false,
            show_tribes: false,
            show_civilization: false,
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

        // Gene pool viewer toggle hint
        let gene_pool_hint_y = lineage_y + 20.0;
        draw_text(
            "[G] Gene Pool",
            10.0,
            gene_pool_hint_y,
            14.0,
            Color::from_rgba(180, 180, 180, 200),
        );

        let tribe_hint_y = gene_pool_hint_y + 18.0;
        draw_text(
            "[T] Tribes",
            10.0,
            tribe_hint_y,
            14.0,
            Color::from_rgba(180, 180, 180, 200),
        );

        let civ_hint_y = tribe_hint_y + 18.0;
        draw_text(
            "[V] Civilization",
            10.0,
            civ_hint_y,
            14.0,
            Color::from_rgba(180, 180, 180, 200),
        );

        if self.show_gene_pool {
            self.draw_gene_pool(sim);
        }

        if self.show_tribes {
            self.draw_tribe_panel(sim);
        }

        if self.show_civilization {
            self.draw_civilization_panel(sim);
        }

        self.draw_minimap(sim);
    }

    pub fn draw_gene_pool(&self, sim: &Sim) {
        let agents = &sim.evo.agents;
        if agents.is_empty() {
            return;
        }

        let sw = screen_width();
        let sh = screen_height();
        let panel_w = 320.0;
        let panel_h = 420.0;
        let panel_x = sw - panel_w - 20.0;
        let panel_y = 70.0;

        draw_rectangle(
            panel_x,
            panel_y,
            panel_w,
            panel_h,
            Color::from_rgba(20, 20, 30, 230),
        );
        draw_rectangle_lines(
            panel_x,
            panel_y,
            panel_w,
            panel_h,
            2.0,
            Color::from_rgba(100, 200, 255, 255),
        );

        let mut y = panel_y + 20.0;
        let line_h = 16.0;

        draw_text(
            "GENE POOL",
            panel_x + 10.0,
            y,
            18.0,
            Color::from_rgba(255, 220, 100, 255),
        );
        y += 25.0;

        draw_text(
            &format!("Population: {}", agents.len()),
            panel_x + 10.0,
            y,
            14.0,
            WHITE,
        );
        y += line_h;

        let mut male_count = 0;
        let mut female_count = 0;
        let mut pregnant_count = 0;
        let mut hetero = 0;
        let mut bi = 0;
        let mut homo = 0;
        let mut trait_sums = [0.0; 17];
        let trait_names = [
            "Speed",
            "Strength",
            "Fertility",
            "Metabolism",
            "Aggression",
            "Sociability",
            "Camouflage",
            "Lifespan",
            "Sight",
            "ColdTol",
            "HeatTol",
            "Sexuality",
            "Intelligence",
            "Curiosity",
            "Conformity",
            "Creativity",
            "Leadership",
        ];

        for agent in agents {
            match agent.gender {
                crate::agent::Gender::Male => male_count += 1,
                crate::agent::Gender::Female => female_count += 1,
            }
            if agent.pregnancy_days > 0 {
                pregnant_count += 1;
            }
            let s = agent.genome.sexuality;
            if s < 0.3 {
                hetero += 1;
            } else if s < 0.7 {
                bi += 1;
            } else {
                homo += 1;
            }
            trait_sums[0] += agent.genome.speed;
            trait_sums[1] += agent.genome.strength;
            trait_sums[2] += agent.genome.fertility;
            trait_sums[3] += agent.genome.metabolism;
            trait_sums[4] += agent.genome.aggression;
            trait_sums[5] += agent.genome.sociability;
            trait_sums[6] += agent.genome.camouflage;
            trait_sums[7] += agent.genome.lifespan;
            trait_sums[8] += agent.genome.sight_range;
            trait_sums[9] += agent.genome.cold_tolerance;
            trait_sums[10] += agent.genome.heat_tolerance;
            trait_sums[11] += agent.genome.sexuality;
            trait_sums[12] += agent.genome.intelligence;
            trait_sums[13] += agent.genome.curiosity;
            trait_sums[14] += agent.genome.conformity;
            trait_sums[15] += agent.genome.creativity;
            trait_sums[16] += agent.genome.leadership;
        }

        let n = agents.len() as f32;
        let male_pct = male_count as f32 / n * 100.0;
        let female_pct = female_count as f32 / n * 100.0;
        let hetero_pct = hetero as f32 / n * 100.0;
        let bi_pct = bi as f32 / n * 100.0;
        let homo_pct = homo as f32 / n * 100.0;

        draw_text(
            &format!(
                "Gender: {}M {}F ({:.0}%/{:.0}%)",
                male_count, female_count, male_pct, female_pct
            ),
            panel_x + 10.0,
            y,
            13.0,
            Color::from_rgba(200, 200, 200, 255),
        );
        y += line_h;

        draw_text(
            &format!("Pregnant: {}", pregnant_count),
            panel_x + 10.0,
            y,
            13.0,
            Color::from_rgba(200, 200, 200, 255),
        );
        y += line_h;

        draw_text(
            &format!(
                "Sexuality: {}% hetero {}% bi {}% homo",
                hetero_pct as u32, bi_pct as u32, homo_pct as u32
            ),
            panel_x + 10.0,
            y,
            13.0,
            Color::from_rgba(200, 200, 200, 255),
        );
        y += line_h + 4.0;

        draw_text(
            "AVERAGE TRAITS",
            panel_x + 10.0,
            y,
            14.0,
            Color::from_rgba(255, 220, 100, 255),
        );
        y += line_h;

        for (i, name) in trait_names.iter().enumerate() {
            let avg = trait_sums[i] / n;
            let bar_w = (avg * 80.0) as f32;
            draw_text(
                &format!("{}: {:.2}", name, avg),
                panel_x + 10.0,
                y,
                12.0,
                WHITE,
            );
            draw_rectangle(
                panel_x + 90.0,
                y - 10.0,
                bar_w,
                8.0,
                Color::from_rgba((avg * 255.0) as u8, 150, ((1.0 - avg) * 255.0) as u8, 200),
            );
            draw_rectangle_lines(
                panel_x + 90.0,
                y - 10.0,
                80.0,
                8.0,
                1.0,
                Color::from_rgba(100, 100, 100, 150),
            );
            y += line_h;
        }

        y += 6.0;
        draw_text(
            "DOMINANT LINEAGES",
            panel_x + 10.0,
            y,
            14.0,
            Color::from_rgba(255, 220, 100, 255),
        );
        y += line_h;

        let mut lineage_pops: Vec<(u32, usize, String)> = sim
            .evo
            .lineages
            .iter()
            .map(|l| (l.id, l.member_count, l.name.clone()))
            .collect();
        lineage_pops.sort_by(|a, b| b.1.cmp(&a.1));
        let top = lineage_pops.iter().take(5);
        for (id, count, name) in top {
            draw_text(
                &format!("{}: {} (id:{})", name, count, id),
                panel_x + 10.0,
                y,
                12.0,
                Color::from_rgba(180, 200, 255, 255),
            );
            y += line_h;
        }
    }

    pub fn draw_tribe_panel(&self, sim: &Sim) {
        let tribes = &sim.evo.tribes;
        if tribes.is_empty() {
            return;
        }

        let sw = screen_width();
        let panel_w = 300.0;
        let panel_h = 350.0;
        let panel_x = sw - panel_w - 20.0;
        let panel_y = 70.0;

        draw_rectangle(
            panel_x,
            panel_y,
            panel_w,
            panel_h,
            Color::from_rgba(20, 20, 30, 230),
        );
        draw_rectangle_lines(
            panel_x,
            panel_y,
            panel_w,
            panel_h,
            2.0,
            Color::from_rgba(100, 200, 255, 255),
        );

        let mut y = panel_y + 20.0;
        let line_h = 16.0;

        draw_text(
            "TRIBES",
            panel_x + 10.0,
            y,
            18.0,
            Color::from_rgba(255, 220, 100, 255),
        );
        y += 25.0;

        draw_text(
            &format!("Active Tribes: {}", tribes.len()),
            panel_x + 10.0,
            y,
            14.0,
            WHITE,
        );
        y += line_h;

        draw_text(
            &format!("Beliefs: {}", sim.evo.beliefs.len()),
            panel_x + 10.0,
            y,
            14.0,
            Color::from_rgba(200, 200, 200, 255),
        );
        y += line_h;

        draw_text(
            &format!(
                "Diplomatic Relations: {}",
                sim.evo.diplomatic_relations.len()
            ),
            panel_x + 10.0,
            y,
            14.0,
            Color::from_rgba(200, 200, 200, 255),
        );
        y += line_h + 4.0;

        draw_text(
            "TOP TRIBES",
            panel_x + 10.0,
            y,
            14.0,
            Color::from_rgba(255, 220, 100, 255),
        );
        y += line_h;

        let mut tribe_pops: Vec<(&Tribe, usize, Color)> = tribes
            .iter()
            .map(|t| (t, t.member_count(), t.culture_profile.color()))
            .collect();
        tribe_pops.sort_by(|a, b| b.1.cmp(&a.1));
        let top = tribe_pops.iter().take(6);
        for (tribe, count, color) in top {
            draw_text(
                &format!(
                    "{}: {} | Know: {:.0}",
                    if tribe.name.len() > 10 {
                        &tribe.name[..10]
                    } else {
                        &tribe.name
                    },
                    count,
                    tribe.knowledge
                ),
                panel_x + 10.0,
                y,
                12.0,
                *color,
            );
            y += line_h;
        }

        y += 6.0;
        draw_text(
            "BELIEFS",
            panel_x + 10.0,
            y,
            14.0,
            Color::from_rgba(255, 220, 100, 255),
        );
        y += line_h;

        let mut belief_pops: Vec<(&Belief, usize)> = sim
            .evo
            .beliefs
            .iter()
            .map(|b| {
                let adherents = sim
                    .evo
                    .agents
                    .iter()
                    .filter(|a| a.belief_id == Some(b.id))
                    .count();
                (b, adherents)
            })
            .collect();
        belief_pops.sort_by(|a, b| b.1.cmp(&a.1));
        for (belief, count) in belief_pops.iter().take(5) {
            draw_text(
                &format!("{}: {}", belief.name, count),
                panel_x + 10.0,
                y,
                12.0,
                Color::from_rgba(255, 200, 150, 255),
            );
            y += line_h;
        }
    }

    pub fn draw_civilization_panel(&self, sim: &Sim) {
        let civs = &sim.evo.civilizations;
        if civs.is_empty() {
            return;
        }

        let sw = screen_width();
        let panel_w = 360.0;
        let panel_h = 500.0;
        let panel_x = sw - panel_w - 20.0;
        let panel_y = 70.0;

        draw_rectangle(
            panel_x,
            panel_y,
            panel_w,
            panel_h,
            Color::from_rgba(20, 20, 30, 230),
        );
        draw_rectangle_lines(
            panel_x,
            panel_y,
            panel_w,
            panel_h,
            2.0,
            Color::from_rgba(255, 220, 100, 255),
        );

        let mut y = panel_y + 20.0;
        let line_h = 16.0;

        for civ in civs.iter().take(1) {
            draw_text(
                &format!("CIVILIZATION: {}", civ.identity.name),
                panel_x + 10.0,
                y,
                18.0,
                Color::from_rgba(255, 220, 100, 255),
            );
            y += line_h + 4.0;

            draw_text(
                &format!(
                    "Trait: {} | Era: {}",
                    civ.identity.primary_trait, civ.current_era
                ),
                panel_x + 10.0,
                y,
                13.0,
                WHITE,
            );
            y += line_h;

            draw_text(
                &format!(
                    "Gov: {} | Pop: {} | Stability: {:.0}%",
                    civ.government_type,
                    civ.total_population(),
                    civ.stability
                ),
                panel_x + 10.0,
                y,
                13.0,
                WHITE,
            );
            y += line_h;

            draw_text(
                &format!("Divine Influence: {:.0}", civ.divine_influence),
                panel_x + 10.0,
                y,
                14.0,
                Color::from_rgba(255, 200, 100, 255),
            );
            y += line_h + 6.0;

            draw_text(
                "IDEOLOGY",
                panel_x + 10.0,
                y,
                14.0,
                Color::from_rgba(255, 220, 100, 255),
            );
            y += line_h;

            let ideology_labels = vec![
                ("Authority", civ.ideology.authority),
                ("Equality", civ.ideology.equality),
                ("Tradition", civ.ideology.tradition),
                ("Spirituality", civ.ideology.spirituality),
                ("Militarism", civ.ideology.militarism),
                ("Individualism", civ.ideology.individualism),
            ];

            for (label, value) in ideology_labels {
                let bar_w = (value * 80.0) as f32;
                draw_text(
                    &format!("{}: {:.0}%", label, value * 100.0),
                    panel_x + 10.0,
                    y,
                    11.0,
                    WHITE,
                );
                draw_rectangle(
                    panel_x + 80.0,
                    y - 8.0,
                    bar_w,
                    6.0,
                    Color::from_rgba(
                        (value * 255.0) as u8,
                        150,
                        ((1.0 - value) * 255.0) as u8,
                        200,
                    ),
                );
                draw_rectangle_lines(
                    panel_x + 80.0,
                    y - 8.0,
                    80.0,
                    6.0,
                    1.0,
                    Color::from_rgba(100, 100, 100, 150),
                );
                y += line_h;
            }

            y += 4.0;

            if !civ.cities.is_empty() {
                draw_text(
                    "CITIES",
                    panel_x + 10.0,
                    y,
                    14.0,
                    Color::from_rgba(255, 220, 100, 255),
                );
                y += line_h;

                for city in civ.cities.iter().take(4) {
                    draw_text(
                        &format!("{} (pop: {})", city.name, city.population),
                        panel_x + 10.0,
                        y,
                        11.0,
                        WHITE,
                    );
                    y += line_h;
                }
                y += 4.0;
            }

            if !civ.history.is_empty() {
                draw_text(
                    "HISTORY",
                    panel_x + 10.0,
                    y,
                    14.0,
                    Color::from_rgba(255, 220, 100, 255),
                );
                y += line_h;

                let start_idx = civ.history.len().saturating_sub(4);
                for entry in civ.history[start_idx..].iter().rev() {
                    let display = if entry.len() > 30 {
                        format!("{}...", &entry[..27])
                    } else {
                        entry.clone()
                    };
                    draw_text(
                        &display,
                        panel_x + 10.0,
                        y,
                        10.0,
                        Color::from_rgba(200, 200, 200, 200),
                    );
                    y += line_h - 2.0;
                }
            }

            y += 6.0;
            draw_text(
                "AVAILABLE FOCUSES",
                panel_x + 10.0,
                y,
                14.0,
                Color::from_rgba(255, 220, 100, 255),
            );
            y += line_h;

            let all_focuses = build_focus_tree();
            let available = civ.available_focuses(&all_focuses);
            if available.is_empty() {
                draw_text(
                    "No focuses available yet",
                    panel_x + 10.0,
                    y,
                    11.0,
                    Color::from_rgba(150, 150, 150, 200),
                );
                y += line_h;
            } else {
                for focus in available.iter().take(5) {
                    let afford = civ.divine_influence >= focus.cost;
                    draw_text(
                        &format!(
                            "[{}] {} (cost: {:.0})",
                            if afford { "1-5" } else { "??" },
                            focus.name,
                            focus.cost
                        ),
                        panel_x + 10.0,
                        y,
                        11.0,
                        if afford {
                            WHITE
                        } else {
                            Color::from_rgba(100, 100, 100, 150)
                        },
                    );
                    y += line_h - 2.0;
                }
                if available.len() > 5 {
                    draw_text(
                        &format!("...and {} more", available.len() - 5),
                        panel_x + 10.0,
                        y,
                        10.0,
                        Color::from_rgba(150, 150, 150, 200),
                    );
                    y += line_h - 2.0;
                }
            }
        }
    }

    fn hue_to_color(hue: f32) -> Color {
        let c = 0.8;
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
        let m = 0.2;
        let (r, g, b) = if hue < 60.0 {
            (c, x, 0.0)
        } else if hue < 120.0 {
            (x, c, 0.0)
        } else if hue < 180.0 {
            (0.0, c, x)
        } else if hue < 240.0 {
            (0.0, x, c)
        } else if hue < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        Color::from_rgba(
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
            255,
        )
    }

    pub fn draw_minimap(&self, sim: &Sim) {
        let sw = screen_width();
        let sh = screen_height();
        let map_w = 180.0;
        let map_h = 120.0;
        let map_x = sw - map_w - 20.0;
        let map_y = sh - map_h - 20.0;

        draw_rectangle(map_x, map_y, map_w, map_h, Color::from_rgba(0, 0, 0, 180));
        draw_rectangle_lines(
            map_x,
            map_y,
            map_w,
            map_h,
            1.0,
            Color::from_rgba(100, 100, 100, 255),
        );

        let world = &sim.world;
        let scale_x = map_w / world.width as f32;
        let scale_y = map_h / world.height as f32;

        for tile in &world.tiles {
            if tile.biome == crate::world::Biome::Ocean {
                continue;
            }
            let x = map_x + tile.col as f32 * scale_x;
            let y = map_y + tile.row as f32 * scale_y;
            let color = tile.biome.color();
            let alpha = if tile.resource.is_some() { 1.0 } else { 0.6 };
            let mut final_color = color;
            final_color.a = alpha;
            draw_rectangle(x, y, scale_x.max(1.0), scale_y.max(1.0), final_color);
        }

        for agent in &sim.evo.agents {
            let x = map_x + agent.col as f32 * scale_x;
            let y = map_y + agent.row as f32 * scale_y;
            let color = if let Some(tribe_id) = agent.tribe_id {
                let hue = (tribe_id as f32 * 137.508 * 1.5) % 360.0;
                Self::hue_to_color(hue)
            } else {
                WHITE
            };
            draw_circle(x, y, 1.5, color);
        }

        draw_text(
            "MINIMAP",
            map_x + 5.0,
            map_y + 12.0,
            10.0,
            Color::from_rgba(200, 200, 200, 200),
        );

        let legend_y = map_y + map_h - 10.0;
        draw_text(
            "Terrain  Resources  Agents",
            map_x + 5.0,
            legend_y,
            8.0,
            Color::from_rgba(150, 150, 150, 200),
        );
    }
}
