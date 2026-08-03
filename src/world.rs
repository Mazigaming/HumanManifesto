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

#[derive(Clone, Copy, Debug)]
pub struct HexTile {
    pub col: i32,
    pub row: i32,
    pub elevation: f32,
    pub moisture: f32,
    pub temperature: f32,
    pub biome: Biome,
    pub is_river: bool,
    pub resource: Option<ResourceNode>,
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

fn hex_center(col: i32, row: i32, size: f32) -> Vec2 {
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
        let hex_size = 16.0;

        let mut tiles: Vec<HexTile> = Vec::with_capacity((width * height) as usize);

        for row in 0..height {
            for col in 0..width {
                let _center = hex_center(col, row, hex_size);
                // Distance-based continents with noise overlay
                let continent1_x = width as f32 * 0.3;
                let continent1_y = height as f32 * 0.4;
                let continent2_x = width as f32 * 0.7;
                let continent2_y = height as f32 * 0.6;

                let dist1 = ((col as f32 - continent1_x).powi(2)
                    + (row as f32 - continent1_y).powi(2))
                .sqrt();
                let dist2 = ((col as f32 - continent2_x).powi(2)
                    + (row as f32 - continent2_y).powi(2))
                .sqrt();
                let min_dist = dist1.min(dist2);

                // Scale continent radius to map size so it works on any map dimension
                let continent_radius = (width.min(height) as f32) * 0.35;

                // Base elevation from distance (1.0 at center, 0.0 at radius) - linear falloff
                let base_elevation = 1.0 - (min_dist / continent_radius).clamp(0.0, 1.0);

                // Blend noise for natural coastlines and terrain variation
                let nx = col as f32 * 0.1;
                let ny = row as f32 * 0.1;
                let continent_noise = fbm(nx * 0.5, ny * 0.5, seed, 3, 1.0);
                let detail_noise = fbm(nx, ny, seed + 100, 4, 2.0);
                let elevation =
                    (base_elevation * 0.65 + continent_noise * 0.25 + detail_noise * 0.1)
                        .clamp(0.0, 1.0);

                // Temperature: latitude-based with noise
                let lat = if height > 1 {
                    (row as f32 / (height - 1) as f32 - 0.5).abs() * 2.0
                } else {
                    0.0
                };
                let temp_noise = fbm(nx * 0.5, ny * 0.5, seed + 300, 3, 1.0);
                let temperature =
                    ((1.0 - lat) * 0.7 + temp_noise * 0.3 - elevation * 0.2).clamp(0.0, 1.0);

                // Moisture: noise-based
                let moisture = fbm(nx * 0.8, ny * 0.8, seed + 400, 3, 1.5).clamp(0.0, 1.0);

                tiles.push(HexTile {
                    col,
                    row,
                    elevation,
                    moisture,
                    temperature,
                    biome: Biome::Ocean,
                    is_river: false,
                    resource: None,
                });
            }
        }

        let mut world = World {
            width,
            height,
            seed,
            tiles,
        };

        // Pit-fill disabled - it drains elevations to ocean level instead of filling pits
        // world.pit_fill(pit_passes);
        println!("[DEBUG] Skipped pit_fill (drains elevations)");

        // Re-enable moisture falloff (now O(n) BFS)
        world.apply_moisture_falloff();
        println!("[DEBUG] After moisture_falloff: BFS flood-fill complete");

        world.assign_biomes();
        println!("[DEBUG] After assign_biomes: biomes assigned");

        world.trace_rivers();
        let river_count = world.tiles.iter().filter(|t| t.is_river).count();
        println!("[DEBUG] After trace_rivers: {} river tiles", river_count);

        world.place_resources();
        let resource_count = world.tiles.iter().filter(|t| t.resource.is_some()).count();
        println!(
            "[DEBUG] After place_resources: {} tiles with resources",
            resource_count
        );

        // DEBUG: Print comprehensive terrain stats
        println!("\n=== TERRAIN GENERATION DEBUG ===");
        println!(
            "Map size: {}x{} = {} tiles",
            world.width,
            world.height,
            world.tiles.len()
        );
        println!("Seed: {}", world.seed);

        let mut min_elev = f32::MAX;
        let mut max_elev = f32::MIN;
        let mut sum_elev = 0.0;
        let mut land_count = 0;
        let mut ocean_count = 0;
        let mut biome_counts = std::collections::HashMap::new();
        let mut elevation_buckets = [0; 10]; // 0.0-0.1, 0.1-0.2, etc.

        for tile in &world.tiles {
            min_elev = min_elev.min(tile.elevation);
            max_elev = max_elev.max(tile.elevation);
            sum_elev += tile.elevation;

            if tile.elevation >= OCEAN_CUTOFF {
                land_count += 1;
            } else {
                ocean_count += 1;
            }

            let bucket = (tile.elevation * 10.0) as usize;
            if bucket < 10 {
                elevation_buckets[bucket] += 1;
            }

            *biome_counts.entry(format!("{:?}", tile.biome)).or_insert(0) += 1;
        }

        let avg_elev = sum_elev / world.tiles.len() as f32;
        let _land_pct = land_count * 100 / world.tiles.len();

        println!("\n--- Elevation Statistics ---");
        println!(
            "Min: {:.3}, Max: {:.3}, Avg: {:.3}",
            min_elev, max_elev, avg_elev
        );
        println!("Ocean cutoff: {:.2}", OCEAN_CUTOFF);
        println!(
            "Land tiles: {} ({:.1}%)",
            land_count,
            land_count as f32 / world.tiles.len() as f32 * 100.0
        );
        println!(
            "Ocean tiles: {} ({:.1}%)",
            ocean_count,
            ocean_count as f32 / world.tiles.len() as f32 * 100.0
        );

        println!("\n--- Elevation Distribution ---");
        for i in 0..10 {
            let low = i as f32 / 10.0;
            let high = (i + 1) as f32 / 10.0;
            let count = elevation_buckets[i];
            let pct = count as f32 / world.tiles.len() as f32 * 100.0;
            let bar = "█".repeat((pct / 2.0) as usize);
            println!(
                "[{:.1}-{:.1}]: {:4} ({:5.1}%) {}",
                low, high, count, pct, bar
            );
        }

        println!("\n--- Biome Distribution ---");
        let mut biome_vec: Vec<_> = biome_counts.into_iter().collect();
        biome_vec.sort_by(|a, b| b.1.cmp(&a.1));
        for (biome, count) in biome_vec {
            let pct = count as f32 / world.tiles.len() as f32 * 100.0;
            println!("  {:20}: {:5} ({:.1}%)", biome, count, pct);
        }

        println!("\n--- Sample Tiles (center of map) ---");
        let center_col = world.width / 2;
        let center_row = world.height / 2;
        for row_offset in -2..=2 {
            for col_offset in -2..=2 {
                let col = center_col + col_offset;
                let row = center_row + row_offset;
                if let Some(tile) = world.get_tile(col, row) {
                    println!(
                        "  [{:3},{:3}] elev={:.3} temp={:.3} moist={:.3} biome={:?}",
                        col, row, tile.elevation, tile.temperature, tile.moisture, tile.biome
                    );
                }
            }
        }

        println!("=================================\n");

        world
    }

