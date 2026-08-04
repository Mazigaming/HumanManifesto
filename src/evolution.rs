use crate::agent::*;
use crate::lineage::{generate_lineage_name, Lineage};
use crate::world::*;
use std::collections::HashMap;

// Sector for regional carrying capacity
#[derive(Clone, Debug)]
pub struct Sector {
    pub col_start: i32,
    pub row_start: i32,
    pub capacity: f32,
    pub current_load: usize,
}

pub struct EvolutionSim {
    pub agents: Vec<Agent>,
    pub lineages: Vec<Lineage>,
    pub sectors: Vec<Sector>,
    pub sector_size: i32,
    next_id: u64,
    next_lineage_id: u32,
    pub tick_count: u64,
    pub disease_outbreaks: u32,
    pub total_births: u64,
    pub total_deaths: u64,
    pub chronicle: Vec<String>,
    // Spatial grid for O(1) agent lookups
    spatial_grid: HashMap<(i32, i32), Vec<usize>>,
    cell_size: i32,
    // Resource cache for fast lookups
    resource_cache: HashMap<(i32, i32), bool>,
    water_cache: HashMap<(i32, i32), bool>,
    // Reusable vectors to avoid allocations
    to_remove: Vec<usize>,
    neighbors_buf: Vec<usize>,
}

impl EvolutionSim {
    pub fn new() -> Self {
        EvolutionSim {
            agents: Vec::new(),
            lineages: Vec::new(),
            sectors: Vec::new(),
            sector_size: 20,
            next_id: 1,
            next_lineage_id: 1,
            tick_count: 0,
            disease_outbreaks: 0,
            total_births: 0,
            total_deaths: 0,
            chronicle: Vec::new(),
            spatial_grid: HashMap::new(),
            cell_size: 10,
            resource_cache: HashMap::new(),
            water_cache: HashMap::new(),
            to_remove: Vec::with_capacity(256),
            neighbors_buf: Vec::with_capacity(64),
        }
    }

    // Build sector grid based on world resource richness
    pub fn build_sectors(&mut self, world: &World) {
        self.sectors.clear();
        let cols = (world.width + self.sector_size - 1) / self.sector_size;
        let rows = (world.height + self.sector_size - 1) / self.sector_size;

        for r in 0..rows {
            for c in 0..cols {
                let col_start = c * self.sector_size;
                let row_start = r * self.sector_size;
                let col_end = ((c + 1) * self.sector_size).min(world.width);
                let row_end = ((r + 1) * self.sector_size).min(world.height);

                // Calculate capacity based on resource richness
                let mut total_richness = 0.0;
                let mut land_tiles = 0;
                for row in row_start..row_end {
                    for col in col_start..col_end {
                        if let Some(tile) = world.get_tile(col, row) {
                            if tile.elevation >= 0.3 {
                                land_tiles += 1;
                                if let Some(ref res) = tile.resource {
                                    total_richness += res.richness;
                                }
                            }
                        }
                    }
                }

                // Base capacity: 5 agents per land tile + bonus from resources
                let capacity = (land_tiles as f32 * 5.0 + total_richness * 10.0).max(1.0);

                self.sectors.push(Sector {
                    col_start,
                    row_start,
                    capacity,
                    current_load: 0,
                });
            }
        }
    }

    // Update sector loads based on current agent positions
    pub fn update_sector_loads(&mut self) {
        for sector in &mut self.sectors {
            sector.current_load = 0;
        }

        for agent in &self.agents {
            let sector_col = agent.col / self.sector_size;
            let sector_row = agent.row / self.sector_size;
            let cols = (200 + self.sector_size - 1) / self.sector_size; // Use world width
            let sector_idx = (sector_row * cols + sector_col) as usize;
            if sector_idx < self.sectors.len() {
                self.sectors[sector_idx].current_load += 1;
            }
        }
    }

