use macroquad::prelude::*;

// 10 genome traits, each 0.0-1.0, each with trade-offs
#[derive(Clone, Copy, Debug)]
pub struct Genome {
    pub speed: f32,          // Movement speed, costs energy while moving
    pub strength: f32,       // Conflict win chance, costs baseline metabolism
    pub fertility: f32,      // Offspring count/cooldown, costs reproduction energy
    pub metabolism: f32,     // Low = survives on less food, but slower recovery
    pub aggression: f32,     // Wins conflicts, increases disease susceptibility
    pub sociability: f32,    // Group bonuses, mate-finding, but disease transmission
    pub camouflage: f32,     // Avoids detection, pure defense
    pub lifespan: f32,       // Max age, reduces fertility scaling at high values
    pub sight_range: f32,    // Vision radius, passive energy cost
    pub cold_tolerance: f32, // Survives cold biomes, costs in hot biomes
    pub heat_tolerance: f32, // Survives hot biomes, costs in cold biomes
}

impl Genome {
    pub fn random() -> Self {
        Genome {
            speed: rand_f32(),
            strength: rand_f32(),
            fertility: rand_f32(),
            metabolism: rand_f32(),
            aggression: rand_f32(),
            sociability: rand_f32(),
            camouflage: rand_f32(),
            lifespan: rand_f32(),
            sight_range: rand_f32(),
            cold_tolerance: rand_f32(),
            heat_tolerance: rand_f32(),
        }
    }

    pub fn blend(parent_a: &Genome, parent_b: &Genome, mutation_rate: f32) -> Self {
        let mut g = Genome {
            speed: (parent_a.speed + parent_b.speed) * 0.5,
            strength: (parent_a.strength + parent_b.strength) * 0.5,
            fertility: (parent_a.fertility + parent_b.fertility) * 0.5,
            metabolism: (parent_a.metabolism + parent_b.metabolism) * 0.5,
            aggression: (parent_a.aggression + parent_b.aggression) * 0.5,
            sociability: (parent_a.sociability + parent_b.sociability) * 0.5,
            camouflage: (parent_a.camouflage + parent_b.camouflage) * 0.5,
            lifespan: (parent_a.lifespan + parent_b.lifespan) * 0.5,
            sight_range: (parent_a.sight_range + parent_b.sight_range) * 0.5,
            cold_tolerance: (parent_a.cold_tolerance + parent_b.cold_tolerance) * 0.5,
            heat_tolerance: (parent_a.heat_tolerance + parent_b.heat_tolerance) * 0.5,
        };
        g.mutate(mutation_rate);
        g
    }

    pub fn mutate(&mut self, rate: f32) {
        // Small mutations
        Self::mutate_trait(&mut self.speed, rate);
        Self::mutate_trait(&mut self.strength, rate);
        Self::mutate_trait(&mut self.fertility, rate);
        Self::mutate_trait(&mut self.metabolism, rate);
        Self::mutate_trait(&mut self.aggression, rate);
        Self::mutate_trait(&mut self.sociability, rate);
        Self::mutate_trait(&mut self.camouflage, rate);
        Self::mutate_trait(&mut self.lifespan, rate);
        Self::mutate_trait(&mut self.sight_range, rate);
        Self::mutate_trait(&mut self.cold_tolerance, rate);
        Self::mutate_trait(&mut self.heat_tolerance, rate);
    }

    fn mutate_trait(trait_val: &mut f32, rate: f32) {
        if rand_f32() < rate {
            // 5% chance of large mutation
            if rand_f32() < 0.05 {
                *trait_val += (rand_f32() - 0.5) * 0.4;
            } else {
                *trait_val += (rand_f32() - 0.5) * 0.08;
            }
            *trait_val = trait_val.clamp(0.0, 1.0);
        }
    }

    // Phenotype color derived from genome
    pub fn phenotype_color(&self) -> Color {
        let r = (self.aggression * 200.0 + 55.0) as u8;
        let g = (self.metabolism * 150.0 + 50.0) as u8;
        let b = (self.sociability * 200.0 + 55.0) as u8;
        Color::from_rgba(r, g, b, 255)
    }