    fn get_tile(&self, col: i32, row: i32) -> Option<&HexTile> {
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

    // O(n) multi-source BFS flood-fill from all ocean tiles
    fn apply_moisture_falloff(&mut self) {
        let max_dist = 15.0_f32;
        let mut queue: std::collections::VecDeque<(i32, i32)> = std::collections::VecDeque::new();
        let mut dist_map = vec![f32::MAX; (self.width * self.height) as usize];

        // Seed BFS from all ocean tiles
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = tile_index(col, row, self.width).unwrap();
                if self.tiles[idx].elevation < OCEAN_CUTOFF {
                    dist_map[idx] = 0.0;
                    queue.push_back((col, row));
                }
            }
        }

        // BFS outward from ocean
        while let Some((col, row)) = queue.pop_front() {
            let idx = tile_index(col, row, self.width).unwrap();
            let dist = dist_map[idx];
            if dist >= max_dist {
                continue;
            }
            for (nc, nr) in hex_neighbors(col, row) {
                if nr < 0 || nr >= self.height {
                    continue;
                }
                if let Some(nidx) = tile_index(nc, nr, self.width) {
                    if nidx < dist_map.len() && dist_map[nidx] == f32::MAX {
                        dist_map[nidx] = dist + 1.0;
                        queue.push_back((nc, nr));
                    }
                }
            }
        }

        // Apply moisture bonus based on BFS distance
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = tile_index(col, row, self.width).unwrap();
                if self.tiles[idx].elevation < OCEAN_CUTOFF {
                    continue;
                }
                let dist = dist_map[idx];
                if dist < max_dist {
                    let bonus = (1.0 - dist / max_dist) * 0.3;
                    self.tiles[idx].moisture = (self.tiles[idx].moisture + bonus).clamp(0.0, 1.0);
                }
            }
        }
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
        let perlin = Perlin::new(layer_seed(self.seed, RIVER_SEED_OFFSET));
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = tile_index(col, row, self.width).unwrap();
                if self.tiles[idx].elevation < 0.75 || self.tiles[idx].is_river {
                    continue;
                }
                if perlin_roll(&perlin, col as f64 * RIVER_SCALE, row as f64 * RIVER_SCALE) < 0.1 {
                    continue;
                }
                self.tiles[idx].is_river = true;
                let mut current = (col, row);
                let mut visited = std::collections::HashSet::new();
                visited.insert(current);
                loop {
                    let current_idx = match tile_index(current.0, current.1, self.width) {
                        Some(i) => i,
                        None => break,
                    };
                    let mut lowest = self.tiles[current_idx].elevation;
                    let mut next = None;
                    for (nc, nr) in hex_neighbors(current.0, current.1) {
                        if visited.contains(&(nc, nr)) {
                            continue;
                        }
                        if let Some(t) = self.get_tile(nc, nr) {
                            if t.elevation < lowest {
                                lowest = t.elevation;
                                next = Some((nc, nr));
                            }
                        }
                    }
                    match next {
                        Some((nc, nr)) => {
                            let idx = tile_index(nc, nr, self.width).unwrap();
                            self.tiles[idx].is_river = true;
                            visited.insert((nc, nr));
                            current = (nc, nr);
                        }
                        None => break,
                    }
                }
            }
        }
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