    // Get overcrowding multiplier for a position (1.0 = normal, >1.0 = stressed)
    pub fn get_overcrowding_multiplier(&self, col: i32, row: i32) -> f32 {
        let sector_col = col / self.sector_size;
        let sector_row = row / self.sector_size;
        let cols = (200 + self.sector_size - 1) / self.sector_size;
        let sector_idx = (sector_row * cols + sector_col) as usize;

        if sector_idx < self.sectors.len() {
            let sector = &self.sectors[sector_idx];
            let load_ratio = sector.current_load as f32 / sector.capacity;
            if load_ratio > 1.0 {
                // Exponential increase in stress
                1.0 + (load_ratio - 1.0).powi(2) * 2.0
            } else {
                1.0
            }
        } else {
            1.0
        }
    }

    pub fn spawn_agent(&mut self, col: i32, row: i32, genome: Genome) -> u64 {
        let id = self.next_id;
        let lineage_id = self.next_lineage_id;
        self.next_id += 1;
        self.agents
            .push(Agent::new(id, lineage_id, col, row, genome));
        id
    }

    pub fn spawn_population(&mut self, col: i32, row: i32, count: usize) {
        let lineage_id = self.next_lineage_id;
        let lineage_name = generate_lineage_name(self.tick_count * 1000 + lineage_id as u64);
        let initial_genome = Genome::random();

        // Create root lineage
        let lineage = Lineage::new(
            lineage_id,
            None, // Root lineage has no parent
            lineage_name.clone(),
            self.tick_count,
            initial_genome,
        );
        self.lineages.push(lineage);
        self.next_lineage_id += 1;

        // Spawn agents with this lineage
        for _ in 0..count {
            let genome = Genome::random();
            let id = self.next_id;
            self.next_id += 1;
            self.agents
                .push(Agent::new(id, lineage_id, col, row, genome));
        }

        // Log the founding
        self.chronicle.push(format!(
            "Lineage {} founded at ({}, {}) with {} individuals",
            lineage_name, col, row, count
        ));
    }

    // Build spatial grid for O(1) agent lookups
    fn build_spatial_grid(&mut self) {
        self.spatial_grid.clear();
        for i in 0..self.agents.len() {
            let agent = &self.agents[i];
            let cell_x = agent.col / self.cell_size;
            let cell_y = agent.row / self.cell_size;
            self.spatial_grid
                .entry((cell_x, cell_y))
                .or_insert_with(Vec::new)
                .push(i);
        }
    }

    // Build resource and water caches
    fn build_caches(&mut self, world: &World) {
        self.resource_cache.clear();
        self.water_cache.clear();
        for tile in &world.tiles {
            if tile.resource.is_some() {
                self.resource_cache.insert((tile.col, tile.row), true);
            }
            if tile.is_river || tile.elevation < 0.3 {
                self.water_cache.insert((tile.col, tile.row), true);
            }
        }
    }

    // Update lineage centroids and record population history
    fn update_lineages(&mut self) {
        // Group agents by lineage
        let mut lineage_agents: HashMap<u32, Vec<usize>> = HashMap::new();
        for (i, agent) in self.agents.iter().enumerate() {
            lineage_agents
                .entry(agent.lineage_id)
                .or_insert_with(Vec::new)
                .push(i);
        }

        // Update each lineage
        for lineage in &mut self.lineages {
            if let Some(agent_indices) = lineage_agents.get(&lineage.id) {
                let genomes: Vec<Genome> = agent_indices
                    .iter()
                    .map(|&i| self.agents[i].genome)
                    .collect();
                lineage.update_centroid(&genomes);
            } else {
                lineage.member_count = 0;
                lineage.is_extinct = true;
            }
            lineage.record_population(self.tick_count);
        }

        // Log extinctions
        for lineage in &self.lineages {
            if lineage.is_extinct && lineage.member_count == 0 {
                let msg = format!(
                    "Lineage {} has gone extinct after {} days",
                    lineage.name,
                    self.tick_count - lineage.founding_tick
                );
                if !self.chronicle.contains(&msg) {
                    self.chronicle.push(msg);
                }
            }
        }
    }

