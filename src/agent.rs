use macroquad::prelude::*;

// 17 genome traits, each 0.0-1.0, each with trade-offs
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
    pub sexuality: f32,      // 0.0=hetero, 0.5=bi, 1.0=homo
    pub intelligence: f32,   // Better memory, foraging, decision-making
    pub curiosity: f32,      // Exploration drive, invention potential
    pub conformity: f32,     // Social cohesion, tradition adherence
    pub creativity: f32,     // Innovation, problem solving
    pub leadership: f32,     // Group coordination, influence
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
            sexuality: rand_f32(),
            intelligence: rand_f32(),
            curiosity: rand_f32(),
            conformity: rand_f32(),
            creativity: rand_f32(),
            leadership: rand_f32(),
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
            sexuality: (parent_a.sexuality + parent_b.sexuality) * 0.5,
            intelligence: (parent_a.intelligence + parent_b.intelligence) * 0.5,
            curiosity: (parent_a.curiosity + parent_b.curiosity) * 0.5,
            conformity: (parent_a.conformity + parent_b.conformity) * 0.5,
            creativity: (parent_a.creativity + parent_b.creativity) * 0.5,
            leadership: (parent_a.leadership + parent_b.leadership) * 0.5,
        };
        g.mutate(mutation_rate);
        g
    }

    pub fn mutate(&mut self, rate: f32) {
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
        Self::mutate_trait(&mut self.sexuality, rate);
        Self::mutate_trait(&mut self.intelligence, rate);
        Self::mutate_trait(&mut self.curiosity, rate);
        Self::mutate_trait(&mut self.conformity, rate);
        Self::mutate_trait(&mut self.creativity, rate);
        Self::mutate_trait(&mut self.leadership, rate);
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
        let r = (self.aggression * 200.0 + 55.0 + self.sexuality * 30.0) as u8;
        let g = (self.metabolism * 150.0 + 50.0 + self.intelligence * 50.0) as u8;
        let b = (self.sociability * 200.0 + 55.0 + (1.0 - self.sexuality) * 30.0) as u8;
        Color::from_rgba(r.min(255), g.min(255), b.min(255), 255)
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
            + (self.heat_tolerance - other.heat_tolerance).abs()
            + (self.sexuality - other.sexuality).abs()
            + (self.intelligence - other.intelligence).abs()
            + (self.curiosity - other.curiosity).abs()
            + (self.conformity - other.conformity).abs()
            + (self.creativity - other.creativity).abs()
            + (self.leadership - other.leadership).abs();
        1.0 - diff / 17.0
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
pub enum Gender {
    Male,
    Female,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviorState {
    Idle,
    Fleeing,
    SeekingFood,
    SeekingWater,
    Reproducing,
    Pregnant,
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
    pub tribe_id: Option<u32>,
    pub belief_id: Option<u32>,
    pub gender: Gender,
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
    pub pregnancy_days: u32,
    pub pregnancy_father_genome: Option<Genome>,
    pub memory: [Option<MemorySlot>; 8],
    pub disease: DiseaseState,
    pub conflict_wins: u32,
    pub large_mutation: bool,
    pub highlight_timer: u32,
    pub exploration_dc: f32,
    pub exploration_dr: f32,
    pub exploration_ticks: u32,
    pub experience: f32,
}

impl Agent {
    pub fn new(id: u64, lineage_id: u32, col: i32, row: i32, genome: Genome) -> Self {
        let max_energy = 100.0 + genome.lifespan * 20.0;
        let gender = if rand_f32() < 0.5 {
            Gender::Male
        } else {
            Gender::Female
        };
        Agent {
            id,
            lineage_id,
            tribe_id: None,
            belief_id: None,
            gender,
            genome,
            energy: max_energy,
            max_energy,
            hydration: 100.0,
            health: 100.0,
            age: 0,
            col,
            row,
            behavior: BehaviorState::Idle,
            repro_cooldown: 0,
            pregnancy_days: 0,
            pregnancy_father_genome: None,
            memory: [None; 8],
            disease: DiseaseState {
                infected: false,
                ticks_infected: 0,
                immune: false,
            },
            conflict_wins: 0,
            large_mutation: false,
            highlight_timer: 0,
            exploration_dc: 0.0,
            exploration_dr: 0.0,
            exploration_ticks: 0,
            experience: 0.0,
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
        for i in 0..8 {
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
        let decay_rate = 0.005 + (1.0 - self.genome.intelligence) * 0.01;
        for i in 0..8 {
            if let Some(ref mut m) = self.memory[i] {
                m.decay -= decay_rate;
                if m.decay <= 0.0 {
                    self.memory[i] = None;
                }
            }
        }
    }

    pub fn sight_radius(&self) -> f32 {
        let base = 5.0 + self.genome.sight_range * 20.0;
        base * (0.8 + self.genome.intelligence * 0.4)
    }
}