    // Color blended with biome for camouflage
    pub fn camouflaged_color(&self, biome_color: Color) -> Color {
        let camo = self.camouflage;
        let base = self.phenotype_color();
        Color::from_rgba(
            (base.r as f32 * (1.0 - camo) + biome_color.r as f32 * camo) as u8,
            (base.g as f32 * (1.0 - camo) + biome_color.g as f32 * camo) as u8,
            (base.b as f32 * (1.0 - camo) + biome_color.b as f32 * camo) as u8,
            255,
        )
    }

    pub fn kinship(&self, other: &Genome) -> f32 {
        let diff = (self.speed - other.speed).abs()
            + (self.strength - other.strength).abs()
            + (self.fertility - other.fertility).abs()
            + (self.metabolism - other.metabolism).abs()
            + (self.aggression - other.aggression).abs()
            + (self.sociability - other.sociability).abs()
            + (self.camouflage - other.camouflage).abs()
            + (self.lifespan - other.lifespan).abs()
            + (self.sight_range - other.sight_range).abs()
            + (self.cold_tolerance - other.cold_tolerance).abs()
            + (self.heat_tolerance - other.heat_tolerance).abs();
        1.0 - diff / 10.0
    }
}

// Simple PRNG for deterministic-ish randomness
pub fn rand_f32() -> f32 {
    use std::time::SystemTime;
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as f32;
    // Simple hash
    let mut h = seed as u32;
    h ^= h << 13;
    h ^= h >> 7;
    h ^= h << 17;
    (h as f32) / (u32::MAX as f32)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviorState {
    Idle,
    Fleeing,
    SeekingFood,
    SeekingWater,
    Reproducing,
    Socializing,
    Wandering,
    Fighting,
    Infected,
}

#[derive(Clone, Copy, Debug)]
pub struct MemorySlot {
    pub col: i32,
    pub row: i32,
    pub value: f32, // Positive = good, negative = danger
    pub decay: f32, // Decreases over time
}

#[derive(Clone, Copy, Debug)]
pub struct DiseaseState {
    pub infected: bool,
    pub ticks_infected: u32,
    pub immune: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Agent {
    pub id: u64,
    pub lineage_id: u32,
    pub genome: Genome,
    pub energy: f32,
    pub max_energy: f32,
    pub hydration: f32,
    pub health: f32,
    pub age: u32,
    pub col: i32,
    pub row: i32,
    pub behavior: BehaviorState,
    pub repro_cooldown: u32,
    pub memory: [Option<MemorySlot>; 5],
    pub disease: DiseaseState,
    pub conflict_wins: u32,
    pub large_mutation: bool, // Flag for visual highlight
    pub highlight_timer: u32, // Countdown for golden mutation flash
}

impl Agent {
    pub fn new(id: u64, lineage_id: u32, col: i32, row: i32, genome: Genome) -> Self {
        let max_energy = 80.0 + genome.strength * 40.0;
        Agent {
            id,
            lineage_id,
            genome,
            energy: max_energy * 0.7,
            max_energy,
            hydration: 80.0,
            health: 100.0,
            age: 0,
            col,
            row,
            behavior: BehaviorState::Idle,
            repro_cooldown: 0,
            memory: [None; 5],
            disease: DiseaseState {
                infected: false,
                ticks_infected: 0,
                immune: false,
            },
            conflict_wins: 0,
            large_mutation: false,
            highlight_timer: 0,
        }
    }

    pub fn add_memory(&mut self, col: i32, row: i32, value: f32) {
        let slot = MemorySlot {
            col,
            row,
            value,
            decay: 1.0,
        };
        // Find empty slot or lowest value slot
        let mut target = 0;
        let mut min_val = f32::MAX;
        for i in 0..5 {
            match self.memory[i] {
                None => {
                    target = i;
                    break;
                }
                Some(m) if m.value < min_val => {
                    min_val = m.value;
                    target = i;
                }
                _ => {}
            }
        }
        self.memory[target] = Some(slot);
    }

    pub fn decay_memories(&mut self) {
        for i in 0..5 {
            if let Some(ref mut m) = self.memory[i] {
                m.decay -= 0.01;
                if m.decay <= 0.0 {
                    self.memory[i] = None;
                }
            }
        }
    }

    pub fn sight_radius(&self) -> f32 {
        2.0 + self.genome.sight_range * 6.0
    }
}