    // Check for speciation splits based on genetic distance
    fn check_speciation(&mut self) {
        let split_threshold = 0.35; // Genetic distance threshold for split
        let min_split_count = 5; // Minimum agents needed for a new lineage
        let min_split_pct = 0.15; // Minimum percentage of lineage for split
        let persistence_required = 3; // Checks needed to confirm split

        // Track candidate splits across checks
        let mut candidate_groups: HashMap<u32, Vec<(u32, Vec<usize>)>> = HashMap::new();

        for lineage in &self.lineages {
            if lineage.is_extinct || lineage.member_count < 10 {
                continue;
            }

            // Get agents in this lineage
            let agent_indices: Vec<usize> = self
                .agents
                .iter()
                .enumerate()
                .filter(|(_, a)| a.lineage_id == lineage.id)
                .map(|(i, _)| i)
                .collect();

            if agent_indices.len() < 10 {
                continue;
            }

            // Group agents by distance from centroid
            let mut near_centroid: Vec<usize> = Vec::new();
            let mut far_from_centroid: Vec<usize> = Vec::new();

            for &idx in &agent_indices {
                let distance = lineage.genetic_distance(&self.agents[idx].genome);
                if distance < split_threshold {
                    near_centroid.push(idx);
                } else {
                    far_from_centroid.push(idx);
                }
            }

            // Check if far group qualifies as split candidate
            let far_count = far_from_centroid.len();
            let far_pct = far_count as f32 / agent_indices.len() as f32;

            if far_count >= min_split_count && far_pct >= min_split_pct {
                // Check internal cohesion of far group
                let avg_pairwise_dist = self.average_pairwise_distance(&far_from_centroid);
                if avg_pairwise_dist < split_threshold * 0.7 {
                    // Valid candidate - track it
                    candidate_groups
                        .entry(lineage.id)
                        .or_insert_with(Vec::new)
                        .push((lineage.id, far_from_centroid));
                }
            }
        }

        // Process confirmed splits
        for (lineage_id, candidates) in &candidate_groups {
            if candidates.len() >= persistence_required {
                // Take the most recent candidate
                if let Some((_, far_group)) = candidates.last() {
                    self.finalize_split(*lineage_id, far_group.clone());
                }
            }
        }
    }

    // Calculate average pairwise genetic distance within a group
    fn average_pairwise_distance(&self, agent_indices: &[usize]) -> f32 {
        if agent_indices.len() < 2 {
            return 0.0;
        }

        let mut total_dist = 0.0;
        let mut count = 0;

        for i in 0..agent_indices.len() {
            for j in (i + 1)..agent_indices.len() {
                let g1 = &self.agents[agent_indices[i]].genome;
                let g2 = &self.agents[agent_indices[j]].genome;
                let dist = ((g1.speed - g2.speed).powi(2)
                    + (g1.strength - g2.strength).powi(2)
                    + (g1.fertility - g2.fertility).powi(2)
                    + (g1.metabolism - g2.metabolism).powi(2)
                    + (g1.aggression - g2.aggression).powi(2)
                    + (g1.sociability - g2.sociability).powi(2)
                    + (g1.camouflage - g2.camouflage).powi(2)
                    + (g1.lifespan - g2.lifespan).powi(2)
                    + (g1.sight_range - g2.sight_range).powi(2)
                    + (g1.cold_tolerance - g2.cold_tolerance).powi(2)
                    + (g1.heat_tolerance - g2.heat_tolerance).powi(2))
                .sqrt()
                    / 3.162;
                total_dist += dist;
                count += 1;
            }
        }

        if count > 0 {
            total_dist / count as f32
        } else {
            0.0
        }
    }

