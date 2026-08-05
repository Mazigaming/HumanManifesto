use crate::agent::*;
use crate::belief::Belief;
use crate::diplomacy::DiplomaticRelation;
use crate::lineage::{generate_lineage_name, Lineage};
use crate::tribe::{CultureProfile, Tribe};
use crate::world::*;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct DebugStats {
    pub agents_start: usize,
    pub agents_end: usize,
    pub ate_food: usize,
    pub drank_water: usize,
    pub desperate: usize,
    pub vision_food_found: usize,
    pub memory_food_found: usize,
    pub births: usize,
    pub deaths_starvation: usize,
    pub deaths_dehydration: usize,
    pub deaths_environment: usize,
    pub deaths_disease: usize,
    pub deaths_old_age: usize,
    pub avg_energy: f32,
    pub avg_hydration: f32,
}

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
    pub tribes: Vec<Tribe>,
    pub beliefs: Vec<Belief>,
    pub diplomatic_relations: Vec<DiplomaticRelation>,
    pub sectors: Vec<Sector>,
    pub sector_size: i32,
    pub civilizations: Vec<crate::civilization::Civilization>,
    pub divine_influence: f64,
    next_id: u64,
    next_lineage_id: u32,
    next_tribe_id: u32,
    next_belief_id: u32,
    pub tick_count: u64,
    pub disease_outbreaks: u32,
    pub total_births: u64,
    pub total_deaths: u64,
    pub chronicle: Vec<String>,
    spatial_grid: HashMap<(i32, i32), Vec<usize>>,
    cell_size: i32,
    resource_cache: HashMap<(i32, i32), bool>,
    water_cache: HashMap<(i32, i32), bool>,
    tribe_candidate_tracker: HashMap<(i32, i32), u32>,
    to_remove: Vec<usize>,
    neighbors_buf: Vec<usize>,
    debug_stats: DebugStats,
}

