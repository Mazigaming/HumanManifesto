mod camera;
mod render;
mod sim;
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
                set_default_camera();
                ui.draw(sim);
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
}
