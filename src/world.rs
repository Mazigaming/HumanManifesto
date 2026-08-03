use macroquad::prelude::*;
use noise::{NoiseFn, Perlin};

// Simple hash-based noise - reliable and debuggable
fn hash2d(x: i32, y: i32, seed: u64) -> f32 {
    let mut h = seed;
    h ^= (x as u64).wrapping_mul(374761393);
    h ^= (y as u64).wrapping_mul(668265263);
    h = h.wrapping_mul(1274126177);
    h ^= h >> 13;
    h = h.wrapping_mul(1274126177);
    h ^= h >> 16;
    (h as u32) as f32 / (u32::MAX as f32)
}

fn smooth_noise(x: f32, y: f32, seed: u64) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;
    let fx = x - x0 as f32;
    let fy = y - y0 as f32;
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);

    let n00 = hash2d(x0, y0, seed);
    let n10 = hash2d(x1, y0, seed);
    let n01 = hash2d(x0, y1, seed);
    let n11 = hash2d(x1, y1, seed);

    let nx0 = n00 + (n10 - n00) * sx;
    let nx1 = n01 + (n11 - n01) * sx;
    nx0 + (nx1 - nx0) * sy
}

fn fbm(x: f32, y: f32, seed: u64, octaves: usize, frequency: f32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;
    let mut freq = frequency;

    for i in 0..octaves {
        value += smooth_noise(x * freq, y * freq, seed + i as u64) * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        freq *= 2.0;
    }

    value / max_value
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Biome {
    Ocean,
    Beach,
    Tundra,
    Taiga,
    Plains,
    TemperateForest,
    Desert,
    Savanna,
    Jungle,
    Swamp,
    Hills,
    HighlandForest,
    Mountain,
    SnowMountain,
}

impl Biome {
    pub fn color(&self) -> Color {
        match self {
            Biome::Ocean => Color::from_rgba(30, 70, 160, 255),
            Biome::Beach => Color::from_rgba(220, 200, 140, 255),
            Biome::Tundra => Color::from_rgba(160, 180, 190, 255),
            Biome::Taiga => Color::from_rgba(60, 110, 80, 255),
            Biome::Plains => Color::from_rgba(140, 190, 70, 255),
            Biome::TemperateForest => Color::from_rgba(50, 140, 50, 255),
            Biome::Desert => Color::from_rgba(210, 180, 100, 255),
            Biome::Savanna => Color::from_rgba(180, 170, 60, 255),
            Biome::Jungle => Color::from_rgba(30, 120, 40, 255),
            Biome::Swamp => Color::from_rgba(80, 110, 70, 255),
            Biome::Hills => Color::from_rgba(150, 140, 100, 255),
            Biome::HighlandForest => Color::from_rgba(60, 100, 50, 255),
            Biome::Mountain => Color::from_rgba(140, 130, 120, 255),
            Biome::SnowMountain => Color::from_rgba(200, 210, 220, 255),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceType {
    Berries,
    Game,
    Fish,
    Timber,
    WildGrain,
    Stone,
    Ore,
}

impl ResourceType {
    pub fn color(&self) -> Color {
        match self {
            ResourceType::Berries => Color::from_rgba(220, 50, 50, 255),
            ResourceType::Game => Color::from_rgba(180, 100, 50, 255),
            ResourceType::Fish => Color::from_rgba(50, 100, 220, 255),
            ResourceType::Timber => Color::from_rgba(100, 70, 40, 255),
            ResourceType::WildGrain => Color::from_rgba(220, 200, 50, 255),
            ResourceType::Stone => Color::from_rgba(160, 150, 140, 255),
            ResourceType::Ore => Color::from_rgba(200, 180, 60, 255),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ResourceNode {
    pub resource_type: ResourceType,
    pub richness: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundaryType {
    Convergent, // Mountains
    Divergent,  // Rift valleys
    Transform,  // Minor disturbance
}

#[derive(Clone, Copy, Debug)]
pub struct Plate {
    pub center_col: f32,
    pub center_row: f32,
    pub drift_x: f32,
    pub drift_y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuinType {
    Structure,
    Burial,
    Monument,
}

#[derive(Clone, Copy, Debug)]
pub struct Ruin {
    pub ruin_type: RuinType,
}

#[derive(Clone, Debug)]
pub struct LegendaryResource {
    pub name: String,
    pub resource_type: ResourceType,
    pub richness: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecialBiomeVariant {
    None,
    CrystalCavern, // Mountain variant
    Oasis,         // Desert variant
    Aurora,        // Tundra variant
}

#[derive(Clone, Copy, Debug)]
pub struct OriginPoint {
    pub col: i32,
    pub row: i32,
    pub color_seed: u64,
}

#[derive(Clone, Debug)]
pub struct HexTile {
    pub col: i32,
    pub row: i32,
    pub elevation: f32,
    pub moisture: f32,
    pub temperature: f32,
    pub biome: Biome,
    pub is_river: bool,
    pub resource: Option<ResourceNode>,
    // Phase 1.5 fields
    pub plate_id: Option<usize>,
    pub boundary_type: Option<BoundaryType>,
    pub origin_point: Option<usize>,
    pub ruin: Option<Ruin>,
    pub legendary_resource: Option<LegendaryResource>,
    pub special_variant: SpecialBiomeVariant,
    pub name: Option<String>,
}

pub struct World {
    pub width: i32,
    pub height: i32,
    pub seed: u64,
    pub tiles: Vec<HexTile>,
}

const OCEAN_CUTOFF: f32 = 0.30;
const BEACH_CUTOFF: f32 = 0.40;

const RIVER_SEED_OFFSET: u64 = 4_000;
const RESOURCE_SEED_OFFSET: u64 = 5_000;
const RESOURCE_RICHNESS_SEED_OFFSET: u64 = 6_000;

const RIVER_SCALE: f64 = 0.25;
const RESOURCE_SCALE: f64 = 0.45;
const RESOURCE_RICHNESS_SCALE: f64 = 0.4;

fn normalized_noise(value: f64) -> f32 {
    ((value + 1.0) * 0.5).clamp(0.0, 1.0) as f32
}

fn layer_seed(base_seed: u64, offset: u64) -> u32 {
    (base_seed.wrapping_add(offset) & 0xFFFF_FFFF) as u32
}

fn perlin_roll(perlin: &Perlin, x: f64, y: f64) -> f32 {
    normalized_noise(perlin.get([x, y]))
}

fn tile_index(col: i32, row: i32, width: i32) -> Option<usize> {
    if col < 0 || col >= width || row < 0 {
        return None;
    }
    // Note: row >= height check is done by caller via get_tile bounds
    let idx = row as usize * width as usize + col as usize;
    Some(idx)
}

fn hex_neighbors(col: i32, row: i32) -> [(i32, i32); 6] {
    let parity = row & 1;
    if parity == 0 {
        [
            (col - 1, row - 1),
            (col, row - 1),
            (col - 1, row),
            (col + 1, row),
            (col - 1, row + 1),
            (col, row + 1),
        ]
    } else {
        [
            (col, row - 1),
            (col + 1, row - 1),
            (col - 1, row),
            (col + 1, row),
            (col, row + 1),
            (col + 1, row + 1),
        ]
    }
}

pub fn hex_center(col: i32, row: i32, size: f32) -> Vec2 {
    let x = size * (3.0_f32.sqrt() * col as f32 + 3.0_f32.sqrt() / 2.0 * (row & 1) as f32);
    let y = size * (1.5 * row as f32);
    vec2(x, y)
}

#[allow(dead_code)]
fn hex_corner(center: Vec2, size: f32, i: u32) -> Vec2 {
    let angle_deg = 60.0 * i as f32 - 30.0;
    let angle_rad = angle_deg * std::f32::consts::PI / 180.0;
    vec2(
        center.x + size * angle_rad.cos(),
        center.y + size * angle_rad.sin(),
    )
}

fn biome_for(elevation: f32, temperature: f32, moisture: f32, is_coastal: bool) -> Biome {
    if elevation < OCEAN_CUTOFF {
        return Biome::Ocean;
    }
    if is_coastal && elevation < BEACH_CUTOFF {
        return Biome::Beach;
    }

    let temp_cat = if temperature < 0.3 {
        "cold"
    } else if temperature < 0.6 {
        "temperate"
    } else {
        "hot"
    };
    let moist_cat = if moisture < 0.25 {
        "dry"
    } else if moisture < 0.55 {
        "moderate"
    } else {
        "wet"
    };

    if elevation < 0.55 {
        if moisture > 0.75 && elevation < BEACH_CUTOFF && temp_cat != "cold" {
            return Biome::Swamp;
        }
        match (temp_cat, moist_cat) {
            ("cold", _) => Biome::Tundra,
            ("temperate", "dry") => Biome::Plains,
            ("temperate", "moderate") => Biome::Plains,
            ("temperate", "wet") => Biome::TemperateForest,
            ("hot", "dry") => Biome::Desert,
            ("hot", "moderate") => Biome::Savanna,
            ("hot", "wet") => Biome::Jungle,
            _ => Biome::Plains,
        }
    } else if elevation < 0.75 {
        match (temp_cat, moist_cat) {
            ("cold", _) => Biome::Taiga,
            ("temperate", "dry") => Biome::Plains,
            ("temperate", "moderate") => Biome::TemperateForest,
            ("temperate", "wet") => Biome::TemperateForest,
            ("hot", "dry") => Biome::Savanna,
            ("hot", "moderate") => Biome::Savanna,
            ("hot", "wet") => Biome::Jungle,
            _ => Biome::TemperateForest,
        }
    } else if elevation < 0.85 {
        if moist_cat == "dry" || moist_cat == "moderate" {
            Biome::Hills
        } else {
            Biome::HighlandForest
        }
    } else {
        if temperature < 0.35 || elevation > 0.95 {
            Biome::SnowMountain
        } else {
            Biome::Mountain
        }
    }
}

fn resource_table(biome: Biome) -> Vec<(ResourceType, f32, f32)> {
    match biome {
        Biome::Ocean => vec![],
        Biome::Beach => vec![(ResourceType::Fish, 0.15, 0.3)],
        Biome::Tundra => vec![(ResourceType::Berries, 0.08, 0.2)],
        Biome::Taiga => vec![
            (ResourceType::Berries, 0.12, 0.3),
            (ResourceType::Game, 0.10, 0.3),
            (ResourceType::Timber, 0.15, 0.4),
        ],
        Biome::Plains => vec![
            (ResourceType::Game, 0.12, 0.4),
            (ResourceType::WildGrain, 0.08, 0.3),
        ],
        Biome::TemperateForest => vec![
            (ResourceType::Berries, 0.15, 0.4),
            (ResourceType::Game, 0.12, 0.4),
            (ResourceType::Timber, 0.18, 0.5),
        ],
        Biome::Desert => vec![(ResourceType::Stone, 0.06, 0.3)],
        Biome::Savanna => vec![
            (ResourceType::Game, 0.15, 0.5),
            (ResourceType::WildGrain, 0.06, 0.3),
        ],
        Biome::Jungle => vec![
            (ResourceType::Berries, 0.20, 0.5),
            (ResourceType::Game, 0.10, 0.3),
            (ResourceType::Timber, 0.15, 0.5),
        ],
        Biome::Swamp => vec![(ResourceType::Berries, 0.10, 0.2)],
        Biome::Hills => vec![
            (ResourceType::Stone, 0.15, 0.4),
            (ResourceType::Game, 0.06, 0.2),
        ],
        Biome::HighlandForest => vec![
            (ResourceType::Timber, 0.12, 0.4),
            (ResourceType::Game, 0.08, 0.3),
        ],
        Biome::Mountain => vec![
            (ResourceType::Stone, 0.20, 0.5),
            (ResourceType::Ore, 0.08, 0.3),
        ],
        Biome::SnowMountain => vec![
            (ResourceType::Stone, 0.12, 0.3),
            (ResourceType::Ore, 0.06, 0.2),
        ],
    }
}

impl World {
    pub fn generate(width: i32, height: i32, seed: u64) -> Self {
        let _hex_size = 16.0;
        let tile_count = (width * height) as usize;

        // Generate plates (1 per ~150x150 area, minimum 2)
        let plate_count = ((width * height) / (150 * 150)).max(2) as usize;
        let plates = Self::generate_plates(plate_count, width, height, seed);

        // Initialize tiles with plate assignment
        let mut tiles: Vec<HexTile> = Vec::with_capacity(tile_count);
        for row in 0..height {
            for col in 0..width {
                let plate_id = Self::find_nearest_plate(col, row, &plates);
                tiles.push(HexTile {
                    col,
                    row,
                    elevation: 0.0,
                    moisture: 0.5,
                    temperature: 0.5,
                    biome: Biome::Ocean,
                    is_river: false,
                    resource: None,
                    plate_id: Some(plate_id),
                    boundary_type: None,
                    origin_point: None,
                    ruin: None,
                    legendary_resource: None,
                    special_variant: SpecialBiomeVariant::None,
                    name: None,
                });
            }
        }

        let mut world = World {
            width,
            height,
            seed,
            tiles,
        };

        // Detect plate boundaries and apply elevation effects
        world.detect_boundaries(&plates);

        // Apply plate-based elevation
        world.apply_plate_elevation(&plates);

        // TODO: Add domain warping, multi-scale noise, wind moisture, erosion...
        // For now, use simple noise overlay
        world.apply_noise_overlay();

        world.assign_biomes();
        world.trace_rivers();
        world.place_resources();

        // Generate discovery layer
        world.generate_origin_points(3);
        world.scatter_ruins(15);
        world.place_legendary_resources(2);
        world.add_special_biome_variants(8);

        println!("[DEBUG] Phase 1.5 generation complete");
        println!("[DEBUG] Plates: {}, Boundaries detected", plates.len());

        world
    }

    fn generate_plates(count: usize, width: i32, height: i32, _seed: u64) -> Vec<Plate> {
        let mut plates = Vec::with_capacity(count);
        for i in 0..count {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
            let drift_x = angle.cos() * 0.5;
            let drift_y = angle.sin() * 0.5;

            // Spread plate centers across the map
            let center_col = (width as f32 / 2.0) + (width as f32 * 0.3 * angle.cos());
            let center_row = (height as f32 / 2.0) + (height as f32 * 0.3 * angle.sin());

            plates.push(Plate {
                center_col,
                center_row,
                drift_x,
                drift_y,
            });
        }
        plates
    }

    fn find_nearest_plate(col: i32, row: i32, plates: &[Plate]) -> usize {
        let mut nearest = 0;
        let mut min_dist = f32::MAX;

        for (i, plate) in plates.iter().enumerate() {
            let dc = col as f32 - plate.center_col;
            let dr = row as f32 - plate.center_row;
            let dist = dc * dc + dr * dr;
            if dist < min_dist {
                min_dist = dist;
                nearest = i;
            }
        }
        nearest
    }

    fn detect_boundaries(&mut self, plates: &[Plate]) {
        // First pass: mark tiles that are near plate boundaries
        let mut boundary_tiles = Vec::new();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = (row * self.width + col) as usize;
                let my_plate = self.tiles[idx].plate_id.unwrap();

                // Check all neighbors for different plates
                for (nc, nr) in hex_neighbors(col, row) {
                    if let Some(nidx) = tile_index(nc, nr, self.width) {
                        if nidx < self.tiles.len() {
                            let neighbor_plate = self.tiles[nidx].plate_id.unwrap();
                            if neighbor_plate != my_plate {
                                boundary_tiles.push((col, row));
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Second pass: smooth boundary effects over 2-tile radius
        for &(col, row) in &boundary_tiles {
            for dr in -2..=2 {
                for dc in -2..=2 {
                    let nc = col + dc;
                    let nr = row + dr;
                    if let Some(idx) = tile_index(nc, nr, self.width) {
                        if idx < self.tiles.len() && self.tiles[idx].boundary_type.is_none() {
                            // Calculate distance from boundary
                            let dist = (dc * dc + dr * dr) as f32;
                            let falloff = (1.0 - dist / 8.0).max(0.0);

                            // Determine boundary type based on nearest boundary tile
                            let my_plate = self.tiles[idx].plate_id.unwrap();
                            for (nc2, nr2) in hex_neighbors(nc, nr) {
                                if let Some(nidx2) = tile_index(nc2, nr2, self.width) {
                                    if nidx2 < self.tiles.len() {
                                        let neighbor_plate = self.tiles[nidx2].plate_id.unwrap();
                                        if neighbor_plate != my_plate {
                                            let my_drift = (
                                                plates[my_plate].drift_x,
                                                plates[my_plate].drift_y,
                                            );
                                            let their_drift = (
                                                plates[neighbor_plate].drift_x,
                                                plates[neighbor_plate].drift_y,
                                            );

                                            let dx = plates[neighbor_plate].center_col
                                                - plates[my_plate].center_col;
                                            let dy = plates[neighbor_plate].center_row
                                                - plates[my_plate].center_row;
                                            let dist = (dx * dx + dy * dy).sqrt();
                                            if dist > 0.0 {
                                                let nx = dx / dist;
                                                let ny = dy / dist;

                                                let rel_vx = their_drift.0 - my_drift.0;
                                                let rel_vy = their_drift.1 - my_drift.1;
                                                let convergence = rel_vx * nx + rel_vy * ny;

                                                let boundary = if convergence > 0.1 {
                                                    BoundaryType::Convergent
                                                } else if convergence < -0.1 {
                                                    BoundaryType::Divergent
                                                } else {
                                                    BoundaryType::Transform
                                                };

                                                // Only set if not already set, or if this is closer
                                                self.tiles[idx].boundary_type = Some(boundary);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn apply_plate_elevation(&mut self, plates: &[Plate]) {
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = (row * self.width + col) as usize;
                let plate_id = self.tiles[idx].plate_id.unwrap();
                let plate = &plates[plate_id];

                // Distance from plate center (normalized)
                let dc = col as f32 - plate.center_col;
                let dr = row as f32 - plate.center_row;
                let dist = (dc * dc + dr * dr).sqrt();
                let max_dist = (self.width.max(self.height) as f32) * 0.5;
                let normalized_dist = (dist / max_dist).min(1.0);

                // Base elevation: high at center, low at edges
                let base_elev = (1.0 - normalized_dist).powi(2);

                // Boundary effects (reduced to avoid visible seams)
                let boundary_boost = match self.tiles[idx].boundary_type {
                    Some(BoundaryType::Convergent) => 0.15, // Mountains (reduced from 0.3)
                    Some(BoundaryType::Divergent) => -0.1,  // Rift valleys (reduced from -0.2)
                    Some(BoundaryType::Transform) => 0.02,  // Minor (reduced from 0.05)
                    None => 0.0,
                };

                self.tiles[idx].elevation = (base_elev + boundary_boost).clamp(0.0, 1.0);
            }
        }
    }

    fn apply_noise_overlay(&mut self) {
        // Domain warping: distort coordinates using secondary noise field
        let warp_strength = 6.0; // Reduced from 8.0 for performance

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = (row * self.width + col) as usize;

                // Generate warp offsets from low-frequency noise (reduced octaves for performance)
                let warp_x = fbm(
                    col as f32 * 0.02,
                    row as f32 * 0.02,
                    self.seed + 1000,
                    1,
                    1.0,
                ) * warp_strength;
                let warp_y = fbm(
                    col as f32 * 0.02,
                    row as f32 * 0.02,
                    self.seed + 2000,
                    1,
                    1.0,
                ) * warp_strength;

                // Sample elevation noise at warped coordinates
                let warped_col = col as f32 + warp_x;
                let warped_row = row as f32 + warp_y;

                // Multi-scale noise blending (reduced octaves for performance)
                let continent_scale =
                    fbm(warped_col * 0.01, warped_row * 0.01, self.seed, 1, 1.0) * 0.4;
                let regional_scale = fbm(
                    warped_col * 0.05,
                    warped_row * 0.05,
                    self.seed + 100,
                    2,
                    1.0,
                ) * 0.3;
                let local_scale =
                    fbm(warped_col * 0.2, warped_row * 0.2, self.seed + 200, 2, 1.0) * 0.1;

                let noise = continent_scale + regional_scale + local_scale;
                self.tiles[idx].elevation = (self.tiles[idx].elevation + noise).clamp(0.0, 1.0);

                // Apply same warp to moisture for natural biome boundaries
                let moisture_warp = fbm(
                    warped_col * 0.03,
                    warped_row * 0.03,
                    self.seed + 300,
                    2,
                    1.0,
                ) * 0.2;
                self.tiles[idx].moisture =
                    (self.tiles[idx].moisture + moisture_warp).clamp(0.0, 1.0);
            }
        }
    }

    fn generate_origin_points(&mut self, count: usize) {
        let mut rng = (self.seed as u64, 0u64);
        let mut attempts = 0;
        let mut placed = 0;

        while placed < count && attempts < 1000 {
            let col = (hash2d(placed as i32, 0, self.seed) * self.width as f32) as i32;
            let row = (hash2d(0, placed as i32, self.seed) * self.height as f32) as i32;

            if let Some(idx) = tile_index(col, row, self.width) {
                if idx < self.tiles.len() {
                    let tile = &self.tiles[idx];
                    // Only place in hospitable biomes
                    if tile.elevation > 0.3
                        && tile.biome != Biome::Desert
                        && tile.biome != Biome::Tundra
                    {
                        // Check minimum distance from other origins
                        let too_close = (0..placed).any(|i| {
                            if let Some(existing_idx) =
                                self.tiles.iter().position(|t| t.origin_point == Some(i))
                            {
                                let ec = self.tiles[existing_idx].col;
                                let er = self.tiles[existing_idx].row;
                                let dist = ((col - ec).pow(2) + (row - er).pow(2)) as f32;
                                let dist = dist.sqrt();
                                dist < 30.0
                            } else {
                                false
                            }
                        });

                        if !too_close {
                            self.tiles[idx].origin_point = Some(placed);
                            placed += 1;
                        }
                    }
                }
            }
            attempts += 1;
        }
        println!("[DEBUG] Origin points placed: {}", placed);
    }

    fn scatter_ruins(&mut self, count: usize) {
        for i in 0..count {
            let col = (hash2d(i as i32 + 1000, 0, self.seed) * self.width as f32) as i32;
            let row = (hash2d(0, i as i32 + 1000, self.seed) * self.height as f32) as i32;

            if let Some(idx) = tile_index(col, row, self.width) {
                if idx < self.tiles.len() {
                    let tile = &self.tiles[idx];
                    // Only on land, not too extreme
                    if tile.elevation > 0.3 && tile.elevation < 0.8 {
                        let ruin_type = match i % 3 {
                            0 => RuinType::Structure,
                            1 => RuinType::Burial,
                            _ => RuinType::Monument,
                        };
                        self.tiles[idx].ruin = Some(Ruin { ruin_type });
                    }
                }
            }
        }
        println!("[DEBUG] Ruins scattered: {}", count);
    }

    fn place_legendary_resources(&mut self, count: usize) {
        let names = vec![
            "Dragon's Hoard",
            "Elven Forge",
            "Dwarven Deep",
            "Ancient Vault",
            "Titan's Cache",
            "Godsblood Mine",
            "Starfall Deposit",
            "Void Crystal",
        ];

        for i in 0..count {
            let col = (hash2d(i as i32 + 2000, 0, self.seed) * self.width as f32) as i32;
            let row = (hash2d(0, i as i32 + 2000, self.seed) * self.height as f32) as i32;

            if let Some(idx) = tile_index(col, row, self.width) {
                if idx < self.tiles.len() {
                    let tile = &self.tiles[idx];
                    // Place in contestable areas (mid-elevation)
                    if tile.elevation > 0.4 && tile.elevation < 0.7 {
                        let name = names[i % names.len()].to_string();
                        let resource_type = if i % 2 == 0 {
                            ResourceType::Ore
                        } else {
                            ResourceType::Stone
                        };
                        self.tiles[idx].legendary_resource = Some(LegendaryResource {
                            name,
                            resource_type,
                            richness: 1.0,
                        });
                    }
                }
            }
        }
        println!("[DEBUG] Legendary resources placed: {}", count);
    }

    fn add_special_biome_variants(&mut self, count: usize) {
        for i in 0..count {
            let col = (hash2d(i as i32 + 3000, 0, self.seed) * self.width as f32) as i32;
            let row = (hash2d(0, i as i32 + 3000, self.seed) * self.height as f32) as i32;

            if let Some(idx) = tile_index(col, row, self.width) {
                if idx < self.tiles.len() {
                    let tile = &self.tiles[idx];
                    let variant = match tile.biome {
                        Biome::Mountain | Biome::SnowMountain => SpecialBiomeVariant::CrystalCavern,
                        Biome::Desert => SpecialBiomeVariant::Oasis,
                        Biome::Tundra => SpecialBiomeVariant::Aurora,
                        _ => continue, // Skip if not a suitable biome
                    };
                    self.tiles[idx].special_variant = variant;
                }
            }
        }
        println!("[DEBUG] Special biome variants added: {}", count);
    }

    pub fn get_tile(&self, col: i32, row: i32) -> Option<&HexTile> {
        let idx = tile_index(col, row, self.width)?;
        self.tiles.get(idx)
    }

    #[allow(dead_code)]
    fn get_tile_mut(&mut self, col: i32, row: i32) -> Option<&mut HexTile> {
        let idx = tile_index(col, row, self.width)?;
        self.tiles.get_mut(idx)
    }

    fn pit_fill(&mut self, passes: usize) {
        for _ in 0..passes {
            for row in 0..self.height {
                for col in 0..self.width {
                    let idx = tile_index(col, row, self.width).unwrap();
                    let elev = self.tiles[idx].elevation;
                    if elev < OCEAN_CUTOFF {
                        continue;
                    }
                    let mut lowest = elev;
                    for (nc, nr) in hex_neighbors(col, row) {
                        if let Some(t) = self.get_tile(nc, nr) {
                            if t.elevation < lowest {
                                lowest = t.elevation;
                            }
                        }
                    }
                    if lowest < elev {
                        self.tiles[idx].elevation = lowest;
                    }
                }
            }
        }
    }

    fn apply_moisture_falloff(&mut self) {
        // Wind-driven moisture with rain shadow
        // Pick a global wind direction (e.g., west-to-east)
        let wind_dx = 1.0; // Wind blows in +x direction
        let wind_dy = 0.0;

        // Process tiles in wind order (left-to-right for west-to-east wind)
        let mut moisture_map = vec![0.5_f32; (self.width * self.height) as usize];

        // Initialize ocean tiles with high moisture
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = (row * self.width + col) as usize;
                if self.tiles[idx].elevation < OCEAN_CUTOFF {
                    moisture_map[idx] = 0.8; // Ocean = very moist
                }
            }
        }

        // Sweep moisture across map in wind direction
        // For west-to-east wind, process columns left to right
        for col in 0..self.width {
            for row in 0..self.height {
                let idx = (row * self.width + col) as usize;
                let tile = &self.tiles[idx];

                if tile.elevation < OCEAN_CUTOFF {
                    continue; // Ocean already initialized
                }

                // Get moisture from upwind neighbor
                let upwind_col = col - 1; // West neighbor for east-blowing wind
                let upwind_idx = if upwind_col >= 0 {
                    tile_index(upwind_col, row, self.width)
                } else {
                    None
                };

                let upwind_moisture = if let Some(ui) = upwind_idx {
                    moisture_map[ui]
                } else {
                    0.5 // Edge of map
                };

                // Moisture decreases when crossing elevation gains (rain shadow)
                let elevation_penalty = if upwind_idx.is_some() {
                    let upwind_elev = self.tiles[upwind_idx.unwrap()].elevation;
                    let elev_gain = tile.elevation - upwind_elev;
                    if elev_gain > 0.0 {
                        elev_gain * 0.5 // Mountains wring out moisture
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                // Calculate final moisture
                let moisture = (upwind_moisture - elevation_penalty).clamp(0.0, 1.0);
                moisture_map[idx] = moisture;
                self.tiles[idx].moisture = moisture;
            }
        }

        println!("[DEBUG] Wind-driven moisture applied (rain shadow enabled)");
    }

    fn assign_biomes(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = tile_index(col, row, self.width).unwrap();
                let tile = &self.tiles[idx];
                let is_coastal = if tile.elevation >= OCEAN_CUTOFF {
                    hex_neighbors(col, row).iter().any(|&(nc, nr)| {
                        self.get_tile(nc, nr)
                            .map_or(false, |t| t.elevation < OCEAN_CUTOFF)
                    })
                } else {
                    false
                };
                let biome = biome_for(tile.elevation, tile.temperature, tile.moisture, is_coastal);
                self.tiles[idx].biome = biome;
            }
        }
    }

    fn trace_rivers(&mut self) {
        // Hydraulic erosion: simulate water droplets flowing downhill
        // Performance: scale droplet count with map size, cap at reasonable limit
        let tile_count = (self.width * self.height) as usize;
        let droplet_count = (tile_count / 200).clamp(50, 500); // Reduced from /100
        let erosion_rate = 0.008; // Slightly reduced
        let deposit_rate = 0.004;
        let min_erosion_threshold = 0.03; // Slightly reduced

        let mut erosion_map = vec![0.0_f32; tile_count];

        // Drop droplets from random elevated points
        for i in 0..droplet_count {
            // Find a random elevated starting point (limit attempts for performance)
            let mut attempts = 0;
            let mut start_col = 0;
            let mut start_row = 0;

            while attempts < 50 {
                // Reduced from 100
                start_col =
                    (hash2d(i as i32, attempts, self.seed + 5000) * self.width as f32) as i32;
                start_row =
                    (hash2d(attempts, i as i32, self.seed + 6000) * self.height as f32) as i32;

                if let Some(idx) = tile_index(start_col, start_row, self.width) {
                    if idx < self.tiles.len() && self.tiles[idx].elevation > 0.45 {
                        // Reduced threshold
                        break;
                    }
                }
                attempts += 1;
            }

            if attempts >= 50 {
                continue;
            }

            // Simulate droplet flowing downhill (limit path length for performance)
            let mut col = start_col;
            let mut row = start_row;
            let mut water = 1.0;
            let mut speed = 1.0;
            let mut steps = 0;
            let max_steps = 100; // Limit path length

            while water > 0.01 && steps < max_steps {
                if let Some(idx) = tile_index(col, row, self.width) {
                    if idx >= self.tiles.len() {
                        break;
                    }

                    let current_elev = self.tiles[idx].elevation;

                    // Find steepest downhill neighbor
                    let mut lowest_elev = current_elev;
                    let mut lowest_neighbor = None;

                    for (nc, nr) in hex_neighbors(col, row) {
                        if let Some(nidx) = tile_index(nc, nr, self.width) {
                            if nidx < self.tiles.len() {
                                let neighbor_elev = self.tiles[nidx].elevation;
                                if neighbor_elev < lowest_elev {
                                    lowest_elev = neighbor_elev;
                                    lowest_neighbor = Some((nc, nr));
                                }
                            }
                        }
                    }

                    // Erode current tile
                    let erosion = erosion_rate * speed * water;
                    erosion_map[idx] += erosion;
                    self.tiles[idx].elevation = (current_elev - erosion).max(0.0);

                    // Move to lowest neighbor or stop
                    if let Some((nc, nr)) = lowest_neighbor {
                        col = nc;
                        row = nr;
                        speed = (speed + 0.1).min(2.0);
                        water *= 0.98;
                    } else {
                        // Deposit sediment at local minimum
                        let deposit = deposit_rate * water;
                        self.tiles[idx].elevation = (current_elev + deposit).min(1.0);
                        break;
                    }
                } else {
                    break;
                }
                steps += 1;
            }
        }

        // Mark heavily eroded tiles as rivers
        for idx in 0..self.tiles.len() {
            if erosion_map[idx] > min_erosion_threshold {
                self.tiles[idx].is_river = true;
            }
        }

        let river_count = self.tiles.iter().filter(|t| t.is_river).count();
        println!(
            "[DEBUG] Hydraulic erosion complete: {} river tiles ({} droplets)",
            river_count, droplet_count
        );
    }

    fn place_resources(&mut self) {
        let perlin = Perlin::new(layer_seed(self.seed, RESOURCE_SEED_OFFSET));
        let richness_perlin = Perlin::new(layer_seed(self.seed, RESOURCE_RICHNESS_SEED_OFFSET));
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = tile_index(col, row, self.width).unwrap();
                let biome = self.tiles[idx].biome;
                let table = resource_table(biome);
                if table.is_empty() {
                    continue;
                }
                let roll = perlin_roll(
                    &perlin,
                    col as f64 * RESOURCE_SCALE,
                    row as f64 * RESOURCE_SCALE,
                ) as f32;
                if roll < 0.2 {
                    continue;
                }
                let mut cumulative = 0.0_f32;
                for (res_type, weight, max_richness) in &table {
                    cumulative += weight;
                    if roll < cumulative {
                        self.tiles[idx].resource = Some(ResourceNode {
                            resource_type: *res_type,
                            richness: {
                                let richness_noise = perlin_roll(
                                    &richness_perlin,
                                    col as f64 * RESOURCE_RICHNESS_SCALE,
                                    row as f64 * RESOURCE_RICHNESS_SCALE,
                                );
                                (0.3 + richness_noise * 0.7 * max_richness).clamp(0.0, 1.0)
                            },
                        });
                        break;
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_tile_info(&self, col: i32, row: i32) -> Option<&HexTile> {
        self.get_tile(col, row)
    }
}

pub fn draw_world(world: &World, cam_target_x: f32, cam_target_y: f32, cam_zoom: f32) {
    let hex_size = 16.0;
    let overlap = 1.005;

    // Calculate view bounds for frustum culling
    let screen_w = screen_width();
    let screen_h = screen_height();
    let view_w = screen_w / cam_zoom;
    let view_h = screen_h / cam_zoom;
    let margin = hex_size * 3.0;

    // Draw ocean tiles
    for tile in &world.tiles {
        if tile.elevation >= OCEAN_CUTOFF {
            continue;
        }
        let center = hex_center(tile.col, tile.row, hex_size);
        // Frustum culling
        if (center.x - cam_target_x).abs() > view_w / 2.0 + margin {
            continue;
        }
        if (center.y - cam_target_y).abs() > view_h / 2.0 + margin {
            continue;
        }
        draw_poly(
            center.x,
            center.y,
            6,
            hex_size * overlap,
            -30.0,
            tile.biome.color(),
        );
    }

    // Draw land tiles
    for tile in &world.tiles {
        if tile.elevation < OCEAN_CUTOFF {
            continue;
        }
        let center = hex_center(tile.col, tile.row, hex_size);
        // Frustum culling
        if (center.x - cam_target_x).abs() > view_w / 2.0 + margin {
            continue;
        }
        if (center.y - cam_target_y).abs() > view_h / 2.0 + margin {
            continue;
        }

        let color = tile.biome.color();
        draw_poly(center.x, center.y, 6, hex_size * overlap, -30.0, color);

        // Draw resources
        if let Some(ref res) = tile.resource {
            let dot_radius = 1.5 + res.richness * 2.0;
            draw_circle(center.x, center.y, dot_radius, res.resource_type.color());
        }

        // Draw legendary resources (gold star)
        if let Some(ref leg) = tile.legendary_resource {
            let star_size = 4.0;
            draw_circle(
                center.x,
                center.y,
                star_size,
                Color::from_rgba(255, 215, 0, 255),
            );
        }

        // Draw ruins (gray square)
        if let Some(ref ruin) = tile.ruin {
            let ruin_size = 3.0;
            draw_rectangle(
                center.x - ruin_size,
                center.y - ruin_size,
                ruin_size * 2.0,
                ruin_size * 2.0,
                Color::from_rgba(128, 128, 128, 200),
            );
        }

        // Draw trees on forest/jungle/taiga tiles (only when zoomed in enough)
        if cam_zoom > 2.0 {
            let has_trees = matches!(
                tile.biome,
                Biome::TemperateForest | Biome::Jungle | Biome::Taiga | Biome::HighlandForest
            );
            if has_trees {
                // Draw fewer trees for performance (1-2 per tile instead of 2-3)
                let tree_count = if tile.biome == Biome::Jungle { 2 } else { 1 };
                for t in 0..tree_count {
                    let angle = (t as f32 / tree_count as f32) * std::f32::consts::TAU;
                    let offset_x = angle.cos() * 4.0;
                    let offset_y = angle.sin() * 4.0;
                    let tree_x = center.x + offset_x;
                    let tree_y = center.y + offset_y;
                    let tree_size = 2.5;
                    // Simple triangle tree
                    draw_triangle(
                        vec2(tree_x, tree_y - tree_size),
                        vec2(tree_x - tree_size * 0.7, tree_y + tree_size * 0.5),
                        vec2(tree_x + tree_size * 0.7, tree_y + tree_size * 0.5),
                        Color::from_rgba(34, 120, 34, 220),
                    );
                }
            }
        }

        // Draw origin point marker (glowing circle)
        if tile.origin_point.is_some() {
            draw_circle(
                center.x,
                center.y,
                5.0,
                Color::from_rgba(255, 255, 100, 180),
            );
            draw_circle(center.x, center.y, 3.0, Color::from_rgba(255, 200, 0, 255));
        }
    }

    // Draw simplified coastline (only at higher zoom)
    if cam_zoom > 1.5 {
        for tile in &world.tiles {
            if tile.elevation < OCEAN_CUTOFF {
                continue;
            }
            let center = hex_center(tile.col, tile.row, hex_size);
            // Frustum culling
            if (center.x - cam_target_x).abs() > view_w / 2.0 + margin {
                continue;
            }
            if (center.y - cam_target_y).abs() > view_h / 2.0 + margin {
                continue;
            }

            // Check each edge for ocean neighbor
            let corners = [
                hex_corner(center, hex_size, 0),
                hex_corner(center, hex_size, 1),
                hex_corner(center, hex_size, 2),
                hex_corner(center, hex_size, 3),
                hex_corner(center, hex_size, 4),
                hex_corner(center, hex_size, 5),
            ];
            let neighbors = hex_neighbors(tile.col, tile.row);
            for i in 0..6 {
                let (nc, nr) = neighbors[i];
                if world
                    .get_tile(nc, nr)
                    .map_or(true, |t| t.elevation < OCEAN_CUTOFF)
                {
                    let next = (i + 1) % 6;
                    draw_line(
                        corners[i].x,
                        corners[i].y,
                        corners[next].x,
                        corners[next].y,
                        2.0,
                        Color::from_rgba(255, 255, 255, 220),
                    );
                }
            }
        }
    }

    // Draw hex grid (only at higher zoom)
    if cam_zoom > 2.0 {
        for tile in &world.tiles {
            let center = hex_center(tile.col, tile.row, hex_size);
            // Frustum culling
            if (center.x - cam_target_x).abs() > view_w / 2.0 + margin {
                continue;
            }
            if (center.y - cam_target_y).abs() > view_h / 2.0 + margin {
                continue;
            }
            draw_poly_lines(
                center.x,
                center.y,
                6,
                hex_size * overlap,
                -30.0,
                0.5,
                Color::from_rgba(0, 0, 0, 60),
            );
        }
    }
}

#[allow(dead_code)]
pub fn hex_center_world(col: i32, row: i32, hex_size: f32) -> Vec2 {
    hex_center(col, row, hex_size)
}

// Convert screen coordinates to world coordinates
pub fn screen_to_world(
    screen_x: f32,
    screen_y: f32,
    cam_target_x: f32,
    cam_target_y: f32,
    cam_zoom: f32,
) -> Vec2 {
    let sw = screen_width();
    let sh = screen_height();
    // Reverse the camera transform
    let world_x = (screen_x - sw / 2.0) / cam_zoom + cam_target_x;
    let world_y = (screen_y - sh / 2.0) / cam_zoom + cam_target_y;
    vec2(world_x, world_y)
}

// Find which tile is at the given world position
pub fn find_tile_at(
    world_x: f32,
    world_y: f32,
    width: i32,
    height: i32,
    hex_size: f32,
) -> Option<(i32, i32)> {
    // Reverse hex_center calculation
    // x = size * (sqrt(3) * col + sqrt(3)/2 * (row & 1))
    // y = size * 1.5 * row

    let row = (world_y / (hex_size * 1.5)).round() as i32;
    let row = row.max(0).min(height - 1);

    let parity = row & 1;
    let offset = if parity == 1 {
        hex_size * 3.0_f32.sqrt() / 2.0
    } else {
        0.0
    };
    let col = ((world_x - offset) / (hex_size * 3.0_f32.sqrt())).round() as i32;
    let col = col.max(0).min(width - 1);

    Some((col, row))
}

// Get tooltip text for a tile
pub fn get_tile_tooltip(tile: &HexTile) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(ref res) = tile.resource {
        parts.push(format!("{:?}", res.resource_type));
    }

    if let Some(ref leg) = tile.legendary_resource {
        parts.push(format!("★ {}", leg.name));
    }

    if let Some(ref ruin) = tile.ruin {
        parts.push(format!("Ruin: {:?}", ruin.ruin_type));
    }

    if tile.origin_point.is_some() {
        parts.push("Origin Point".to_string());
    }

    if tile.is_river {
        parts.push("River".to_string());
    }

    if !parts.is_empty() {
        Some(parts.join(", "))
    } else {
        None
    }
}