impl EvolutionSim {
    pub fn new() -> Self {
        EvolutionSim {
            agents: Vec::new(),
            lineages: Vec::new(),
            tribes: Vec::new(),
            beliefs: Vec::new(),
            diplomatic_relations: Vec::new(),
            sectors: Vec::new(),
            sector_size: 20,
            civilizations: Vec::new(),
            divine_influence: 0.0,
            next_id: 1,
            next_lineage_id: 1,
            next_tribe_id: 1,
            next_belief_id: 1,
            tick_count: 0,
            disease_outbreaks: 0,
            total_births: 0,
            total_deaths: 0,
            chronicle: Vec::new(),
            spatial_grid: HashMap::new(),
            cell_size: 10,
            resource_cache: HashMap::new(),
            water_cache: HashMap::new(),
            tribe_candidate_tracker: HashMap::new(),
            to_remove: Vec::with_capacity(256),
            neighbors_buf: Vec::with_capacity(64),
            debug_stats: DebugStats::default(),
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
    pub fn update_sector_loads(&mut self, world: &World) {
        for sector in &mut self.sectors {
            sector.current_load = 0;
        }

        for agent in &self.agents {
            let sector_col = agent.col / self.sector_size;
            let sector_row = agent.row / self.sector_size;
            let cols = (world.width + self.sector_size - 1) / self.sector_size;
            let sector_idx = (sector_row * cols + sector_col) as usize;
            if sector_idx < self.sectors.len() {
                self.sectors[sector_idx].current_load += 1;
            }
        }
    }

    // Get overcrowding multiplier for a position (1.0 = normal, >1.0 = stressed)
    pub fn get_overcrowding_multiplier(&self, col: i32, row: i32, world: &World) -> f32 {
        let sector_col = col / self.sector_size;
        let sector_row = row / self.sector_size;
        let cols = (world.width + self.sector_size - 1) / self.sector_size;
        let sector_idx = (sector_row * cols + sector_col) as usize;

        if sector_idx < self.sectors.len() {
            let sector = &self.sectors[sector_idx];
            let load_ratio = sector.current_load as f32 / sector.capacity;
            if load_ratio > 1.0 {
                (1.0 + (load_ratio - 1.0) * 0.5).min(3.0)
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
            let mut agent = Agent::new(id, lineage_id, col, row, genome);
            agent.energy = agent.max_energy;
            agent.hydration = 100.0;
            self.agents.push(agent);
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
            if tile.is_river {
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
                    + (g1.heat_tolerance - g2.heat_tolerance).powi(2)
                    + (g1.sexuality - g2.sexuality).powi(2)
                    + (g1.intelligence - g2.intelligence).powi(2)
                    + (g1.curiosity - g2.curiosity).powi(2)
                    + (g1.conformity - g2.conformity).powi(2)
                    + (g1.creativity - g2.creativity).powi(2)
                    + (g1.leadership - g2.leadership).powi(2))
                .sqrt()
                    / 4.0;
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
            sexuality: 0.0,
            intelligence: 0.0,
            curiosity: 0.0,
            conformity: 0.0,
            creativity: 0.0,
            leadership: 0.0,
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
            initial_centroid.sexuality += g.sexuality;
            initial_centroid.intelligence += g.intelligence;
            initial_centroid.curiosity += g.curiosity;
            initial_centroid.conformity += g.conformity;
            initial_centroid.creativity += g.creativity;
            initial_centroid.leadership += g.leadership;
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
        initial_centroid.sexuality /= count;
        initial_centroid.intelligence /= count;
        initial_centroid.curiosity /= count;
        initial_centroid.conformity /= count;
        initial_centroid.creativity /= count;
        initial_centroid.leadership /= count;

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

    pub fn tick(&mut self, world: &mut World) {
        self.tick_count += 1;

        // Build sectors once at start
        if self.tick_count == 1 {
            self.build_sectors(world);
        }

        // Build spatial indexes every tick so indices stay valid after deaths
        self.build_spatial_grid();
        if self.tick_count % 10 == 0 {
            // Rebuild caches every 10 ticks (resources change slowly)
            self.build_caches(world);
            // Update sector loads every 10 ticks
            self.update_sector_loads(world);
        }

        // Update lineage centroids and check for speciation every 100 ticks
        if self.tick_count % 100 == 0 {
            self.update_lineages();
            self.check_speciation();
        }

        // Phase 4 — Society & Culture every 300 ticks
        if self.tick_count % 300 == 0 {
            self.check_tribe_formation();
            self.update_tribe_cultures();
            self.manage_tribe_membership();
            self.update_tribe_knowledge();
        }

        // Phase 4 — Beliefs and Diplomacy every 400 ticks
        if self.tick_count % 400 == 0 {
            self.update_beliefs();
            self.update_diplomacy();
            self.process_trade();
        }

        // Phase 4 — Tribe conflicts every 600 ticks
        if self.tick_count % 600 == 0 {
            self.tick_tribe_conflicts();
        }

        // Phase 4 — Belief origination triggers (disease recovery, large mutation)
        if self.tick_count % 200 == 0 {
            self.check_belief_origination();
        }

        // Regrow food around population centers every 100 ticks
        if self.tick_count % 100 == 0 {
            let agent_positions: Vec<(i32, i32)> =
                self.agents.iter().map(|a| (a.col, a.row)).collect();
            world.regrow_food(&agent_positions);
        }

        // Clear reusable vectors
        self.to_remove.clear();
        let mut births: Vec<Agent> = Vec::new();

        // Update each agent
        for i in 0..self.agents.len() {
            // Extract position first for overcrowding calculation
            let agent_col = self.agents[i].col;
            let agent_row = self.agents[i].row;
            let overcrowding = self.get_overcrowding_multiplier(agent_col, agent_row, world);
            let civ_bonus = self.get_civilization_bonus(agent_col, agent_row);

            let agent = &mut self.agents[i];

            // Look up tribe knowledge for bonuses
            let agent_tribe_knowledge = agent
                .tribe_id
                .and_then(|tid| {
                    self.tribes
                        .iter()
                        .find(|t| t.id == tid)
                        .map(|t| t.knowledge)
                })
                .unwrap_or(0.0);

            // Look up belief profile for effects
            let agent_belief_profile = agent.belief_id.and_then(|bid| {
                self.beliefs
                    .iter()
                    .find(|b| b.id == bid)
                    .map(|b| &b.tenet_profile)
            });

            // Look up tribe culture for bonuses
            let agent_tribe_culture = agent.tribe_id.and_then(|tid| {
                self.tribes
                    .iter()
                    .find(|t| t.id == tid)
                    .map(|t| &t.culture_profile)
            });

            // Age
            agent.age += 1;

            // Gain experience from surviving
            agent.experience += 0.1;

            // Grace period: new agents don't take damage for first 30 days
            let is_newborn = agent.age < 30;

            // Decay memories
            agent.decay_memories();

            // Reduce cooldowns
            if agent.repro_cooldown > 0 {
                agent.repro_cooldown -= 1;
            }
            if agent.highlight_timer > 0 {
                agent.highlight_timer -= 1;
            }

            // Pregnancy progression
            let is_pregnant = agent.pregnancy_days > 0;
            if is_pregnant {
                agent.pregnancy_days -= 1;
                // Extra energy drain during pregnancy (must be sustainable)
                agent.energy -= 0.02;
                if agent.pregnancy_days == 0 {
                    // Give birth!
                    if let Some(father_genome) = agent.pregnancy_father_genome {
                        let child_genome = Genome::blend(&agent.genome, &father_genome, 0.1);
                        let mut child = Agent::new(
                            self.next_id,
                            agent.lineage_id,
                            agent_col,
                            agent_row,
                            child_genome,
                        );
                        if rand_f32() < 0.02 {
                            child.large_mutation = true;
                            child.highlight_timer = 120;
                        }
                        births.push(child);
                        self.next_id += 1;
                        self.total_births += 1;
                        let child_id = self.next_id - 1;
                        let child_sex = if child.gender == Gender::Female {
                            "daughter"
                        } else {
                            "son"
                        };
                        self.chronicle.push(format!(
                            "Birth: {} gives birth to {} (#{})",
                            agent.lineage_id, child_sex, child_id
                        ));
                    }
                    agent.pregnancy_father_genome = None;
                    agent.repro_cooldown = 30; // Post-birth cooldown
                }
            }

            // Energy drain (strength and speed cost energy)
            let mut energy_drain = 0.05 + agent.genome.strength * 0.1 + agent.genome.speed * 0.1;
            if let Some(profile) = agent_belief_profile {
                if profile.asceticism > 0.6 {
                    energy_drain *= 0.95;
                }
            }
            agent.energy -= if is_pregnant {
                energy_drain * 1.3
            } else {
                energy_drain
            };

            // Hydration drain (metabolism drives thirst)
            let mut hydration_drain = 0.03 + agent.genome.metabolism * 0.05;
            if let Some(profile) = agent_belief_profile {
                if profile.asceticism > 0.6 {
                    hydration_drain *= 0.95;
                }
            }
            agent.hydration -= hydration_drain;

            // Passive regeneration when well-rested (simulates basic foraging/snacking)
            if agent.energy > 40.0 && agent.hydration > 40.0 && agent.age > 30 {
                let mut regen = 0.05 + agent.genome.intelligence * 0.08;
                let exp_bonus = (agent.experience / 1000.0).min(0.1);
                regen += exp_bonus;

                // Tribe knowledge bonus
                if agent_tribe_knowledge >= 50.0 {
                    regen += 0.03;
                }

                // Belief: ancestor reverence gives regen when part of a tribe
                if let Some(profile) = agent_belief_profile {
                    if profile.ancestor_reverence > 0.6 && agent.tribe_id.is_some() {
                        regen += 0.02;
                    }
                }

                // Tribe culture: communal gives regen
                if let Some(culture) = agent_tribe_culture {
                    if culture.communal > 0.6 {
                        regen += 0.02;
                    }
                }

                // Phase 6 — Civilization bonus
                regen += civ_bonus.regen_bonus;
                agent.hydration = (agent.hydration + civ_bonus.regen_bonus * 0.8).min(100.0);

                // Environmental proximity bonuses
                if let Some(tile) = world.get_tile(agent_col, agent_row) {
                    if let Some(ref res) = tile.resource {
                        match res.resource_type {
                            ResourceType::Timber => regen += 0.02,
                            ResourceType::MedicinalHerbs => regen += 0.03,
                            ResourceType::WildGrain => regen += 0.015,
                            _ => {}
                        }
                    }
                }

                agent.energy = (agent.energy + regen).min(agent.max_energy);
                agent.hydration = (agent.hydration + regen * 0.8).min(100.0);
            }

            // Environmental hazard damage (negligible)
            if let Some(tile) = world.get_tile(agent_col, agent_row) {
                let temp = tile.temperature;
                let cold_tol = agent.genome.cold_tolerance;
                let heat_tol = agent.genome.heat_tolerance;
                let mut damage_mult = 1.0;

                // Tribe knowledge environmental mastery
                if agent_tribe_knowledge >= 300.0 {
                    damage_mult *= 0.85;
                }

                // Proximity to stone provides shelter from elements
                if let Some(ref res) = tile.resource {
                    if res.resource_type == ResourceType::Stone {
                        damage_mult *= 0.5;
                    }
                }

                if temp < 0.3 && cold_tol < 0.5 {
                    let damage = (0.3 - temp) * (0.5 - cold_tol) * 0.005 * damage_mult;
                    agent.health -= damage;
                }
                if temp > 0.7 && heat_tol < 0.5 {
                    let damage = (temp - 0.7) * (0.5 - heat_tol) * 0.005 * damage_mult;
                    agent.health -= damage;
                }
            }

            // Disease progression (very mild)
            if agent.disease.infected {
                agent.disease.ticks_infected += 1;
                let mut health_drain = 0.05 * overcrowding;

                // Civilization disease resistance
                health_drain *= (1.0 - civ_bonus.disease_resistance).max(0.1);

                // Medicinal herbs nearby help fight disease
                if let Some(tile) = world.get_tile(agent_col, agent_row) {
                    if let Some(ref res) = tile.resource {
                        if res.resource_type == ResourceType::MedicinalHerbs {
                            health_drain *= 0.3;
                        }
                    }
                }

                agent.health -= health_drain;
                if agent.disease.ticks_infected > 30 {
                    if rand_f32() < 0.15 {
                        agent.disease.infected = false;
                        agent.disease.immune = true;
                    }
                    if agent.disease.ticks_infected > 80 && rand_f32() < 0.002 {
                        agent.health = 0.0;
                    }
                }
            }

            // Starvation damage (very mild, skip for newborns)
            if agent.energy <= 0.0 && !is_newborn {
                let mut health_drain = 0.15;
                if let Some(profile) = agent_belief_profile {
                    if profile.fatalism > 0.6 {
                        health_drain *= 0.9;
                    }
                }
                agent.health -= health_drain;
                agent.energy = 0.0;
            }

            // Dehydration damage (very mild, skip for newborns)
            if agent.hydration <= 0.0 && !is_newborn {
                let mut health_drain = 0.2;
                if let Some(profile) = agent_belief_profile {
                    if profile.fatalism > 0.6 {
                        health_drain *= 0.9;
                    }
                }
                agent.health -= health_drain;
                agent.hydration = 0.0;
            }

            // Old age death (rising probability past lifespan, skip newborns)
            if !is_newborn {
                let max_age = (2000.0 + agent.genome.lifespan * 20000.0) as u32;
                if agent.age > max_age {
                    let death_chance = ((agent.age - max_age) as f32 / 3000.0).min(0.02);
                    if rand_f32() < death_chance {
                        agent.health = 0.0;
                    }
                }
            }

            // Check death
            if agent.health <= 0.0 {
                if agent.energy <= 0.0 {
                    self.debug_stats.deaths_starvation += 1;
                } else if agent.hydration <= 0.0 {
                    self.debug_stats.deaths_dehydration += 1;
                } else if agent.disease.infected {
                    self.debug_stats.deaths_disease += 1;
                } else {
                    self.debug_stats.deaths_environment += 1;
                }
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

        // Debug stats: print every 30 days
        if self.tick_count % 30 == 0 && !self.agents.is_empty() {
            let mut total_energy = 0.0;
            let mut total_hydration = 0.0;
            for agent in &self.agents {
                total_energy += agent.energy;
                total_hydration += agent.hydration;
            }
            println!(
                "[DEBUG] Day {}: {} agents | avg energy: {:.1} | avg hydration: {:.1} | ate: {} | drank: {} | vision_food: {} | memory_food: {} | births: {} | deaths: {} | starving: {} | dehydrated: {}",
                self.tick_count,
                self.agents.len(),
                total_energy / self.agents.len() as f32,
                total_hydration / self.agents.len() as f32,
                self.debug_stats.ate_food,
                self.debug_stats.drank_water,
                self.debug_stats.vision_food_found,
                self.debug_stats.memory_food_found,
                self.debug_stats.births,
                self.total_deaths,
                self.debug_stats.deaths_starvation,
                self.debug_stats.deaths_dehydration,
            );
            self.debug_stats = DebugStats::default();
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
        let agent_pregnancy_days;
        let agent_intelligence;
        let agent_age;
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
            agent_pregnancy_days = agent.pregnancy_days;
            agent_intelligence = agent.genome.intelligence;
            agent_age = agent.age;
        }

        // 1. Flee danger (use spatial grid)
        if let Some((dc, dr)) = self.find_nearest_danger_spatial(agent_idx, agent_sight) {
            self.move_agent(agent_idx, -dc, -dr, world);
            self.agents[agent_idx].behavior = BehaviorState::Fleeing;
            return;
        }

        // 2. Seek food/water using direct vision (no cache)
        let is_desperate = agent_energy < agent_max_energy * 0.9 || agent_hydration < 80.0;
        let vision_radius = if is_desperate {
            (agent_sight + 30.0) as i32
        } else {
            (agent_sight + 10.0) as i32
        };

        // Smart agents revisit remembered food/water first
        if agent_intelligence > 0.05 && !is_desperate {
            if let Some((col, row)) = self.find_nearest_resource_memory(agent_idx) {
                let dc = (col - agent_col) as f32;
                let dr = (row - agent_row) as f32;
                let dist = (dc * dc + dr * dr).sqrt();
                if dist < 100.0 {
                    self.move_agent(agent_idx, dc, dr, world);
                    self.agents[agent_idx].behavior = BehaviorState::SeekingFood;
                    if (col - agent_col).abs() <= 1 && (row - agent_row).abs() <= 1 {
                        self.eat_resource(agent_idx, col, row, world);
                    }
                    return;
                }
            }
            if let Some((col, row)) = self.find_nearest_water_memory(agent_idx) {
                let dc = (col - agent_col) as f32;
                let dr = (row - agent_row) as f32;
                let dist = (dc * dc + dr * dr).sqrt();
                if dist < 80.0 {
                    self.move_agent(agent_idx, dc, dr, world);
                    self.agents[agent_idx].behavior = BehaviorState::SeekingWater;
                    if (col - agent_col).abs() <= 1 && (row - agent_row).abs() <= 1 {
                        self.agents[agent_idx].hydration = 100.0;
                        self.debug_stats.drank_water += 1;
                    }
                    return;
                }
            }
        }

        // Direct vision search for food
        if agent_energy < agent_max_energy * 0.99 || is_desperate {
            if let Some((col, row)) =
                self.find_nearest_resource_vision(agent_idx, world, vision_radius)
            {
                let dc = (col - agent_col) as f32;
                let dr = (row - agent_row) as f32;
                self.move_agent(agent_idx, dc, dr, world);
                self.agents[agent_idx].behavior = BehaviorState::SeekingFood;
                if (col - agent_col).abs() <= 1 && (row - agent_row).abs() <= 1 {
                    self.eat_resource(agent_idx, col, row, world);
                    self.debug_stats.vision_food_found += 1;
                }
                return;
            }
        }

        // Direct vision search for water
        if agent_hydration < 99.0 || is_desperate {
            if let Some((col, row)) =
                self.find_nearest_water_vision(agent_idx, world, vision_radius)
            {
                let dc = (col - agent_col) as f32;
                let dr = (row - agent_row) as f32;
                self.move_agent(agent_idx, dc, dr, world);
                self.agents[agent_idx].behavior = BehaviorState::SeekingWater;
                if (col - agent_col).abs() <= 1 && (row - agent_row).abs() <= 1 {
                    self.agents[agent_idx].hydration = 100.0;
                    self.debug_stats.drank_water += 1;
                }
                return;
            }
        }

        // 3. Reproduce (use spatial grid with expanded search)
        let can_reproduce_age = agent_age > 60;
        if can_reproduce_age
            && agent_energy > 80.0
            && agent_hydration > 50.0
            && agent_repro_cooldown == 0
            && agent_pregnancy_days == 0
        {
            // Use wider search for mates (3x normal sight) to combat low population density
            let mate_search_radius = agent_sight * 3.0;
            if let Some(mate_idx) = self.find_mate_spatial(agent_idx, mate_search_radius) {
                // Additional mate quality checks
                let mate = &self.agents[mate_idx];
                let mate_ok = mate.health > 70.0 && mate.energy > 80.0 && mate.hydration > 50.0;
                if mate_ok {
                    self.reproduce(agent_idx, mate_idx);
                    self.agents[agent_idx].behavior = BehaviorState::Reproducing;
                    return;
                }
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

        // 5. Wander with systematic exploration
        let agent_exploration_ticks = self.agents[agent_idx].exploration_ticks;
        if agent_exploration_ticks == 0 {
            let new_dc = (rand_f32() - 0.5) * 2.0;
            let new_dr = (rand_f32() - 0.5) * 2.0;
            self.agents[agent_idx].exploration_dc = new_dc;
            self.agents[agent_idx].exploration_dr = new_dr;
            // Smarter/curious agents explore more dynamically
            let base_ticks = 20 + (rand_f32() * 30.0) as u32;
            let intel_factor = 1.0 - agent_intelligence * 0.6;
            let curiosity_factor = 1.0 - self.agents[agent_idx].genome.curiosity * 0.4;
            self.agents[agent_idx].exploration_ticks =
                (base_ticks as f32 * intel_factor * curiosity_factor).max(5.0) as u32;
        }
        let dc = self.agents[agent_idx].exploration_dc;
        let dr = self.agents[agent_idx].exploration_dr;
        self.agents[agent_idx].exploration_ticks -= 1;
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

    // Spatial grid-based mate search (considers gender and sexuality)
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
                            && other.energy > other.max_energy * 0.3
                            && other.pregnancy_days == 0
                        // Can't mate if already pregnant
                        {
                            // Calculate gender/sexuality compatibility
                            // sexuality: 0.0=hetero, 0.5=bi, 1.0=homo
                            let same_gender = agent.gender == other.gender;

                            // Attraction based on sexuality
                            let attraction = if same_gender {
                                // Same-gender attraction increases with sexuality (homo)
                                agent.genome.sexuality
                            } else {
                                // Opposite-gender attraction decreases with sexuality (homo)
                                1.0 - agent.genome.sexuality
                            };

                            // Skip if no attraction
                            if attraction < 0.1 {
                                continue;
                            }

                            let compat = 1.0
                                - (agent.genome.aggression - other.genome.aggression).abs() * 0.5
                                - (agent.genome.sociability - other.genome.sociability).abs() * 0.3;
                            let score = compat * attraction - dist * 0.1;

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

    // Memory-based resource search (smarter agents remember food locations)
    fn find_nearest_resource_memory(&self, agent_idx: usize) -> Option<(i32, i32)> {
        let agent = &self.agents[agent_idx];
        let mut best = None;
        let mut best_dist = f32::MAX;
        let intelligence = agent.genome.intelligence;

        for slot in agent.memory.iter() {
            if let Some(m) = slot {
                if m.value > 0.0 && m.decay > 0.2 {
                    let dc = (m.col - agent.col) as f32;
                    let dr = (m.row - agent.row) as f32;
                    let dist = (dc * dc + dr * dr).sqrt();
                    if dist < best_dist && dist < 30.0 + intelligence * 20.0 {
                        best_dist = dist;
                        best = Some((m.col, m.row));
                    }
                }
            }
        }
        best
    }

    // Memory-based water search
    fn find_nearest_water_memory(&self, agent_idx: usize) -> Option<(i32, i32)> {
        let agent = &self.agents[agent_idx];
        let mut best = None;
        let mut best_dist = f32::MAX;
        let intelligence = agent.genome.intelligence;

        for slot in agent.memory.iter() {
            if let Some(m) = slot {
                if m.value > 0.0 && m.decay > 0.2 {
                    let dc = (m.col - agent.col) as f32;
                    let dr = (m.row - agent.row) as f32;
                    let dist = (dc * dc + dr * dr).sqrt();
                    if dist < best_dist && dist < 25.0 + intelligence * 15.0 {
                        best_dist = dist;
                        best = Some((m.col, m.row));
                    }
                }
            }
        }
        best
    }

    // Direct vision-based resource search (no cache dependency)
    fn find_nearest_resource_vision(
        &self,
        agent_idx: usize,
        world: &World,
        sight: i32,
    ) -> Option<(i32, i32)> {
        let agent = &self.agents[agent_idx];
        let mut best = None;
        let mut best_dist = sight as f32;

        for dr in -sight..=sight {
            for dc in -sight..=sight {
                let c = agent.col + dc;
                let r = agent.row + dr;
                if let Some(tile) = world.get_tile(c, r) {
                    if tile.resource.is_some() {
                        let dist = ((dc * dc + dr * dr) as f32).sqrt();
                        if dist < best_dist && dist > 0.0 {
                            best_dist = dist;
                            best = Some((c, r));
                        }
                    }
                }
            }
        }
        best
    }

    // Direct vision-based water search (any non-ocean water source)
    fn find_nearest_water_vision(
        &self,
        agent_idx: usize,
        world: &World,
        sight: i32,
    ) -> Option<(i32, i32)> {
        let agent = &self.agents[agent_idx];
        let mut best = None;
        let mut best_dist = sight as f32;

        for dr in -sight..=sight {
            for dc in -sight..=sight {
                let c = agent.col + dc;
                let r = agent.row + dr;
                if let Some(tile) = world.get_tile(c, r) {
                    if tile.is_river || tile.biome == Biome::Swamp || tile.moisture > 0.6 {
                        let dist = ((dc * dc + dr * dr) as f32).sqrt();
                        if dist < best_dist && dist > 0.0 {
                            best_dist = dist;
                            best = Some((c, r));
                        }
                    }
                }
            }
        }
        best
    }

    fn move_agent(&mut self, agent_idx: usize, dc: f32, dr: f32, world: &World) {
        let mut speed = 0.5 + self.agents[agent_idx].genome.speed * 1.5;

        // Civilization speed bonus
        let agent_col = self.agents[agent_idx].col;
        let agent_row = self.agents[agent_idx].row;
        let civ_bonus = self.get_civilization_bonus(agent_col, agent_row);
        speed *= 1.0 + civ_bonus.speed_bonus;

        // Tribe knowledge speed bonus
        if let Some(tid) = self.agents[agent_idx].tribe_id {
            if let Some(tribe) = self.tribes.iter().find(|t| t.id == tid) {
                if tribe.unlocked_tech.contains(&500) {
                    speed *= 1.08;
                }
                // Tribe culture: expansionist gives speed
                if tribe.culture_profile.expansionist > 0.6 {
                    speed *= 1.03;
                }
            }
        }

        let new_col = (self.agents[agent_idx].col as f32 + dc * speed).round() as i32;
        let new_row = (self.agents[agent_idx].row as f32 + dr * speed).round() as i32;
        let new_col = new_col.max(0).min(world.width - 1);
        let new_row = new_row.max(0).min(world.height - 1);
        self.agents[agent_idx].energy -= self.agents[agent_idx].genome.speed * 0.1;
        self.agents[agent_idx].col = new_col;
        self.agents[agent_idx].row = new_row;
    }

    fn eat_resource(&mut self, agent_idx: usize, col: i32, row: i32, _world: &World) {
        let civ_bonus = self.get_civilization_bonus(col, row);
        let agent = &mut self.agents[agent_idx];
        let exp_bonus = (agent.experience / 500.0).min(0.5);
        let tribe_bonus = agent
            .tribe_id
            .and_then(|tid| {
                self.tribes.iter().find(|t| t.id == tid).map(|t| {
                    let mut bonus = 0.0;
                    if t.unlocked_tech.contains(&150) {
                        bonus += 0.05;
                    }
                    if t.unlocked_tech.contains(&300) {
                        bonus += 0.05;
                    }
                    // Tribe culture: industrious gives foraging bonus
                    if t.culture_profile.industrious > 0.6 {
                        bonus += 0.03;
                    }
                    bonus
                })
            })
            .unwrap_or(0.0);
        agent.energy += 200.0 * (1.0 + exp_bonus + tribe_bonus + civ_bonus.food_bonus);
        agent.hydration += 80.0 * (1.0 + exp_bonus + tribe_bonus + civ_bonus.food_bonus);
        agent.add_memory(col, row, 1.0);
        self.debug_stats.ate_food += 1;
    }

    fn reproduce(&mut self, agent_idx: usize, mate_idx: usize) {
        self.debug_stats.births += 1;
        let mate_genome;
        let agent_genome_copy;
        let agent_fertility;
        let agent_gender;
        {
            let agent = &self.agents[agent_idx];
            let mate = &self.agents[mate_idx];
            mate_genome = mate.genome;
            agent_genome_copy = agent.genome;
            agent_fertility = agent.genome.fertility;
            agent_gender = agent.gender;
        }

        // Determine who gets pregnant (the female)
        let female_idx;
        let male_genome;
        if agent_gender == Gender::Female {
            female_idx = agent_idx;
            male_genome = mate_genome;
        } else {
            // mate should be female
            female_idx = mate_idx;
            male_genome = agent_genome_copy;
        }

        // Set pregnancy (90 days = 3 months)
        self.agents[female_idx].pregnancy_days = 90;
        self.agents[female_idx].pregnancy_father_genome = Some(male_genome);
        self.agents[female_idx].behavior = BehaviorState::Pregnant;

        // Both get cooldowns
        self.agents[agent_idx].repro_cooldown = 30 - (agent_fertility * 15.0) as u32;
        self.agents[agent_idx].energy -= 10.0;
        self.agents[mate_idx].repro_cooldown = 30 - (mate_genome.fertility * 15.0) as u32;
        self.agents[mate_idx].energy -= 10.0;
    }

    // Phase 4 — Tribe formation detection
    fn check_tribe_formation(&mut self) {
        let min_cluster_size = 4;
        let persistence_required = 2;
        let cluster_radius = 18;

        let mut cluster_counts: HashMap<(i32, i32), Vec<usize>> = HashMap::new();

        for (i, agent) in self.agents.iter().enumerate() {
            if agent.tribe_id.is_some() {
                continue;
            }
            let cell_x = agent.col / cluster_radius;
            let cell_y = agent.row / cluster_radius;
            cluster_counts
                .entry((cell_x, cell_y))
                .or_insert_with(Vec::new)
                .push(i);
        }

        let mut new_candidates: Vec<(i32, i32)> = Vec::new();
        for (cell, members) in &cluster_counts {
            if members.len() >= min_cluster_size {
                new_candidates.push(*cell);
            }
        }

        for cell in &new_candidates {
            let counter = self.tribe_candidate_tracker.entry(*cell).or_insert(0);
            *counter += 1;
        }

        let mut to_remove: Vec<(i32, i32)> = Vec::new();
        for (cell, _count) in &self.tribe_candidate_tracker {
            if !new_candidates.contains(cell) {
                to_remove.push(*cell);
            }
        }
        for cell in &to_remove {
            self.tribe_candidate_tracker.remove(cell);
        }

        let mut formed: Vec<(i32, i32, Vec<usize>)> = Vec::new();
        for (cell, count) in &self.tribe_candidate_tracker {
            if *count >= persistence_required {
                if let Some(members) = cluster_counts.get(cell) {
                    formed.push((cell.0, cell.1, members.clone()));
                }
            }
        }

        for (cell_x, cell_y, members) in formed {
            let first_idx = members[0];
            let first_agent = &self.agents[first_idx];
            let first_col = first_agent.col;
            let first_row = first_agent.row;
            let tribe_name =
                generate_lineage_name(self.tick_count * 1000 + self.next_tribe_id as u64);
            let tribe = Tribe::new(
                self.next_tribe_id,
                tribe_name.clone(),
                self.tick_count,
                first_agent,
            );
            self.tribes.push(tribe);
            let tribe_id = self.next_tribe_id;
            self.next_tribe_id += 1;

            for &idx in &members {
                self.agents[idx].tribe_id = Some(tribe_id);
            }

            self.chronicle.push(format!(
                "Tribe {} founded near ({}, {}) with {} members",
                tribe_name,
                first_col,
                first_row,
                members.len()
            ));
            self.tribe_candidate_tracker.remove(&(cell_x, cell_y));
        }
    }

    // Phase 4 — Update tribe cultures
    fn update_tribe_cultures(&mut self) {
        for tribe in &mut self.tribes {
            if tribe.is_extinct {
                continue;
            }
            let member_refs: Vec<&Agent> = tribe
                .member_ids
                .iter()
                .filter_map(|id| self.agents.iter().find(|a| a.id == *id))
                .collect();
            if member_refs.is_empty() {
                tribe.is_extinct = true;
                continue;
            }
            tribe.culture_profile = CultureProfile::from_agents(&member_refs);
            let mut sum_col = 0;
            let mut sum_row = 0;
            for agent in &member_refs {
                sum_col += agent.col;
                sum_row += agent.row;
            }
            tribe.territory_center = (
                sum_col / member_refs.len() as i32,
                sum_row / member_refs.len() as i32,
            );
            tribe.territory_radius =
                (15.0 + member_refs.len() as f32 * 2.0 + tribe.culture_profile.expansionist * 20.0)
                    .min(80.0);
            tribe.record_population(self.tick_count);
        }
    }

    // Phase 5 — Tribe knowledge accumulation and technology unlocks
    fn update_tribe_knowledge(&mut self) {
        let tech_thresholds: [f32; 4] = [50.0, 150.0, 300.0, 500.0];

        for tribe in &mut self.tribes {
            if tribe.is_extinct {
                continue;
            }

            let member_ids: Vec<u64> = tribe.member_ids.clone();
            let mut living_members = 0;
            let mut total_intelligence = 0.0;

            for id in &member_ids {
                if let Some(agent) = self.agents.iter().find(|a| a.id == *id) {
                    living_members += 1;
                    total_intelligence += agent.genome.intelligence;
                }
            }

            if living_members == 0 {
                tribe.is_extinct = true;
                continue;
            }

            let avg_intelligence = total_intelligence / living_members as f32;
            let knowledge_gain = living_members as f32 * 0.5 + avg_intelligence * 2.0;
            tribe.knowledge += knowledge_gain;

            for &threshold in &tech_thresholds {
                if tribe.knowledge >= threshold
                    && !tribe.unlocked_tech.contains(&(threshold as u32))
                {
                    tribe.unlocked_tech.push(threshold as u32);
                    self.chronicle.push(format!(
                        "Tribe {} unlocks knowledge threshold {:.0}",
                        tribe.name, threshold
                    ));
                }
            }
        }
    }

    // Phase 4 — Manage tribe membership (join/leave/dissolve)
    fn manage_tribe_membership(&mut self) {
        let min_tribe_pop = 2;
        let join_range = 40;
        let leave_threshold = 0.1;

        for i in 0..self.agents.len() {
            let agent_tribe = self.agents[i].tribe_id;
            let _agent_col = self.agents[i].col;
            let _agent_row = self.agents[i].row;

            if agent_tribe.is_none() {
                if let Some((tribe_id, compat)) = self.find_nearest_tribe_for_join(i, join_range) {
                    if compat > 0.3 {
                        self.agents[i].tribe_id = Some(tribe_id);
                        if let Some(tribe) = self.tribes.iter_mut().find(|t| t.id == tribe_id) {
                            tribe.member_ids.push(self.agents[i].id);
                        }
                    }
                }
            } else {
                let tribe_id = agent_tribe.unwrap();
                if let Some(tribe) = self.tribes.iter().find(|t| t.id == tribe_id) {
                    let compat = self.agent_tribe_culture_compatibility(i, &tribe.culture_profile);
                    if compat < leave_threshold || tribe.member_count() < min_tribe_pop {
                        let will_dissolve = tribe.member_count() < min_tribe_pop;
                        self.agents[i].tribe_id = None;
                        if let Some(tribe) = self.tribes.iter_mut().find(|t| t.id == tribe_id) {
                            tribe.member_ids.retain(|&id| id != self.agents[i].id);
                        }
                        if will_dissolve {
                            if let Some(tribe) = self.tribes.iter_mut().find(|t| t.id == tribe_id) {
                                tribe.is_extinct = true;
                                self.chronicle
                                    .push(format!("Tribe {} has dissolved", tribe.name));
                            }
                        }
                    }
                }
            }
        }

        // Absorb unaffiliated agents into nearby compatible tribes
        for i in 0..self.agents.len() {
            if self.agents[i].tribe_id.is_none() {
                if let Some((tribe_id, _)) = self.find_nearest_tribe_for_join(i, join_range * 2) {
                    self.agents[i].tribe_id = Some(tribe_id);
                    if let Some(tribe) = self.tribes.iter_mut().find(|t| t.id == tribe_id) {
                        tribe.member_ids.push(self.agents[i].id);
                    }
                }
            }
        }
    }

    fn find_nearest_tribe_for_join(&self, agent_idx: usize, range: i32) -> Option<(u32, f32)> {
        let agent = &self.agents[agent_idx];
        let mut best = None;
        let mut best_compat = 0.0;

        for tribe in &self.tribes {
            if tribe.is_extinct {
                continue;
            }
            let dc = tribe.territory_center.0 - agent.col;
            let dr = tribe.territory_center.1 - agent.row;
            let dist = ((dc * dc + dr * dr) as f32).sqrt();
            let effective_range = (tribe.territory_radius + range as f32).min(100.0);
            if dist < effective_range {
                let compat =
                    self.agent_tribe_culture_compatibility(agent_idx, &tribe.culture_profile);
                if compat > best_compat {
                    best_compat = compat;
                    best = Some((tribe.id, compat));
                }
            }
        }
        best
    }

    fn agent_tribe_culture_compatibility(&self, agent_idx: usize, culture: &CultureProfile) -> f32 {
        let agent = &self.agents[agent_idx];
        let agent_culture = CultureProfile::from_agents(&[agent]);
        agent_culture.compatibility(culture)
    }

    // Phase 4 — Belief system: origination, spread, schism
    fn update_beliefs(&mut self) {
        let spread_chance = 0.05;
        let schism_threshold = 0.7;
        let schism_persistence = 3;

        let agent_count = self.agents.len();
        let mut to_infect: Vec<(usize, u32)> = Vec::new();

        for i in 0..agent_count {
            let agent = &self.agents[i];
            if let Some(belief_id) = agent.belief_id {
                if rand_f32() < spread_chance {
                    let cell_x = agent.col / self.cell_size;
                    let cell_y = agent.row / self.cell_size;
                    let sight_cells = 2;
                    for dy in -sight_cells..=sight_cells {
                        for dx in -sight_cells..=sight_cells {
                            if let Some(cell_agents) =
                                self.spatial_grid.get(&(cell_x + dx, cell_y + dy))
                            {
                                for &j in cell_agents {
                                    if j < agent_count && self.agents[j].belief_id.is_none() {
                                        let other = &self.agents[j];
                                        let same_tribe = agent.tribe_id == other.tribe_id;
                                        let base_rate = if same_tribe { 0.3 } else { 0.1 };
                                        let should_spread = if let Some(rel) = self
                                            .get_diplomatic_relation(agent.tribe_id, other.tribe_id)
                                        {
                                            if rel.score > 30.0 {
                                                rand_f32() < base_rate * 1.5
                                            } else if rel.score < -50.0 {
                                                rand_f32() < base_rate * 0.2
                                            } else {
                                                rand_f32() < base_rate
                                            }
                                        } else {
                                            rand_f32() < base_rate
                                        };
                                        if should_spread {
                                            to_infect.push((j, belief_id));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for (idx, belief_id) in to_infect {
            self.agents[idx].belief_id = Some(belief_id);
        }

        // Count adherents per belief
        let mut adherent_counts: HashMap<u32, usize> = HashMap::new();
        for agent in &self.agents {
            if let Some(belief_id) = agent.belief_id {
                *adherent_counts.entry(belief_id).or_insert(0) += 1;
            }
        }
        for belief in &mut self.beliefs {
            if let Some(&count) = adherent_counts.get(&belief.id) {
                belief.record_adherents(self.tick_count, count);
            }
        }

        // Check for schisms
        let mut schism_candidates: HashMap<u32, Vec<u32>> = HashMap::new();
        for belief in &self.beliefs {
            if belief.parent_belief_id.is_none() {
                continue;
            }
            if let Some(parent) = self
                .beliefs
                .iter()
                .find(|b| b.id == belief.parent_belief_id.unwrap())
            {
                let dist = belief.tenet_profile.distance(&parent.tenet_profile);
                if dist > schism_threshold {
                    schism_candidates
                        .entry(parent.id)
                        .or_insert_with(Vec::new)
                        .push(belief.id);
                }
            }
        }

        for (parent_id, children) in &schism_candidates {
            if children.len() >= schism_persistence {
                for &child_id in children {
                    let child = self.beliefs.iter().find(|b| b.id == child_id);
                    if let Some(child) = child {
                        self.chronicle.push(format!(
                            "Belief '{}' schisms from '{}'",
                            child.name,
                            self.beliefs
                                .iter()
                                .find(|b| b.id == *parent_id)
                                .map(|b| b.name.as_str())
                                .unwrap_or("Unknown")
                        ));
                    }
                }
            }
        }
    }

    // Phase 4 — Diplomacy scoring
    fn update_diplomacy(&mut self) {
        let culture_weight = 0.4;
        let religion_weight = 0.3;

        let tribe_ids: Vec<u32> = self
            .tribes
            .iter()
            .filter(|t| !t.is_extinct)
            .map(|t| t.id)
            .collect();
        for ii in 0..tribe_ids.len() {
            for jj in (ii + 1)..tribe_ids.len() {
                let a = tribe_ids[ii];
                let b = tribe_ids[jj];

                let culture_compat = {
                    let tribe_a = self.tribes.iter().find(|t| t.id == a).unwrap();
                    let tribe_b = self.tribes.iter().find(|t| t.id == b).unwrap();
                    tribe_a
                        .culture_profile
                        .compatibility(&tribe_b.culture_profile)
                };

                let shared_belief = {
                    let tribe_a = self.tribes.iter().find(|t| t.id == a).unwrap();
                    let tribe_b = self.tribes.iter().find(|t| t.id == b).unwrap();
                    if tribe_a.member_ids.iter().any(|id| {
                        self.agents
                            .iter()
                            .any(|a| a.id == *id && a.belief_id.is_some())
                    }) && tribe_b.member_ids.iter().any(|id| {
                        self.agents
                            .iter()
                            .any(|a| a.id == *id && a.belief_id.is_some())
                    }) {
                        let a_beliefs: Vec<u32> = tribe_a
                            .member_ids
                            .iter()
                            .filter_map(|id| {
                                self.agents
                                    .iter()
                                    .find(|a| a.id == *id)
                                    .and_then(|a| a.belief_id)
                            })
                            .collect();
                        let b_beliefs: Vec<u32> = tribe_b
                            .member_ids
                            .iter()
                            .filter_map(|id| {
                                self.agents
                                    .iter()
                                    .find(|a| a.id == *id)
                                    .and_then(|a| a.belief_id)
                            })
                            .collect();
                        a_beliefs.iter().any(|b| b_beliefs.contains(b))
                    } else {
                        false
                    }
                };

                let target_score = culture_compat * 100.0 * culture_weight
                    + if shared_belief { 20.0 } else { -10.0 } * religion_weight;

                let tick = self.tick_count;
                let rel = self.get_or_create_relation(a, b);
                let drift = (target_score - rel.score) * 0.1;
                rel.score = (rel.score + drift).clamp(-100.0, 100.0);
                rel.last_updated_tick = tick;
            }
        }
    }

    fn get_or_create_relation(&mut self, a: u32, b: u32) -> &mut DiplomaticRelation {
        let key = (a.min(b), a.max(b));
        let idx = self
            .diplomatic_relations
            .iter()
            .position(|r| (r.tribe_a.min(r.tribe_b), r.tribe_a.max(r.tribe_b)) == key);
        if let Some(idx) = idx {
            &mut self.diplomatic_relations[idx]
        } else {
            self.diplomatic_relations
                .push(DiplomaticRelation::new(a, b, self.tick_count));
            self.diplomatic_relations.last_mut().unwrap()
        }
    }

    fn get_diplomatic_relation(
        &self,
        a: Option<u32>,
        b: Option<u32>,
    ) -> Option<&DiplomaticRelation> {
        if a.is_none() || b.is_none() {
            return None;
        }
        let a = a.unwrap();
        let b = b.unwrap();
        let key = (a.min(b), a.max(b));
        self.diplomatic_relations
            .iter()
            .find(|r| (r.tribe_a.min(r.tribe_b), r.tribe_a.max(r.tribe_b)) == key)
    }

    // Phase 5 — Knowledge trade between allied tribes
    fn process_trade(&mut self) {
        let trade_threshold = 30.0;

        let tribe_ids: Vec<u32> = self
            .tribes
            .iter()
            .filter(|t| !t.is_extinct)
            .map(|t| t.id)
            .collect();

        for ii in 0..tribe_ids.len() {
            for jj in (ii + 1)..tribe_ids.len() {
                let a = tribe_ids[ii];
                let b = tribe_ids[jj];

                let relation = self.get_diplomatic_relation(Some(a), Some(b));
                if relation.map(|r| r.score < trade_threshold).unwrap_or(false) {
                    continue;
                }

                let tribe_a = match self.tribes.iter().find(|t| t.id == a) {
                    Some(t) => t,
                    None => continue,
                };
                let tribe_b = match self.tribes.iter().find(|t| t.id == b) {
                    Some(t) => t,
                    None => continue,
                };

                let dc = tribe_a.territory_center.0 - tribe_b.territory_center.0;
                let dr = tribe_a.territory_center.1 - tribe_b.territory_center.1;
                let dist = ((dc * dc + dr * dr) as f32).sqrt();
                let trade_range = tribe_a.territory_radius + tribe_b.territory_radius;

                if dist < trade_range {
                    let trade_amount = (relation.unwrap().score * 0.1).max(1.0);
                    if let Some(tribe) = self.tribes.iter_mut().find(|t| t.id == a) {
                        tribe.knowledge += trade_amount * 0.5;
                    }
                    if let Some(tribe) = self.tribes.iter_mut().find(|t| t.id == b) {
                        tribe.knowledge += trade_amount * 0.5;
                    }
                }
            }
        }
    }

    // Phase 4 — Belief origination
    fn try_found_belief(&mut self, agent_idx: usize, trigger: &str) {
        let agent_id = self.agents[agent_idx].id;
        let has_belief = self.agents[agent_idx].belief_id.is_some();
        if has_belief {
            return;
        }

        let tenet_profile = crate::belief::TenetProfile {
            fatalism: if trigger == "disease" { 0.8 } else { 0.3 },
            ancestor_reverence: if trigger == "disease" || trigger == "mutation" {
                0.8
            } else {
                0.4
            },
            asceticism: if trigger == "mutation" { 0.7 } else { 0.3 },
        };

        let belief_name = format!(
            "{}'s Way",
            generate_lineage_name(self.tick_count * 1000 + self.next_belief_id as u64)
        );
        let belief = Belief::new(
            self.next_belief_id,
            None,
            belief_name.clone(),
            self.tick_count,
            Some(agent_id),
            tenet_profile,
        );
        self.beliefs.push(belief);
        let belief_id = self.next_belief_id;
        self.next_belief_id += 1;
        self.agents[agent_idx].belief_id = Some(belief_id);

        self.chronicle.push(format!(
            "Belief '{}' founded by agent #{} after {}",
            belief_name, agent_id, trigger
        ));
    }

    // Phase 4 — Belief origination
    fn check_belief_origination(&mut self) {
        for i in 0..self.agents.len() {
            let agent = &self.agents[i];
            if agent.belief_id.is_some() {
                continue;
            }
            let mut trigger = None;
            if agent.disease.immune && agent.disease.ticks_infected > 30 {
                trigger = Some("disease");
            } else if agent.large_mutation && agent.age > 200 {
                trigger = Some("mutation");
            } else if agent.tribe_id.is_some() && agent.age > 300 && rand_f32() < 0.1 {
                trigger = Some("tribe_founding");
            }

            if let Some(trigger) = trigger {
                if rand_f32() < 0.15 {
                    self.try_found_belief(i, trigger);
                    break;
                }
            }
        }
    }

    // Phase 4 — Tribe conflicts and skirmishes
    fn tick_tribe_conflicts(&mut self) {
        let min_relation = -30.0;
        let skirmish_size = 3;

        let active_tribes: Vec<u32> = self
            .tribes
            .iter()
            .filter(|t| !t.is_extinct && t.member_ids.len() >= skirmish_size)
            .map(|t| t.id)
            .collect();

        for ii in 0..active_tribes.len() {
            for jj in (ii + 1)..active_tribes.len() {
                let a_id = active_tribes[ii];
                let b_id = active_tribes[jj];

                let relation = self.get_diplomatic_relation(Some(a_id), Some(b_id));
                if relation.map(|r| r.score < min_relation).unwrap_or(false) {
                    let tribe_a = match self.tribes.iter().find(|t| t.id == a_id) {
                        Some(t) => t,
                        None => continue,
                    };
                    let tribe_b = match self.tribes.iter().find(|t| t.id == b_id) {
                        Some(t) => t,
                        None => continue,
                    };

                    let dc = tribe_a.territory_center.0 - tribe_b.territory_center.0;
                    let dr = tribe_a.territory_center.1 - tribe_b.territory_center.1;
                    let dist = ((dc * dc + dr * dr) as f32).sqrt();
                    let overlap_range = tribe_a.territory_radius + tribe_b.territory_radius;

                    if dist < overlap_range {
                        self.resolve_skirmish(a_id, b_id, skirmish_size);
                    }
                }
            }
        }
    }

    fn resolve_skirmish(&mut self, tribe_a_id: u32, tribe_b_id: u32, size: usize) {
        let mut team_a: Vec<usize> = Vec::new();
        let mut team_b: Vec<usize> = Vec::new();

        for (i, agent) in self.agents.iter().enumerate() {
            if team_a.len() >= size && team_b.len() >= size {
                break;
            }
            if let Some(tid) = agent.tribe_id {
                if tid == tribe_a_id && team_a.len() < size {
                    team_a.push(i);
                } else if tid == tribe_b_id && team_b.len() < size {
                    team_b.push(i);
                }
            }
        }

        if team_a.is_empty() || team_b.is_empty() {
            return;
        }

        let mut score_a = 0.0;
        for &i in &team_a {
            let agent = &self.agents[i];
            let mut s = agent.genome.strength * 0.7 + agent.genome.aggression * 0.3;
            if let Some(tid) = agent.tribe_id {
                if let Some(tribe) = self.tribes.iter().find(|t| t.id == tid) {
                    if tribe.culture_profile.warlike > 0.6 {
                        s *= 1.05;
                    }
                }
            }
            score_a += s;
        }

        let mut score_b = 0.0;
        for &i in &team_b {
            let agent = &self.agents[i];
            let mut s = agent.genome.strength * 0.7 + agent.genome.aggression * 0.3;
            if let Some(tid) = agent.tribe_id {
                if let Some(tribe) = self.tribes.iter().find(|t| t.id == tid) {
                    if tribe.culture_profile.warlike > 0.6 {
                        s *= 1.05;
                    }
                }
            }
            score_b += s;
        }

        let a_wins = score_a >= score_b;

        let mut to_remove: Vec<usize> = Vec::new();
        if a_wins {
            for &i in &team_b {
                if rand_f32() < 0.35 {
                    to_remove.push(i);
                }
            }
            for &i in &team_a {
                self.agents[i].energy =
                    (self.agents[i].energy + 15.0).min(self.agents[i].max_energy);
            }
        } else {
            for &i in &team_a {
                if rand_f32() < 0.35 {
                    to_remove.push(i);
                }
            }
            for &i in &team_b {
                self.agents[i].energy =
                    (self.agents[i].energy + 15.0).min(self.agents[i].max_energy);
            }
        }

        to_remove.sort();
        to_remove.reverse();
        for i in &to_remove {
            self.agents.remove(*i);
            self.total_deaths += 1;
        }

        let rel = self.get_or_create_relation(tribe_a_id, tribe_b_id);
        rel.score = (rel.score + if a_wins { -5.0 } else { 5.0 }).clamp(-100.0, 100.0);
    }

    pub fn get_tribe(&self, tribe_id: u32) -> Option<&Tribe> {
        self.tribes.iter().find(|t| t.id == tribe_id)
    }

    pub fn get_belief(&self, belief_id: u32) -> Option<&Belief> {
        self.beliefs.iter().find(|b| b.id == belief_id)
    }

    // Phase 6 — Civilization detection and divine influence
    pub fn update_civilizations(&mut self, world: &World) {
        // Check for tribes that should become civilizations
        for tribe in &self.tribes {
            if tribe.is_extinct {
                continue;
            }
            let member_count = tribe.member_ids.len();
            let already_civ = self.civilizations.iter().any(|c| {
                c.identity.name == tribe.name || c.history.iter().any(|h| h.contains(&tribe.name))
            });

            if member_count >= 30
                && tribe.knowledge >= 100.0
                && !already_civ
                && self.civilizations.len() < 5
            {
                let primary_trait = match tribe.culture_profile.communal {
                    c if c > 0.7 => "Agricultural".to_string(),
                    _ => {
                        let has_water = tribe.member_ids.iter().any(|id| {
                            self.agents.iter().find(|a| a.id == *id).map_or(false, |a| {
                                world
                                    .get_tile(a.col, a.row)
                                    .map_or(false, |t| t.biome == crate::world::Biome::Swamp)
                            })
                        });
                        if has_water {
                            "River".to_string()
                        } else {
                            "Mountain".to_string()
                        }
                    }
                };

                let mut civ = crate::civilization::Civilization::new(
                    tribe.id,
                    tribe.name.clone(),
                    primary_trait,
                    self.tick_count,
                );

                civ.ideology.authority = tribe.culture_profile.communal;
                civ.ideology.equality = tribe.culture_profile.communal;
                civ.ideology.tradition = tribe.culture_profile.traditional;
                civ.ideology.spirituality = 0.5;
                civ.ideology.militarism = tribe.culture_profile.warlike;
                civ.ideology.individualism = 1.0 - tribe.culture_profile.communal;

                civ.government_type = civ.ideology.government_type().to_string();

                // Transfer tribe knowledge to civilization
                let _tech_count = tribe.unlocked_tech.len();
                for (i, &_tech_id) in tribe.unlocked_tech.iter().enumerate() {
                    if i < civ.tech_tree.len() {
                        civ.tech_tree[i].unlocked = true;
                        civ.tech_tree[i].progress = 100.0;
                    }
                }

                self.civilizations.push(civ);
                self.divine_influence += 100.0;

                self.chronicle.push(format!(
                    "The {} tribe has evolved into a civilization!",
                    tribe.name
                ));
            }
        }

        // Earn divine influence from milestones
        for civ in &mut self.civilizations {
            let pop = civ.total_population();
            let prev_earned = civ.total_influence_earned;

            // Phase 6 — Tech progress over time based on population/knowledge
            if civ.current_era != "Information Era" {
                let research_rate = (pop as f32 / 1000.0).max(0.1);
                for tech in &mut civ.tech_tree {
                    if !tech.unlocked && tech.progress < 100.0 {
                        tech.progress += research_rate;
                        if tech.progress >= 100.0 {
                            tech.unlocked = true;
                            self.chronicle.push(format!(
                                "[{}] Technology researched: {}",
                                civ.identity.name, tech.name
                            ));
                        }
                    }
                }
                // Auto-advance era when enough techs unlocked
                let unlocked_count = civ.tech_tree.iter().filter(|t| t.unlocked).count();
                if unlocked_count >= 5 && civ.current_era == "Stone Age" {
                    civ.current_era = "Agricultural Era".to_string();
                    self.chronicle.push(format!(
                        "[{}] Advanced to Agricultural Era",
                        civ.identity.name
                    ));
                } else if unlocked_count >= 10 && civ.current_era == "Agricultural Era" {
                    civ.current_era = "Knowledge Era".to_string();
                    self.chronicle
                        .push(format!("[{}] Advanced to Knowledge Era", civ.identity.name));
                } else if unlocked_count >= 15 && civ.current_era == "Knowledge Era" {
                    civ.current_era = "Industrial Era".to_string();
                    self.chronicle.push(format!(
                        "[{}] Advanced to Industrial Era",
                        civ.identity.name
                    ));
                } else if unlocked_count >= 18 && civ.current_era == "Industrial Era" {
                    civ.current_era = "Information Era".to_string();
                    self.chronicle.push(format!(
                        "[{}] Advanced to Information Era",
                        civ.identity.name
                    ));
                }
            }

            // Phase 6 — City founding from large tribe populations
            if civ.cities.len() < 10 && pop >= 200 && civ.current_era != "Stone Age" {
                if let Some(center_tribe) = self
                    .tribes
                    .iter()
                    .find(|t| !t.is_extinct && t.member_ids.len() >= 20)
                {
                    let new_city_name =
                        format!("{} City {}", civ.identity.name, civ.cities.len() + 1);
                    civ.add_city(
                        new_city_name,
                        center_tribe.territory_center.0,
                        center_tribe.territory_center.1,
                        50,
                        self.tick_count,
                    );
                    self.chronicle.push(format!(
                        "[{}] Founded new city: {}",
                        civ.identity.name,
                        civ.cities.last().unwrap().name
                    ));
                }
            }

            // Population milestones
            let pop_thresholds = vec![100.0, 500.0, 1000.0, 5000.0, 10000.0];
            for &threshold in &pop_thresholds {
                if pop as f32 >= threshold && prev_earned < threshold {
                    civ.earn_influence(25.0, &format!("Population reached {}", threshold));
                    self.divine_influence += 25.0;
                }
            }

            // Tech milestones
            let tech_count = civ.tech_tree.iter().filter(|t| t.unlocked).count();
            if tech_count >= 5 && prev_earned < 200.0 {
                civ.earn_influence(30.0, "Multiple technologies discovered");
                self.divine_influence += 30.0;
            }

            // Era advancement
            if civ.current_era != "Stone Age" && prev_earned < 150.0 {
                civ.earn_influence(20.0, &format!("Entered the {}", civ.current_era));
                self.divine_influence += 20.0;
            }

            // Stability bonus
            if civ.stability > 80.0 && self.tick_count % 100 == 0 {
                civ.earn_influence(5.0, "Stable society");
                self.divine_influence += 5.0;
            }
        }
    }

    // Phase 6 — Apply civilization effects to agents
    pub fn get_civilization_bonus(&self, _col: i32, _row: i32) -> crate::civilization::CivBonus {
        let mut bonus = crate::civilization::CivBonus::default();
        for civ in &self.civilizations {
            let city_count = civ.cities.len() as f32;
            if city_count == 0.0 {
                continue;
            }

            // Per-city bonuses scale with population
            let city_pop_factor = civ.total_population() as f32 / 1000.0;
            bonus.regen_bonus += 0.02 * city_count + city_pop_factor * 0.01;
            bonus.food_bonus += 0.015 * city_count;

            // Government bonuses
            match civ.government_type.as_str() {
                "Monarchy" => {
                    bonus.stability_bonus += 0.1;
                    bonus.regen_bonus += 0.01;
                }
                "Republic" => {
                    bonus.food_bonus += 0.02;
                    bonus.birth_rate_bonus += 0.05;
                }
                "Military Dictatorship" => {
                    bonus.stability_bonus -= 0.05;
                    bonus.speed_bonus += 0.03;
                }
                _ => {}
            }

            // Active focus bonuses
            let focus_bonus = civ.active_focus_bonuses();
            bonus.regen_bonus += focus_bonus.regen_bonus;
            bonus.food_bonus += focus_bonus.food_bonus;
            bonus.birth_rate_bonus += focus_bonus.birth_rate_bonus;
            bonus.disease_resistance += focus_bonus.disease_resistance;
            bonus.speed_bonus += focus_bonus.speed_bonus;
            bonus.max_energy_bonus += focus_bonus.max_energy_bonus;
            bonus.research_rate_bonus += focus_bonus.research_rate_bonus;
            bonus.stability_bonus += focus_bonus.stability_bonus;

            // Tech-based bonuses
            if civ.has_tech(5) {
                bonus.food_bonus += 0.05; // Agriculture
            }
            if civ.has_tech(9) {
                bonus.regen_bonus += 0.03; // Permanent Cities
            }
            if civ.has_tech(13) {
                bonus.disease_resistance += 0.1; // Medicine
            }
            if civ.has_tech(15) {
                bonus.speed_bonus += 0.05; // Steam Power
            }
            if civ.has_tech(17) {
                bonus.max_energy_bonus += 10.0; // Electricity
            }

            // Identity trait bonuses
            let trait_bonus = civ.identity.get_modifier("Agriculture");
            bonus.food_bonus += trait_bonus * 0.5;
            let trade_bonus = civ.identity.get_modifier("Trade");
            bonus.food_bonus += trade_bonus * 0.3;
        }
        bonus
    }
}
