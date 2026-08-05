#[derive(Clone, Debug)]
pub struct TenetProfile {
    pub fatalism: f32,           // 0=agency, 1=fatalism
    pub ancestor_reverence: f32, // 0=future_focus, 1=ancestor_reverence
    pub asceticism: f32,         // 0=materialism, 1=asceticism
}

impl TenetProfile {
    pub fn new() -> Self {
        TenetProfile {
            fatalism: 0.5,
            ancestor_reverence: 0.5,
            asceticism: 0.5,
        }
    }

    pub fn blend(a: &TenetProfile, b: &TenetProfile, mutation_rate: f32) -> Self {
        let mut profile = TenetProfile {
            fatalism: (a.fatalism + b.fatalism) * 0.5,
            ancestor_reverence: (a.ancestor_reverence + b.ancestor_reverence) * 0.5,
            asceticism: (a.asceticism + b.asceticism) * 0.5,
        };
        profile.mutate(mutation_rate);
        profile
    }

    pub fn mutate(&mut self, rate: f32) {
        self.fatalism = Self::clamp_mutate(self.fatalism, rate);
        self.ancestor_reverence = Self::clamp_mutate(self.ancestor_reverence, rate);
        self.asceticism = Self::clamp_mutate(self.asceticism, rate);
    }

    fn clamp_mutate(val: f32, rate: f32) -> f32 {
        if crate::agent::rand_f32() < rate {
            (val + (crate::agent::rand_f32() - 0.5) * 0.2).clamp(0.0, 1.0)
        } else {
            val
        }
    }

    pub fn distance(&self, other: &TenetProfile) -> f32 {
        ((self.fatalism - other.fatalism).powi(2)
            + (self.ancestor_reverence - other.ancestor_reverence).powi(2)
            + (self.asceticism - other.asceticism).powi(2))
        .sqrt()
            / 1.732
    }
}

#[derive(Clone, Debug)]
pub struct Belief {
    pub id: u32,
    pub parent_belief_id: Option<u32>,
    pub name: String,
    pub tenet_profile: TenetProfile,
    pub founding_tick: u64,
    pub founder_agent_id: Option<u64>,
    pub adherent_count_history: Vec<(u64, usize)>,
}

impl Belief {
    pub fn new(
        id: u32,
        parent_belief_id: Option<u32>,
        name: String,
        founding_tick: u64,
        founder_agent_id: Option<u64>,
        tenet_profile: TenetProfile,
    ) -> Self {
        Belief {
            id,
            parent_belief_id,
            name,
            tenet_profile,
            founding_tick,
            founder_agent_id,
            adherent_count_history: vec![(founding_tick, 0)],
        }
    }

    pub fn record_adherents(&mut self, tick: u64, count: usize) {
        self.adherent_count_history.push((tick, count));
    }
}
