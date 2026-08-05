use crate::agent::Genome;
use macroquad::prelude::Color;

#[derive(Clone, Debug)]
pub struct Lineage {
    pub id: u32,
    pub parent_lineage_id: Option<u32>,
    pub name: String,
    pub founding_tick: u64,
    pub centroid_genome: Genome,
    pub dominant_color: Color,
    pub population_history: Vec<(u64, usize)>,
    pub member_count: usize,
    pub is_extinct: bool,
}

impl Lineage {
    pub fn new(
        id: u32,
        parent_id: Option<u32>,
        name: String,
        founding_tick: u64,
        initial_genome: Genome,
    ) -> Self {
        let dominant_color = initial_genome.phenotype_color();
        Lineage {
            id,
            parent_lineage_id: parent_id,
            name,
            founding_tick,
            centroid_genome: initial_genome,
            dominant_color,
            population_history: Vec::new(),
            member_count: 0,
            is_extinct: false,
        }
    }

    pub fn update_centroid(&mut self, genomes: &[Genome]) {
        if genomes.is_empty() {
            self.member_count = 0;
            self.is_extinct = true;
            return;
        }

        self.member_count = genomes.len();
        self.is_extinct = false;

        // Compute average genome
        let count = genomes.len() as f32;
        let mut avg = Genome {
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

        for genome in genomes {
            avg.speed += genome.speed;
            avg.strength += genome.strength;
            avg.fertility += genome.fertility;
            avg.metabolism += genome.metabolism;
            avg.aggression += genome.aggression;
            avg.sociability += genome.sociability;
            avg.camouflage += genome.camouflage;
            avg.lifespan += genome.lifespan;
            avg.sight_range += genome.sight_range;
            avg.cold_tolerance += genome.cold_tolerance;
            avg.heat_tolerance += genome.heat_tolerance;
            avg.sexuality += genome.sexuality;
            avg.intelligence += genome.intelligence;
            avg.curiosity += genome.curiosity;
            avg.conformity += genome.conformity;
            avg.creativity += genome.creativity;
            avg.leadership += genome.leadership;
        }

        avg.speed /= count;
        avg.strength /= count;
        avg.fertility /= count;
        avg.metabolism /= count;
        avg.aggression /= count;
        avg.sociability /= count;
        avg.camouflage /= count;
        avg.lifespan /= count;
        avg.sight_range /= count;
        avg.cold_tolerance /= count;
        avg.heat_tolerance /= count;
        avg.sexuality /= count;
        avg.intelligence /= count;
        avg.curiosity /= count;
        avg.conformity /= count;
        avg.creativity /= count;
        avg.leadership /= count;

        self.centroid_genome = avg;
        self.dominant_color = avg.phenotype_color();
    }

    pub fn record_population(&mut self, tick: u64) {
        self.population_history.push((tick, self.member_count));
    }

    pub fn genetic_distance(&self, genome: &Genome) -> f32 {
        let c = &self.centroid_genome;
        let dist = ((genome.speed - c.speed).powi(2)
            + (genome.strength - c.strength).powi(2)
            + (genome.fertility - c.fertility).powi(2)
            + (genome.metabolism - c.metabolism).powi(2)
            + (genome.aggression - c.aggression).powi(2)
            + (genome.sociability - c.sociability).powi(2)
            + (genome.camouflage - c.camouflage).powi(2)
            + (genome.lifespan - c.lifespan).powi(2)
            + (genome.sight_range - c.sight_range).powi(2)
            + (genome.cold_tolerance - c.cold_tolerance).powi(2)
            + (genome.heat_tolerance - c.heat_tolerance).powi(2)
            + (genome.sexuality - c.sexuality).powi(2)
            + (genome.intelligence - c.intelligence).powi(2)
            + (genome.curiosity - c.curiosity).powi(2)
            + (genome.conformity - c.conformity).powi(2)
            + (genome.creativity - c.creativity).powi(2)
            + (genome.leadership - c.leadership).powi(2))
        .sqrt();
        dist / 4.0 // Normalize: sqrt(17) max distance
    }
}

// Simple name generator for lineages
const SYLLABLES: &[&str] = &[
    "ka", "to", "ri", "ven", "thal", "mor", "gan", "del", "sar", "quin", "bor", "ath", "wen",
    "dor", "lith", "fen", "gar", "hel", "is", "jar", "kel", "lor", "mir", "nor", "oth", "pen",
    "ral", "sol", "tar", "ul", "var", "wyn", "xen", "yr", "zor",
];

pub fn generate_lineage_name(seed: u64) -> String {
    let mut h = seed;
    let mut name = String::new();

    // 2-3 syllables
    let syllable_count = 2 + (h % 2) as usize;
    h = h
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);

    for i in 0..syllable_count {
        h = h
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let idx = (h >> 33) as usize % SYLLABLES.len();
        name.push_str(SYLLABLES[idx]);
    }

    // Capitalize first letter
    let mut chars = name.chars();
    if let Some(c) = chars.next() {
        name = c.to_uppercase().collect::<String>() + chars.as_str();
    }

    name
}
