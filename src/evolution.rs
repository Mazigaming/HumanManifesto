use crate::agent::*;
use crate::world::*;

pub struct EvolutionSim {
    pub agents: Vec<Agent>,
    next_id: u64,
    pub tick_count: u64,
    pub disease_outbreaks: u32,
    pub total_births: u64,
    pub total_deaths: u64,
}

impl EvolutionSim {
    pub fn new() -> Self {
        EvolutionSim {
            agents: Vec::new(),
            next_id: 1,
            tick_count: 0,
            disease_outbreaks: 0,
            total_births: 0,
            total_deaths: 0,
        }
    }

    pub fn spawn_agent(&mut self, col: i32, row: i32, genome: Genome) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.agents.push(Agent::new(id, col, row, genome));
        id
    }

    pub fn spawn_population(&mut self, col: i32, row: i32, count: usize) {
        for _ in 0..count {
            let genome = Genome::random();
            self.spawn_agent(col, row, genome);
        }
    }

    pub fn tick(&mut self, world: &World) {
        self.tick_count += 1;

        // Update each agent
        let mut to_remove = Vec::new();
        let mut to_add = Vec::new();

        for i in 0..self.agents.len() {
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

            // Energy drain (metabolism + sight range passive cost)
            let energy_drain = 0.5 + agent.genome.metabolism * 0.3 + agent.genome.sight_range * 0.2;
            agent.energy -= energy_drain;

            // Hydration drain
            agent.hydration -= 0.3;

            // Environmental hazard damage (temperature mismatch)
            if let Some(tile) = world.get_tile(agent.col, agent.row) {
                let temp = tile.temperature;
                // Cold damage if low cold tolerance in cold biomes
                if temp < 0.3 && agent.genome.cold_tolerance < 0.5 {
                    let damage = (0.3 - temp) * (0.5 - agent.genome.cold_tolerance) * 2.0;
                    agent.health -= damage;
                }
                // Heat damage if low heat tolerance in hot biomes
                if temp > 0.7 && agent.genome.heat_tolerance < 0.5 {
                    let damage = (temp - 0.7) * (0.5 - agent.genome.heat_tolerance) * 2.0;
                    agent.health -= damage;
                }
            }

            // Disease progression
            if agent.disease.infected {
                agent.disease.ticks_infected += 1;
                agent.health -= 0.5;
                // Chance to recover or die
                if agent.disease.ticks_infected > 50 {
                    if rand_f32() < 0.02 {
                        agent.disease.infected = false;
                        agent.disease.immune = true;
                    }
                    if agent.disease.ticks_infected > 100 && rand_f32() < 0.01 {
                        agent.health = 0.0;
                    }
                }
            }

            // Starvation damage
            if agent.energy <= 0.0 {
                agent.health -= 2.0;
                agent.energy = 0.0;
            }

            // Dehydration damage
            if agent.hydration <= 0.0 {
                agent.health -= 3.0;
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
                to_remove.push(i);
                continue;
            }

            // Clamp stats
            agent.energy = agent.energy.min(agent.max_energy);
            agent.hydration = agent.hydration.min(100.0);
            agent.health = agent.health.min(100.0);

            // Decision loop
            self.decide_action(i, world);
        }

        // Process reproduction (add new agents after iteration)
        for (col, row, genome) in to_add {
            let mut child = Agent::new(self.next_id, col, row, genome);
            self.next_id += 1;
            // Check for large mutation
            if rand_f32() < 0.02 {
                child.large_mutation = true;
                child.highlight_timer = 120;
            }
            self.agents.push(child);
            self.total_births += 1;
        }

        // Remove dead agents (reverse order to preserve indices)
        to_remove.sort();
        to_remove.reverse();
        for i in to_remove {
            self.agents.remove(i);
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

        // 1. Flee danger
        if let Some((dc, dr)) = self.find_nearest_danger(agent_idx, agent_sight) {
            self.move_agent(agent_idx, -dc, -dr, world);
            self.agents[agent_idx].behavior = BehaviorState::Fleeing;
            return;
        }

        // 2. Seek food/water
        if agent_energy < agent_max_energy * 0.5 {
            if let Some((col, row)) = self.find_nearest_resource(agent_idx, agent_sight, world) {
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
            if let Some((col, row)) = self.find_nearest_water(agent_idx, agent_sight, world) {
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

        // 3. Reproduce
        if agent_energy > agent_max_energy * 0.7
            && agent_hydration > 60.0
            && agent_repro_cooldown == 0
        {
            if let Some(mate_idx) = self.find_mate(agent_idx, agent_sight) {
                self.reproduce(agent_idx, mate_idx);
                self.agents[agent_idx].behavior = BehaviorState::Reproducing;
                return;
            }
        }

        // 4. Socialize (move toward other agents if sociable)
        if agent_sociability > 0.6 {
            if let Some((dc, dr)) = self.find_nearest_agent(agent_idx, agent_sight) {
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

    fn find_nearest_danger(&self, agent_idx: usize, sight: f32) -> Option<(f32, f32)> {
        let agent = &self.agents[agent_idx];
        let mut nearest = None;
        let mut min_dist = sight;

        for i in 0..self.agents.len() {
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
        nearest
    }

    fn find_nearest_resource(
        &self,
        agent_idx: usize,
        sight: f32,
        world: &World,
    ) -> Option<(i32, i32)> {
        let agent = &self.agents[agent_idx];
        let sight_int = sight as i32;

        for dr in -sight_int..=sight_int {
            for dc in -sight_int..=sight_int {
                let col = agent.col + dc;
                let row = agent.row + dr;
                if let Some(tile) = world.get_tile(col, row) {
                    if tile.resource.is_some() {
                        return Some((col, row));
                    }
                }
            }
        }
        None
    }

    fn find_nearest_water(
        &self,
        agent_idx: usize,
        sight: f32,
        world: &World,
    ) -> Option<(i32, i32)> {
        let agent = &self.agents[agent_idx];
        let sight_int = sight as i32;

        for dr in -sight_int..=sight_int {
            for dc in -sight_int..=sight_int {
                let col = agent.col + dc;
                let row = agent.row + dr;
                if let Some(tile) = world.get_tile(col, row) {
                    if tile.is_river || tile.elevation < 0.3 {
                        return Some((col, row));
                    }
                }
            }
        }
        None
    }

    fn find_mate(&self, agent_idx: usize, sight: f32) -> Option<usize> {
        let agent = &self.agents[agent_idx];
        let mut best_mate = None;
        let mut best_score = -1.0;

        for i in 0..self.agents.len() {
            if i == agent_idx {
                continue;
            }
            let other = &self.agents[i];
            let dc = (other.col - agent.col) as f32;
            let dr = (other.row - agent.row) as f32;
            let dist = (dc * dc + dr * dr).sqrt();

            if dist < sight && other.repro_cooldown == 0 && other.energy > other.max_energy * 0.5 {
                // Compatibility score (avoid extreme aggression/sociability mismatch)
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
        best_mate
    }

    fn find_nearest_agent(&self, agent_idx: usize, sight: f32) -> Option<(f32, f32)> {
        let agent = &self.agents[agent_idx];
        let mut nearest = None;
        let mut min_dist = sight;

        for i in 0..self.agents.len() {
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
        nearest
    }

    fn move_agent(&mut self, agent_idx: usize, dc: f32, dr: f32, world: &World) {
        let agent = &mut self.agents[agent_idx];
        let speed = 0.5 + agent.genome.speed * 1.5;

        let new_col = agent.col + (dc * speed).round() as i32;
        let new_row = agent.row + (dr * speed).round() as i32;

        // Clamp to world bounds
        let new_col = new_col.max(0).min(world.width - 1);
        let new_row = new_row.max(0).min(world.height - 1);

        // Movement costs energy
        agent.energy -= agent.genome.speed * 0.3;

        agent.col = new_col;
        agent.row = new_row;
    }

    fn eat_resource(&mut self, agent_idx: usize, col: i32, row: i32, world: &World) {
        // Note: In a full implementation, this would modify the world's resource nodes
        // For now, just restore energy
        let agent = &mut self.agents[agent_idx];
        agent.energy += 30.0;
        agent.add_memory(col, row, 1.0);
    }

    fn reproduce(&mut self, agent_idx: usize, mate_idx: usize) {
        let mutation_rate = 0.1;
        let mate_genome;
        let agent_genome_copy;
        let agent_fertility;
        let col;
        let row;
        {
            let agent = &self.agents[agent_idx];
            let mate = &self.agents[mate_idx];
            mate_genome = mate.genome;
            agent_genome_copy = agent.genome;
            agent_fertility = agent.genome.fertility;
            col = (agent.col + mate.col) / 2;
            row = (agent.row + mate.row) / 2;
        }

        // Twin birth chance based on fertility
        let twin_count = if rand_f32() < agent_fertility * 0.1 {
            2
        } else {
            1
        };

        for _ in 0..twin_count {
            let child_genome = Genome::blend(&agent_genome_copy, &mate_genome, mutation_rate);
            let mut child = Agent::new(self.next_id, col, row, child_genome);
            if rand_f32() < 0.02 {
                child.large_mutation = true;
                child.highlight_timer = 120;
            }
            self.agents.push(child);
            self.next_id += 1;
            self.total_births += 1;
        }

        self.agents[agent_idx].repro_cooldown = 100 - (agent_fertility * 50.0) as u32;
        self.agents[agent_idx].energy -= 20.0;
    }
}