    // Finalize a speciation split
    fn finalize_split(&mut self, parent_lineage_id: u32, far_group: Vec<usize>) {
        // Create new lineage
        let new_lineage_id = self.next_lineage_id;
        self.next_lineage_id += 1;

        let new_name = generate_lineage_name(self.tick_count * 1000 + new_lineage_id as u64);

        // Compute initial centroid from far group
        let genomes: Vec<Genome> = far_group.iter().map(|&i| self.agents[i].genome).collect();

        let mut initial_centroid = Genome {
            speed: 0.0,
            strength: 0.0,
            fertility: 0.0,
            metabolism: 0.0,
            aggression: 0.0,
            sociability: 0.0,
            camouflage: 0.0,
            lifespan: 0.0,
            sight_range: 0.0,
            cold_tolerance: 0.0,
            heat_tolerance: 0.0,
        };

        let count = genomes.len() as f32;
        for g in &genomes {
            initial_centroid.speed += g.speed;
            initial_centroid.strength += g.strength;
            initial_centroid.fertility += g.fertility;
            initial_centroid.metabolism += g.metabolism;
            initial_centroid.aggression += g.aggression;
            initial_centroid.sociability += g.sociability;
            initial_centroid.camouflage += g.camouflage;
            initial_centroid.lifespan += g.lifespan;
            initial_centroid.sight_range += g.sight_range;
            initial_centroid.cold_tolerance += g.cold_tolerance;
            initial_centroid.heat_tolerance += g.heat_tolerance;
        }

        initial_centroid.speed /= count;
        initial_centroid.strength /= count;
        initial_centroid.fertility /= count;
        initial_centroid.metabolism /= count;
        initial_centroid.aggression /= count;
        initial_centroid.sociability /= count;
        initial_centroid.camouflage /= count;
        initial_centroid.lifespan /= count;
        initial_centroid.sight_range /= count;
        initial_centroid.cold_tolerance /= count;
        initial_centroid.heat_tolerance /= count;

        let new_lineage = Lineage::new(
            new_lineage_id,
            Some(parent_lineage_id),
            new_name.clone(),
            self.tick_count,
            initial_centroid,
        );
        self.lineages.push(new_lineage);

        // Reassign agents to new lineage
        for &idx in &far_group {
            self.agents[idx].lineage_id = new_lineage_id;
        }

        // Log the split
        let msg = format!(
            "Lineage {} splits from parent — {} individuals diverge",
            new_name,
            far_group.len()
        );
        self.chronicle.push(msg);
    }

    pub fn tick(&mut self, world: &World) {
        self.tick_count += 1;

        // Build sectors once at start
        if self.tick_count == 1 {
            self.build_sectors(world);
        }

        // Build spatial indexes every 10 ticks for performance
        // Safe because we process deaths at end of tick, so indices stay valid
        if self.tick_count % 10 == 0 {
            self.build_spatial_grid();
        }
        if self.tick_count % 10 == 0 {
            // Rebuild caches every 10 ticks (resources change slowly)
            self.build_caches(world);
            // Update sector loads every 10 ticks
            self.update_sector_loads();
        }

        // Update lineage centroids and check for speciation every 100 ticks
        if self.tick_count % 100 == 0 {
            self.update_lineages();
            self.check_speciation();
        }

        // Clear reusable vectors
        self.to_remove.clear();

        // Update each agent
        for i in 0..self.agents.len() {
            // Extract position first for overcrowding calculation
            let agent_col = self.agents[i].col;
            let agent_row = self.agents[i].row;
            let overcrowding = self.get_overcrowding_multiplier(agent_col, agent_row);

            let agent = &mut self.agents[i];

            // Age
            agent.age += 1;

            // Decay memories
            agent.decay_memories();

            // Reduce cooldowns
            if agent.repro_cooldown > 0 {
                agent.repro_cooldown -= 1;
            }
            if agent.highlight_timer > 0 {
                agent.highlight_timer -= 1;
            }

            // Energy drain (reduced for better survival)
            let energy_drain =
                0.15 + agent.genome.metabolism * 0.1 + agent.genome.sight_range * 0.05;
            agent.energy -= energy_drain;

            // Hydration drain (reduced)
            agent.hydration -= 0.1;

            // Environmental hazard damage (temperature mismatch, reduced)
            if let Some(tile) = world.get_tile(agent_col, agent_row) {
                let temp = tile.temperature;
                let cold_tol = agent.genome.cold_tolerance;
                let heat_tol = agent.genome.heat_tolerance;
                // Cold damage if low cold tolerance in cold biomes
                if temp < 0.3 && cold_tol < 0.5 {
                    let damage = (0.3 - temp) * (0.5 - cold_tol) * 0.5 * overcrowding;
                    agent.health -= damage;
                }
                // Heat damage if low heat tolerance in hot biomes
                if temp > 0.7 && heat_tol < 0.5 {
                    let damage = (temp - 0.7) * (0.5 - heat_tol) * 0.5 * overcrowding;
                    agent.health -= damage;
                }
            }

            // Disease progression (less lethal, faster recovery)
            if agent.disease.infected {
                agent.disease.ticks_infected += 1;
                agent.health -= 0.15 * overcrowding;
                // Chance to recover or die
                if agent.disease.ticks_infected > 30 {
                    if rand_f32() < 0.08 {
                        agent.disease.infected = false;
                        agent.disease.immune = true;
                    }
                    if agent.disease.ticks_infected > 80 && rand_f32() < 0.005 {
                        agent.health = 0.0;
                    }
                }
            }

            // Starvation damage (reduced)
            if agent.energy <= 0.0 {
                agent.health -= 0.5; // Reduced from 2.0
                agent.energy = 0.0;
            }

            // Dehydration damage (reduced)
            if agent.hydration <= 0.0 {
                agent.health -= 0.8; // Reduced from 3.0
                agent.hydration = 0.0;
            }

            // Old age death (rising probability past lifespan)
            let max_age = (500.0 + agent.genome.lifespan * 1500.0) as u32;
            if agent.age > max_age {
                let death_chance = ((agent.age - max_age) as f32 / 500.0).min(0.1);
                if rand_f32() < death_chance {
                    agent.health = 0.0;
                }
            }

            // Check death
            if agent.health <= 0.0 {
                self.to_remove.push(i);
                continue;
            }

            // Clamp stats
            agent.energy = agent.energy.min(agent.max_energy);
            agent.hydration = agent.hydration.min(100.0);
            agent.health = agent.health.min(100.0);

            // Decision loop
            self.decide_action(i, world);
        }

        // Remove dead agents (reverse order to preserve indices)
        self.to_remove.sort();
        self.to_remove.reverse();
        for i in &self.to_remove {
            self.agents.remove(*i);
            self.total_deaths += 1;
        }

        // Random disease outbreaks
        if self.tick_count % 500 == 0 && !self.agents.is_empty() && rand_f32() < 0.3 {
            let idx = (rand_f32() * self.agents.len() as f32) as usize;
            if idx < self.agents.len() && !self.agents[idx].disease.immune {
                self.agents[idx].disease.infected = true;
                self.agents[idx].disease.ticks_infected = 0;
                self.disease_outbreaks += 1;
            }
        }
    }

