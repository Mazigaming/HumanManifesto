use crate::agent::Agent;
use macroquad::prelude::Color;

#[derive(Clone, Debug)]
pub struct CultureProfile {
    pub warlike: f32,
    pub communal: f32,
    pub traditional: f32,
    pub industrious: f32,
    pub expansionist: f32,
}

impl CultureProfile {
    pub fn new() -> Self {
        CultureProfile {
            warlike: 0.5,
            communal: 0.5,
            traditional: 0.5,
            industrious: 0.5,
            expansionist: 0.5,
        }
    }

    pub fn from_agents(agents: &[&Agent]) -> Self {
        if agents.is_empty() {
            return CultureProfile::new();
        }

        let n = agents.len() as f32;
        let mut avg_aggression = 0.0;
        let mut avg_sociability = 0.0;
        let mut avg_metabolism = 0.0;
        let mut avg_sight = 0.0;
        let mut avg_speed = 0.0;
        let mut genome_variance = 0.0;
        let centroid = crate::agent::Genome {
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

        for agent in agents {
            avg_aggression += agent.genome.aggression;
            avg_sociability += agent.genome.sociability;
            avg_metabolism += agent.genome.metabolism;
            avg_sight += agent.genome.sight_range;
            avg_speed += agent.genome.speed;
        }

        avg_aggression /= n;
        avg_sociability /= n;
        avg_metabolism /= n;
        avg_sight /= n;
        avg_speed /= n;

        for agent in agents {
            let diff = (agent.genome.speed - centroid.speed).abs()
                + (agent.genome.strength - centroid.strength).abs()
                + (agent.genome.fertility - centroid.fertility).abs()
                + (agent.genome.metabolism - centroid.metabolism).abs()
                + (agent.genome.aggression - centroid.aggression).abs()
                + (agent.genome.sociability - centroid.sociability).abs()
                + (agent.genome.camouflage - centroid.camouflage).abs()
                + (agent.genome.lifespan - centroid.lifespan).abs()
                + (agent.genome.sight_range - centroid.sight_range).abs()
                + (agent.genome.cold_tolerance - centroid.cold_tolerance).abs()
                + (agent.genome.heat_tolerance - centroid.heat_tolerance).abs()
                + (agent.genome.sexuality - centroid.sexuality).abs()
                + (agent.genome.intelligence - centroid.intelligence).abs()
                + (agent.genome.curiosity - centroid.curiosity).abs()
                + (agent.genome.conformity - centroid.conformity).abs()
                + (agent.genome.creativity - centroid.creativity).abs()
                + (agent.genome.leadership - centroid.leadership).abs();
            genome_variance += diff / 17.0;
        }
        genome_variance /= n;

        let traditional = (1.0 - genome_variance).clamp(0.0, 1.0);
        let industrious = (avg_metabolism * 0.6 + 0.4).clamp(0.0, 1.0);
        let expansionist = ((avg_sight + avg_speed) * 0.5).clamp(0.0, 1.0);

        CultureProfile {
            warlike: avg_aggression,
            communal: avg_sociability,
            traditional,
            industrious,
            expansionist,
        }
    }

    pub fn color(&self) -> Color {
        let r = (self.warlike * 200.0 + 55.0) as u8;
        let g = (self.communal * 150.0 + 50.0) as u8;
        let b = (self.traditional * 200.0 + 55.0) as u8;
        Color::from_rgba(r.min(255), g, b.min(255), 255)
    }

    pub fn compatibility(&self, other: &CultureProfile) -> f32 {
        let diff = (self.warlike - other.warlike).abs()
            + (self.communal - other.communal).abs()
            + (self.traditional - other.traditional).abs()
            + (self.industrious - other.industrious).abs()
            + (self.expansionist - other.expansionist).abs();
        1.0 - (diff / 5.0).min(1.0)
    }
}

#[derive(Clone, Debug)]
pub struct Tribe {
    pub id: u32,
    pub name: String,
    pub member_ids: Vec<u64>,
    pub culture_profile: CultureProfile,
    pub territory_center: (i32, i32),
    pub territory_radius: f32,
    pub leader_agent_id: Option<u64>,
    pub founding_tick: u64,
    pub population_history: Vec<(u64, usize)>,
    pub is_extinct: bool,
    pub knowledge: f32,
    pub unlocked_tech: Vec<u32>,
}

impl Tribe {
    pub fn new(id: u32, name: String, founding_tick: u64, initial_agent: &Agent) -> Self {
        let territory_center = (initial_agent.col, initial_agent.row);
        let culture_profile = CultureProfile::from_agents(&[initial_agent]);
        Tribe {
            id,
            name,
            member_ids: vec![initial_agent.id],
            culture_profile,
            territory_center,
            territory_radius: 20.0,
            leader_agent_id: Some(initial_agent.id),
            founding_tick,
            population_history: vec![(founding_tick, 1)],
            is_extinct: false,
            knowledge: 0.0,
            unlocked_tech: Vec::new(),
        }
    }

    pub fn member_count(&self) -> usize {
        self.member_ids.len()
    }

    pub fn record_population(&mut self, tick: u64) {
        self.population_history.push((tick, self.member_count()));
    }
}
