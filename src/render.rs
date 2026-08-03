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

            // Agent color with camouflage
            let agent_color = agent.genome.camouflaged_color(biome_color);

            // Size scales with strength
            let size = 3.0 + agent.genome.strength * 3.0;

            // Draw agent body
            draw_circle(center.x, center.y, size, agent_color);

            // State ring color
            let ring_color = match agent.behavior {
                BehaviorState::Fleeing => Color::from_rgba(255, 100, 100, 200),
                BehaviorState::SeekingFood | BehaviorState::SeekingWater => {
                    Color::from_rgba(100, 255, 100, 200)
                }
                BehaviorState::Reproducing => Color::from_rgba(255, 200, 100, 200),
                BehaviorState::Socializing => Color::from_rgba(100, 200, 255, 200),
                BehaviorState::Fighting => Color::from_rgba(255, 50, 50, 200),
                BehaviorState::Infected => Color::from_rgba(150, 50, 200, 200),
                _ => Color::from_rgba(200, 200, 200, 100),
            };

            // Draw state ring
            draw_circle_lines(center.x, center.y, size + 1.5, 1.0, ring_color);

            // Golden mutation highlight
            if agent.highlight_timer > 0 {
                let flash_alpha = (agent.highlight_timer as f32 / 120.0) * 255.0;
                draw_circle(
                    center.x,
                    center.y,
                    size + 3.0,
                    Color::from_rgba(255, 255, 100, flash_alpha as u8),
                );
            }

            // Disease indicator
            if agent.disease.infected {
                draw_circle(
                    center.x,
                    center.y,
                    size * 0.5,
                    Color::from_rgba(150, 50, 200, 200),
                );
            }
        }
    }
}