    fn decide_action(&mut self, agent_idx: usize, world: &World) {
        // Copy needed values to avoid borrow conflicts
        let agent_col;
        let agent_row;
        let agent_energy;
        let agent_max_energy;
        let agent_hydration;
        let agent_repro_cooldown;
        let agent_sight;
        let agent_sociability;
        {
            let agent = &self.agents[agent_idx];
            agent_col = agent.col;
            agent_row = agent.row;
            agent_energy = agent.energy;
            agent_max_energy = agent.max_energy;
            agent_hydration = agent.hydration;
            agent_repro_cooldown = agent.repro_cooldown;
            agent_sight = agent.sight_radius();
            agent_sociability = agent.genome.sociability;
        }

        // 1. Flee danger (use spatial grid)
        if let Some((dc, dr)) = self.find_nearest_danger_spatial(agent_idx, agent_sight) {
            self.move_agent(agent_idx, -dc, -dr, world);
            self.agents[agent_idx].behavior = BehaviorState::Fleeing;
            return;
        }

        // 2. Seek food/water (use cache)
        if agent_energy < agent_max_energy * 0.5 {
            if let Some((col, row)) =
                self.find_nearest_resource_cached(agent_col, agent_row, agent_sight as i32)
            {
                self.move_agent(
                    agent_idx,
                    (col - agent_col) as f32,
                    (row - agent_row) as f32,
                    world,
                );
                self.agents[agent_idx].behavior = BehaviorState::SeekingFood;
                // Eat if adjacent
                if (col - agent_col).abs() <= 1 && (row - agent_row).abs() <= 1 {
                    self.eat_resource(agent_idx, col, row, world);
                }
                return;
            }
        }

        if agent_hydration < 50.0 {
            if let Some((col, row)) =
                self.find_nearest_water_cached(agent_col, agent_row, agent_sight as i32)
            {
                self.move_agent(
                    agent_idx,
                    (col - agent_col) as f32,
                    (row - agent_row) as f32,
                    world,
                );
                self.agents[agent_idx].behavior = BehaviorState::SeekingWater;
                // Drink if adjacent
                if (col - agent_col).abs() <= 1 && (row - agent_row).abs() <= 1 {
                    self.agents[agent_idx].hydration = 100.0;
                }
                return;
            }
        }

        // 3. Reproduce (use spatial grid)
        if agent_energy > agent_max_energy * 0.8
            && agent_hydration > 70.0
            && agent_repro_cooldown == 0
        {
            if let Some(mate_idx) = self.find_mate_spatial(agent_idx, agent_sight) {
                self.reproduce(agent_idx, mate_idx);
                self.agents[agent_idx].behavior = BehaviorState::Reproducing;
                return;
            }
        }

        // 4. Socialize (use spatial grid)
        if agent_sociability > 0.6 {
            if let Some((dc, dr)) = self.find_nearest_agent_spatial(agent_idx, agent_sight) {
                self.move_agent(agent_idx, dc, dr, world);
                self.agents[agent_idx].behavior = BehaviorState::Socializing;
                return;
            }
        }

        // 5. Wander
        let dc = (rand_f32() - 0.5) * 2.0;
        let dr = (rand_f32() - 0.5) * 2.0;
        self.move_agent(agent_idx, dc, dr, world);
        self.agents[agent_idx].behavior = BehaviorState::Wandering;
    }

