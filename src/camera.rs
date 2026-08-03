use macroquad::prelude::*;

pub struct SimCamera {
    pub target_x: f32,
    pub target_y: f32,
    pan_x: f32,
    pan_y: f32,
    pub zoom: f32,
    pub zoom_target: f32,
    pub min_zoom: f32,
    pub rotation: f32,
}

impl SimCamera {
    pub fn new() -> Self {
        SimCamera {
            target_x: 0.0,
            target_y: 0.0,
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
            zoom_target: 1.0,
            min_zoom: 0.1,
            rotation: 0.0,
        }
    }

    pub fn center_on(&mut self, world_x: f32, world_y: f32) {
        self.target_x = world_x;
        self.target_y = world_y;
        self.pan_x = world_x;
        self.pan_y = world_y;
    }

    pub fn handle_input(&mut self) {
        let dt = get_frame_time();
        let base_pan = 400.0;

        let boost = if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
            3.0
        } else {
            1.0
        };
        let pan_speed = base_pan * boost / self.zoom;

        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            self.pan_y -= pan_speed * dt;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            self.pan_y += pan_speed * dt;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            self.pan_x -= pan_speed * dt;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            self.pan_x += pan_speed * dt;
        }

        let scroll = mouse_wheel();
        if scroll.1 != 0.0 {
            self.zoom_target *= 1.0 + scroll.1 * 0.1;
            self.zoom_target = self.zoom_target.clamp(self.min_zoom, 20.0);
        }

        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            self.zoom_target = (self.zoom_target / 1.2).max(self.min_zoom);
        }
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            self.zoom_target = (self.zoom_target * 1.2).min(20.0);
        }

        self.zoom += (self.zoom_target - self.zoom) * 0.3;

        if is_mouse_button_down(MouseButton::Middle) {
            let delta = mouse_delta_position();
            self.pan_x -= delta.x / self.zoom;
            self.pan_y -= delta.y / self.zoom;
        }

        // Smoothly interpolate actual camera position toward pan target
        let lerp_factor = 0.4;
        self.target_x += (self.pan_x - self.target_x) * lerp_factor;
        self.target_y += (self.pan_y - self.target_y) * lerp_factor;
    }

    pub fn as_camera2d(&self) -> Camera2D {
        let sw = screen_width();
        let sh = screen_height();
        // Convert pixels-per-unit zoom to normalized device space (-1..1)
        let zoom_x = 2.0 * self.zoom / sw;
        let zoom_y = 2.0 * self.zoom / sh;
        Camera2D {
            target: vec2(self.target_x, self.target_y),
            offset: vec2(0.0, 0.0),
            rotation: self.rotation,
            zoom: vec2(zoom_x, zoom_y),
            render_target: None,
            viewport: None,
        }
    }
}
