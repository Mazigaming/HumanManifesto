mod agent;
mod camera;
mod civilization;
mod evolution;
mod lineage;
mod render;
mod sim;
mod tribe;
mod belief;
mod diplomacy;
mod ui;
mod world;

use camera::SimCamera;
use macroquad::prelude::*;
use render::Renderer;
use sim::Sim;
use ui::UI;

const HEX_SIZE: f32 = 16.0;

#[macroquad::main("Humanity Evolution Sim")]
async fn main() {
    let mut sim: Option<Sim> = None;
    let mut sim_camera = SimCamera::new();
    let renderer = Renderer::new();
    let mut ui = UI::new();
    let mut generating = true;

    let fixed_dt = 1.0 / 60.0;

    loop {
        if generating {
            clear_background(BLACK);
            let text = format!("Generating world ({}x{})...", ui.map_width, ui.map_height);
            let text_size = measure_text(&text, None, 24, 1.0);
            draw_text(
                &text,
                screen_width() / 2.0 - text_size.width / 2.0,
                screen_height() / 2.0,
                24.0,
                WHITE,
            );
            next_frame().await;

            let s = Sim::new(ui.map_width, ui.map_height);
            center_camera_on_world(&mut sim_camera, &s);
            sim = Some(s);
            generating = false;
            continue;
        }

        if let Some(ref mut sim) = sim {
            sim_camera.handle_input();
            handle_input(sim, &mut ui, &mut sim_camera);

            let dt = get_frame_time() as f64;
            let effective_dt = if sim.paused {
                0.0
            } else {
                dt * sim.speed_multiplier()
            };

            sim.accumulator += effective_dt;
            while sim.accumulator >= fixed_dt {
                sim.tick(fixed_dt);
                sim.accumulator -= fixed_dt;
            }

            if sim.is_loading {
                clear_background(BLACK);
                let text = &sim.loading_message;
                let text_size = measure_text(text, None, 24, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_size.width / 2.0,
                    screen_height() / 2.0,
                    24.0,
                    WHITE,
                );
            } else {
                // Ocean blue background so zooming out shows water, not black
                clear_background(Color::from_rgba(30, 70, 160, 255));
                set_camera(&sim_camera.as_camera2d());
                renderer.draw_world(&sim.world, sim_camera.target_x, sim_camera.target_y, sim_camera.zoom);
                renderer.draw_agents(&sim.evo.agents, &sim.world, sim_camera.target_x, sim_camera.target_y, sim_camera.zoom);
                set_default_camera();
                ui.draw(sim);

                // Hover tooltip for resources/features
                let mouse_pos = mouse_position();
                let world_pos = world::screen_to_world(
                    mouse_pos.0,
                    mouse_pos.1,
                    sim_camera.target_x,
                    sim_camera.target_y,
                    sim_camera.zoom,
                );

                if let Some((col, row)) = world::find_tile_at(
                    world_pos.x,
                    world_pos.y,
                    sim.world.width,
                    sim.world.height,
                    HEX_SIZE,
                ) {
                    if let Some(tile) = sim.world.get_tile(col, row) {
                        if let Some(tooltip) = world::get_tile_tooltip(tile) {
                            // Draw tooltip background
                            let tooltip_size = measure_text(&tooltip, None, 20, 1.0);
                            let tooltip_x = mouse_pos.0 + 10.0;
                            let tooltip_y = mouse_pos.1 - 10.0;

                            draw_rectangle(
                                tooltip_x - 5.0,
                                tooltip_y - 20.0,
                                tooltip_size.width + 10.0,
                                25.0,
                                Color::from_rgba(0, 0, 0, 200),
                            );

                            draw_text(
                                &tooltip,
                                tooltip_x,
                                tooltip_y,
                                20.0,
                                WHITE,
                            );
                        }
                    }
                }
            }
        }

        next_frame().await;
    }
}