    // Spatial grid-based danger search
    fn find_nearest_danger_spatial(&self, agent_idx: usize, sight: f32) -> Option<(f32, f32)> {
        let agent = &self.agents[agent_idx];
        let cell_x = agent.col / self.cell_size;
        let cell_y = agent.row / self.cell_size;
        let sight_cells = (sight / self.cell_size as f32).ceil() as i32;

        let mut nearest = None;
        let mut min_dist = sight;

        for dy in -sight_cells..=sight_cells {
            for dx in -sight_cells..=sight_cells {
                if let Some(cell_agents) = self.spatial_grid.get(&(cell_x + dx, cell_y + dy)) {
                    for &i in cell_agents {
                        if i == agent_idx {
                            continue;
                        }
                        let other = &self.agents[i];
                        let dc = (other.col - agent.col) as f32;
                        let dr = (other.row - agent.row) as f32;
                        let dist = (dc * dc + dr * dr).sqrt();

                        if dist < sight
                            && other.genome.aggression > 0.7
                            && other.genome.aggression > agent.genome.strength
                        {
                            if dist < min_dist {
                                min_dist = dist;
                                nearest = Some((dc, dr));
                            }
                        }
                    }
                }
            }
        }
        nearest
    }

    // Spatial grid-based mate search
    fn find_mate_spatial(&self, agent_idx: usize, sight: f32) -> Option<usize> {
        let agent = &self.agents[agent_idx];
        let cell_x = agent.col / self.cell_size;
        let cell_y = agent.row / self.cell_size;
        let sight_cells = (sight / self.cell_size as f32).ceil() as i32;

        let mut best_mate = None;
        let mut best_score = -1.0;

        for dy in -sight_cells..=sight_cells {
            for dx in -sight_cells..=sight_cells {
                if let Some(cell_agents) = self.spatial_grid.get(&(cell_x + dx, cell_y + dy)) {
                    for &i in cell_agents {
                        if i == agent_idx {
                            continue;
                        }
                        let other = &self.agents[i];
                        let dc = (other.col - agent.col) as f32;
                        let dr = (other.row - agent.row) as f32;
                        let dist = (dc * dc + dr * dr).sqrt();

                        if dist < sight
                            && other.repro_cooldown == 0
                            && other.energy > other.max_energy * 0.5
                        {
                            let compat = 1.0
                                - (agent.genome.aggression - other.genome.aggression).abs() * 0.5
                                - (agent.genome.sociability - other.genome.sociability).abs() * 0.3;
                            let score = compat - dist * 0.1;

                            if score > best_score {
                                best_score = score;
                                best_mate = Some(i);
                            }
                        }
                    }
                }
            }
        }
        best_mate
    }

