use crate::world;

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
}
