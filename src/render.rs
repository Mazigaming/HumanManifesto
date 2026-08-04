use crate::agent::{Agent, BehaviorState};
use crate::world;
use macroquad::prelude::*;

pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Renderer
    }

    pub fn draw_world(
        &self,
        w: &world::World,
        cam_target_x: f32,
        cam_target_y: f32,
        cam_zoom: f32,
    ) {
        world::draw_world(w, cam_target_x, cam_target_y, cam_zoom);
    }

    // Generate unique lineage color based on lineage_id
    fn lineage_color(lineage_id: u32) -> Color {
        // Use golden angle for distinct hue distribution
        let hue = (lineage_id as f32 * 137.508) % 360.0;
        let saturation = 0.7;
        let value = 0.9;

        // HSV to RGB conversion
        let c = value * saturation;
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
        let m = value - c;

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

    pub fn draw_agents(
        &self,
        agents: &[Agent],
        world: &world::World,
        cam_target_x: f32,
        cam_target_y: f32,
        cam_zoom: f32,
    ) {
        let hex_size = 16.0;
        let screen_w = screen_width();
        let screen_h = screen_height();
        let view_w = screen_w / cam_zoom;
        let view_h = screen_h / cam_zoom;
        let margin = hex_size * 3.0;

        // Skip rendering individual agents when zoomed out too far or too many agents
        let agent_count = agents.len();
        let min_agent_size = 2.0;
        let skip_rendering = cam_zoom < 0.3 || (agent_count > 500 && cam_zoom < 0.5);

        for agent in agents {
            // Convert tile position to world position
            let center = world::hex_center(agent.col, agent.row, hex_size);

            // Frustum culling
            if (center.x - cam_target_x).abs() > view_w / 2.0 + margin {
                continue;
            }
            if (center.y - cam_target_y).abs() > view_h / 2.0 + margin {
                continue;
            }

            // Get biome color for camouflage blending
            let biome_color = if let Some(tile) = world.get_tile(agent.col, agent.row) {
                tile.biome.color()
            } else {
                Color::from_rgba(100, 100, 100, 255)
            };

            // Lineage base color
            let lineage_base = Self::lineage_color(agent.lineage_id);

            // Enhance phenotype with lineage color and trait-based variation
            let g = &agent.genome;
            let r = ((lineage_base.r as f32 * 0.6 + g.aggression * 100.0 + g.speed * 50.0) as u8)
                .min(255);
            let gr = ((lineage_base.g as f32 * 0.6 + g.metabolism * 80.0 + g.camouflage * 60.0)
                as u8)
                .min(255);
            let b = ((lineage_base.b as f32 * 0.6 + g.sociability * 100.0 + g.sight_range * 40.0)
                as u8)
                .min(255);

            let base_color = Color::from_rgba(r, gr, b, 255);

            // Apply camouflage blending with biome
            let camo = g.camouflage;
            let agent_color = Color::from_rgba(
                (base_color.r as f32 * (1.0 - camo) + biome_color.r as f32 * camo) as u8,
                (base_color.g as f32 * (1.0 - camo) + biome_color.g as f32 * camo) as u8,
                (base_color.b as f32 * (1.0 - camo) + biome_color.b as f32 * camo) as u8,
                255,
            );

            // Size scales more dramatically with strength (2.0 to 7.0)
            let size = 2.0 + g.strength * 5.0;

            if skip_rendering {
                // Simplified rendering when zoomed out or many agents
                draw_circle(center.x, center.y, min_agent_size, agent_color);
            } else {
                // Full rendering with details
                draw_circle(center.x, center.y, size, agent_color);

                // Draw inner detail based on dominant trait
                let dominant_trait = g.speed.max(g.strength).max(g.aggression).max(g.sociability);
                let inner_size = size * 0.5;

                if g.aggression == dominant_trait {
                    // Aggressive: red triangle
                    draw_triangle(
                        vec2(center.x, center.y - inner_size),
                        vec2(center.x - inner_size * 0.866, center.y + inner_size * 0.5),
                        vec2(center.x + inner_size * 0.866, center.y + inner_size * 0.5),
                        Color::from_rgba(255, 50, 50, 200),
                    );
                } else if g.sociability == dominant_trait {
                    // Social: blue circle
                    draw_circle(
                        center.x,
                        center.y,
                        inner_size * 0.7,
                        Color::from_rgba(50, 150, 255, 200),
                    );
                } else if g.speed == dominant_trait {
                    // Fast: green diamond
                    draw_triangle(
                        vec2(center.x, center.y - inner_size),
                        vec2(center.x + inner_size * 0.7, center.y),
                        vec2(center.x, center.y + inner_size),
                        Color::from_rgba(50, 255, 100, 200),
                    );
                    draw_triangle(
                        vec2(center.x, center.y - inner_size),
                        vec2(center.x - inner_size * 0.7, center.y),
                        vec2(center.x, center.y + inner_size),
                        Color::from_rgba(50, 255, 100, 200),
                    );
                } else {
                    // Strong: yellow square
                    draw_rectangle(
                        center.x - inner_size * 0.7,
                        center.y - inner_size * 0.7,
                        inner_size * 1.4,
                        inner_size * 1.4,
                        Color::from_rgba(255, 200, 50, 200),
                    );
                }

                // State ring color (more distinct)
                let ring_color = match agent.behavior {
                    BehaviorState::Fleeing => Color::from_rgba(255, 50, 50, 255),
                    BehaviorState::SeekingFood | BehaviorState::SeekingWater => {
                        Color::from_rgba(50, 255, 50, 255)
                    }
                    BehaviorState::Reproducing => Color::from_rgba(255, 150, 50, 255),
                    BehaviorState::Socializing => Color::from_rgba(50, 150, 255, 255),
                    BehaviorState::Fighting => Color::from_rgba(255, 0, 0, 255),
                    BehaviorState::Infected => Color::from_rgba(150, 0, 255, 255),
                    _ => Color::from_rgba(150, 150, 150, 150),
                };

                // Draw state ring (thicker for visibility)
                draw_circle_lines(center.x, center.y, size + 2.0, 1.5, ring_color);

                // Golden mutation highlight (more pronounced)
                if agent.highlight_timer > 0 {
                    let flash_alpha = (agent.highlight_timer as f32 / 120.0) * 255.0;
                    draw_circle(
                        center.x,
                        center.y,
                        size + 4.0,
                        Color::from_rgba(255, 255, 0, flash_alpha as u8),
                    );
                    // Sparkle effect
                    for i in 0..4 {
                        let angle = (i as f32 / 4.0) * std::f32::consts::TAU;
                        let spark_x = center.x + angle.cos() * (size + 3.0);
                        let spark_y = center.y + angle.sin() * (size + 3.0);
                        draw_circle(
                            spark_x,
                            spark_y,
                            1.5,
                            Color::from_rgba(255, 255, 200, flash_alpha as u8),
                        );
                    }
                }

                // Disease indicator (more visible)
                if agent.disease.infected {
                    draw_circle(
                        center.x,
                        center.y,
                        size * 0.6,
                        Color::from_rgba(150, 0, 255, 220),
                    );
                    // Pulse effect
                    let pulse = (agent.disease.ticks_infected % 20) as f32 / 20.0;
                    draw_circle_lines(
                        center.x,
                        center.y,
                        size * (0.8 + pulse * 0.4),
                        1.0,
                        Color::from_rgba(200, 50, 255, 150),
                    );
                }

                // Health indicator (red outline when low health)
                if agent.health < 50.0 {
                    let health_alpha = (1.0 - agent.health / 50.0) * 200.0;
                    draw_circle_lines(
                        center.x,
                        center.y,
                        size + 3.0,
                        2.0,
                        Color::from_rgba(255, 0, 0, health_alpha as u8),
                    );
                }
            }
        }
    }
}