    // Spatial grid-based nearest agent search
    fn find_nearest_agent_spatial(&self, agent_idx: usize, sight: f32) -> Option<(f32, f32)> {
        let agent = &self.agents[agent_idx];
        let cell_x = agent.col / self.cell_size;
        let cell_y = agent.row / self.cell_size;
        let sight_cells = (sight / self.cell_size as f32).ceil() as i32;

        let mut nearest = None;
        let mut min_dist = sight;

        for dy in -sight_cells..=sight_cells {
            for dx in -sight_cells..=sight_cells {
                if let Some(cell_agents) = self.spatial_grid.get(&(cell_x + dx, cell_y + dy)) {
                    for &i in cell_agents {
                        if i == agent_idx {
                            continue;
                        }
                        let other = &self.agents[i];
                        let dc = (other.col - agent.col) as f32;
                        let dr = (other.row - agent.row) as f32;
                        let dist = (dc * dc + dr * dr).sqrt();

                        if dist < min_dist {
                            min_dist = dist;
                            nearest = Some((dc, dr));
                        }
                    }
                }
            }
        }
        nearest
    }

    // Cache-based resource search
    fn find_nearest_resource_cached(&self, col: i32, row: i32, sight: i32) -> Option<(i32, i32)> {
        let mut best = None;
        let mut min_dist = sight as f32;

        for dr in -sight..=sight {
            for dc in -sight..=sight {
                let c = col + dc;
                let r = row + dr;
                if self.resource_cache.contains_key(&(c, r)) {
                    let dist = (dc * dc + dr * dr) as f32;
                    if dist < min_dist {
                        min_dist = dist;
                        best = Some((c, r));
                    }
                }
            }
        }
        best
    }

    // Cache-based water search
    fn find_nearest_water_cached(&self, col: i32, row: i32, sight: i32) -> Option<(i32, i32)> {
        let mut best = None;
        let mut min_dist = sight as f32;

        for dr in -sight..=sight {
            for dc in -sight..=sight {
                let c = col + dc;
                let r = row + dr;
                if self.water_cache.contains_key(&(c, r)) {
                    let dist = (dc * dc + dr * dr) as f32;
                    if dist < min_dist {
                        min_dist = dist;
                        best = Some((c, r));
                    }
                }
            }
        }
        best
    }

    fn move_agent(&mut self, agent_idx: usize, dc: f32, dr: f32, world: &World) {
        let speed = 0.5 + self.agents[agent_idx].genome.speed * 1.5;
        let new_col = (self.agents[agent_idx].col as f32 + dc * speed).round() as i32;
        let new_row = (self.agents[agent_idx].row as f32 + dr * speed).round() as i32;
        let new_col = new_col.max(0).min(world.width - 1);
        let new_row = new_row.max(0).min(world.height - 1);
        self.agents[agent_idx].energy -= self.agents[agent_idx].genome.speed * 0.3;
        self.agents[agent_idx].col = new_col;
        self.agents[agent_idx].row = new_row;
    }

    fn eat_resource(&mut self, agent_idx: usize, col: i32, row: i32, _world: &World) {
        self.agents[agent_idx].energy += 50.0; // Increased from 30.0
        self.agents[agent_idx].add_memory(col, row, 1.0);
    }

    fn reproduce(&mut self, agent_idx: usize, mate_idx: usize) {
        let mutation_rate = 0.1;
        let mate_genome;
        let agent_genome_copy;
        let agent_fertility;
        let agent_lineage;
        let col;
        let row;
        {
            let agent = &self.agents[agent_idx];
            let mate = &self.agents[mate_idx];
            mate_genome = mate.genome;
            agent_genome_copy = agent.genome;
            agent_fertility = agent.genome.fertility;
            agent_lineage = agent.lineage_id;
            col = (agent.col + mate.col) / 2;
            row = (agent.row + mate.row) / 2;
        }

        let twin_count = if rand_f32() < agent_fertility * 0.1 {
            2
        } else {
            1
        };

        for _ in 0..twin_count {
            let child_genome = Genome::blend(&agent_genome_copy, &mate_genome, mutation_rate);
            let mut child = Agent::new(self.next_id, agent_lineage, col, row, child_genome);
            if rand_f32() < 0.02 {
                child.large_mutation = true;
                child.highlight_timer = 120;
            }
            self.agents.push(child);
            self.next_id += 1;
            self.total_births += 1;
        }

        self.agents[agent_idx].repro_cooldown = 200 - (agent_fertility * 80.0) as u32;
        self.agents[agent_idx].energy -= 35.0;
    }
}
