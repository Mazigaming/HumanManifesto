#[derive(Clone, Debug)]
pub struct DiplomaticRelation {
    pub tribe_a: u32,
    pub tribe_b: u32,
    pub score: f32,
    pub last_updated_tick: u64,
}

impl DiplomaticRelation {
    pub fn new(tribe_a: u32, tribe_b: u32, tick: u64) -> Self {
        DiplomaticRelation {
            tribe_a,
            tribe_b,
            score: 0.0,
            last_updated_tick: tick,
        }
    }

    pub fn other_tribe(&self, tribe_id: u32) -> u32 {
        if self.tribe_a == tribe_id {
            self.tribe_b
        } else {
            self.tribe_a
        }
    }

    pub fn behavior_band(&self) -> &'static str {
        if self.score < -50.0 {
            "Hostile"
        } else if self.score > 30.0 {
            "Allied"
        } else {
            "Neutral"
        }
    }
}