fn center_camera_on_world(camera: &mut SimCamera, sim: &Sim) {
    let cx = sim.world.width as f32 * 3.0_f32.sqrt() * HEX_SIZE / 2.0;
    let cy = sim.world.height as f32 * 1.5 * HEX_SIZE / 2.0;
    camera.center_on(cx, cy);
    // Calculate zoom to fit whole map on screen with some padding
    let map_width = sim.world.width as f32 * 3.0_f32.sqrt() * HEX_SIZE;
    let map_height = sim.world.height as f32 * 1.5 * HEX_SIZE;
    let screen_width = 1280.0;
    let screen_height = 720.0;
    let zoom_x = (screen_width / map_width) * 0.85;
    let zoom_y = (screen_height / map_height) * 0.85;
    let fit_zoom = zoom_x.min(zoom_y).min(1.0);
    camera.zoom = fit_zoom;
    camera.zoom_target = fit_zoom;
    // Store minimum zoom so user can't zoom out past the map
    camera.min_zoom = fit_zoom;
}

fn handle_input(sim: &mut Sim, ui: &mut UI, camera: &mut SimCamera) {
    if sim.is_loading {
        return;
    }
    if is_key_pressed(KeyCode::Space) {
        sim.paused = !sim.paused;
    }
    if is_key_pressed(KeyCode::Key0) {
        sim.speed_idx = 0;
    }
    if is_key_pressed(KeyCode::Key1) {
        sim.speed_idx = 1;
    }
    if is_key_pressed(KeyCode::Key2) {
        sim.speed_idx = 2;
    }
    if is_key_pressed(KeyCode::R) {
        let new_seed = if ui.use_same_seed {
            sim.world.seed
        } else {
            sim.random_seed()
        };
        sim.regenerate(ui.map_width, ui.map_height, new_seed);
        center_camera_on_world(camera, sim);
    }
    if is_key_pressed(KeyCode::LeftBracket) {
        ui.map_width = (ui.map_width - 10).max(20);
    }
    if is_key_pressed(KeyCode::RightBracket) {
        ui.map_width = (ui.map_width + 10).min(1000);
    }
    if is_key_pressed(KeyCode::Semicolon) {
        ui.map_height = (ui.map_height - 10).max(20);
    }
    if is_key_pressed(KeyCode::Apostrophe) {
        ui.map_height = (ui.map_height + 10).min(1000);
    }
    if is_key_pressed(KeyCode::Tab) {
        ui.use_same_seed = !ui.use_same_seed;
    }
    if is_key_pressed(KeyCode::C) {
        center_camera_on_world(camera, sim);
    }
    if is_key_pressed(KeyCode::G) {
        ui.show_gene_pool = !ui.show_gene_pool;
    }
    if is_key_pressed(KeyCode::T) {
        ui.show_tribes = !ui.show_tribes;
    }
    if is_key_pressed(KeyCode::V) {
        ui.show_civilization = !ui.show_civilization;
    }

    // Phase 6 — Divine guidance: number keys 1-9 select available focuses
    if ui.show_civilization {
        let all_focuses = crate::civilization::build_focus_tree();
        for i in 1..=9 {
            if is_key_pressed(match i {
                1 => KeyCode::Key1,
                2 => KeyCode::Key2,
                3 => KeyCode::Key3,
                4 => KeyCode::Key4,
                5 => KeyCode::Key5,
                6 => KeyCode::Key6,
                7 => KeyCode::Key7,
                8 => KeyCode::Key8,
                _ => KeyCode::Key9,
            }) {
                if let Some(civ) = sim.evo.civilizations.first_mut() {
                    let available: Vec<u32> = civ.available_focuses(&all_focuses)
                        .iter()
                        .map(|f| f.id)
                        .collect();
                    if let Some(&focus_id) = available.get(i - 1) {
                        if let Some(focus) = all_focuses.iter().find(|f| f.id == focus_id) {
                            if civ.spend_influence(focus.cost) {
                                civ.unlock_focus(focus.id);
                                civ.activate_focus(focus.id);
                                let logs = civ.apply_effects(&focus.effects);
                                for log in logs {
                                    sim.evo.chronicle.push(format!("[{}] {}", civ.identity.name, log));
                                }
                                sim.evo.chronicle.push(format!(
                                    "[{}] Purchased focus: {}",
                                    civ.identity.name, focus.name
                                ));
                            }
                        }
                    }
                }
                break;
            }
        }
    }
}
