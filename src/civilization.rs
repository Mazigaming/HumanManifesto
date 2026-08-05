use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct IdeologyValues {
    pub authority: f32,
    pub equality: f32,
    pub tradition: f32,
    pub spirituality: f32,
    pub militarism: f32,
    pub individualism: f32,
}

impl IdeologyValues {
    pub fn new() -> Self {
        IdeologyValues {
            authority: 0.5,
            equality: 0.5,
            tradition: 0.5,
            spirituality: 0.5,
            militarism: 0.5,
            individualism: 0.5,
        }
    }

    pub fn blend(a: &IdeologyValues, b: &IdeologyValues, weight: f32) -> Self {
        IdeologyValues {
            authority: a.authority * (1.0 - weight) + b.authority * weight,
            equality: a.equality * (1.0 - weight) + b.equality * weight,
            tradition: a.tradition * (1.0 - weight) + b.tradition * weight,
            spirituality: a.spirituality * (1.0 - weight) + b.spirituality * weight,
            militarism: a.militarism * (1.0 - weight) + b.militarism * weight,
            individualism: a.individualism * (1.0 - weight) + b.individualism * weight,
        }
    }

    pub fn government_type(&self) -> &'static str {
        if self.authority > 0.8 && self.militarism > 0.7 {
            "Military Dictatorship"
        } else if self.authority > 0.8 && self.spirituality > 0.7 {
            "Theocracy"
        } else if self.authority > 0.75 && self.tradition > 0.7 {
            "Monarchy"
        } else if self.authority > 0.7 && self.equality < 0.3 {
            "Feudal Hierarchy"
        } else if self.authority < 0.25 && self.equality > 0.75 {
            "Communal Council"
        } else if self.authority < 0.3 && self.individualism > 0.7 {
            "Anarcho-Individualist"
        } else if self.authority < 0.35 && self.tradition < 0.3 {
            "Tribal Council"
        } else if self.authority < 0.4 && self.equality > 0.6 {
            "Republic"
        } else if self.equality > 0.7 && self.spirituality > 0.6 {
            "Clerical Republic"
        } else if self.militarism > 0.7 && self.individualism > 0.6 {
            "Mercenary State"
        } else if self.spirituality > 0.75 {
            "Religious State"
        } else if self.individualism > 0.7 && self.equality > 0.5 {
            "Liberal Democracy"
        } else if self.equality > 0.65 {
            "Social Democracy"
        } else if self.tradition > 0.7 {
            "Conservative Republic"
        } else {
            "Chiefdom"
        }
    }
}

#[derive(Clone, Debug)]
pub struct CivilizationIdentity {
    pub name: String,
    pub primary_trait: String,
    pub modifiers: HashMap<String, f32>,
}

impl CivilizationIdentity {
    pub fn new(name: String, primary_trait: String) -> Self {
        let mut modifiers = HashMap::new();
        match primary_trait.as_str() {
            "Agricultural" => {
                modifiers.insert("Agriculture".to_string(), 0.4);
                modifiers.insert("Population Growth".to_string(), 0.2);
                modifiers.insert("Trade".to_string(), -0.1);
            }
            "Mountain" => {
                modifiers.insert("Engineering".to_string(), 0.2);
                modifiers.insert("Defense".to_string(), 0.3);
                modifiers.insert("Trade".to_string(), -0.2);
                modifiers.insert("Agriculture".to_string(), -0.2);
            }
            "River" => {
                modifiers.insert("Agriculture".to_string(), 0.4);
                modifiers.insert("Population Growth".to_string(), 0.2);
                modifiers.insert("Trade".to_string(), 0.15);
            }
            "Coastal" => {
                modifiers.insert("Trade".to_string(), 0.4);
                modifiers.insert("Naval".to_string(), 0.3);
                modifiers.insert("Agriculture".to_string(), -0.1);
            }
            "Forest" => {
                modifiers.insert("Hunting".to_string(), 0.3);
                modifiers.insert("Nature Magic".to_string(), 0.2);
                modifiers.insert("Agriculture".to_string(), -0.1);
            }
            "Desert" => {
                modifiers.insert("Engineering".to_string(), 0.15);
                modifiers.insert("Defense".to_string(), 0.2);
                modifiers.insert("Agriculture".to_string(), -0.3);
            }
            _ => {
                modifiers.insert("Adaptability".to_string(), 0.2);
            }
        }
        CivilizationIdentity {
            name,
            primary_trait,
            modifiers,
        }
    }

    pub fn get_modifier(&self, key: &str) -> f32 {
        *self.modifiers.get(key).unwrap_or(&0.0)
    }
}

#[derive(Clone, Debug)]
pub enum FocusRequirement {
    Population(u32),
    TechUnlocked(u32),
    GovernmentType(String),
    BeliefAdopted(u32),
    TribeKnowledge(f32),
    CityCount(u32),
    EraReached(String),
}

#[derive(Clone, Debug)]
pub enum FocusEffect {
    UnlockFocus(u32),
    ModifyStat(String, f32),
    UnlockTech(u32),
    ChangeGovernment(String),
    SpawnEvent(String),
    GrantInfluence(f32),
    UnlockEra(String),
}

#[derive(Clone, Debug)]
pub struct FocusNode {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub era: String,
    pub cost: f32,
    pub requirements: Vec<FocusRequirement>,
    pub effects: Vec<FocusEffect>,
    pub unlocked_ids: Vec<u32>,
    pub category: String,
}

impl FocusNode {
    pub fn is_available(&self, civ: &Civilization) -> bool {
        for req in &self.requirements {
            match req {
                FocusRequirement::Population(min) => {
                    if civ.total_population() < *min {
                        return false;
                    }
                }
                FocusRequirement::TechUnlocked(id) => {
                    if !civ.has_tech(*id) {
                        return false;
                    }
                }
                FocusRequirement::GovernmentType(gov) => {
                    if civ.government_type != *gov {
                        return false;
                    }
                }
                FocusRequirement::BeliefAdopted(bid) => {
                    if !civ.adopted_beliefs.contains(bid) {
                        return false;
                    }
                }
                FocusRequirement::TribeKnowledge(min_knowledge) => {
                    if civ.total_knowledge() < *min_knowledge {
                        return false;
                    }
                }
                FocusRequirement::CityCount(min) => {
                    if civ.cities.len() < (*min) as usize {
                        return false;
                    }
                }
                FocusRequirement::EraReached(era) => {
                    if civ.current_era != *era {
                        return false;
                    }
                }
            }
        }
        true
    }
}

#[derive(Clone, Debug)]
pub struct TechNode {
    pub id: u32,
    pub name: String,
    pub era: String,
    pub unlocked: bool,
    pub progress: f32,
    pub required_focus: Option<u32>,
    pub description: String,
}

impl TechNode {
    pub fn new(
        id: u32,
        name: &str,
        era: &str,
        description: &str,
        required_focus: Option<u32>,
    ) -> Self {
        TechNode {
            id,
            name: name.to_string(),
            era: era.to_string(),
            unlocked: false,
            progress: 0.0,
            required_focus,
            description: description.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct City {
    pub name: String,
    pub col: i32,
    pub row: i32,
    pub population: u32,
    pub founded_tick: u64,
}

#[derive(Clone, Debug)]
pub struct Civilization {
    pub id: u32,
    pub identity: CivilizationIdentity,
    pub ideology: IdeologyValues,
    pub divine_influence: f32,
    pub unlocked_focuses: Vec<u32>,
    pub active_focuses: Vec<u32>,
    pub adopted_beliefs: Vec<u32>,
    pub tech_tree: Vec<TechNode>,
    pub government_type: String,
    pub laws: HashMap<String, String>,
    pub stability: f32,
    pub cities: Vec<City>,
    pub founded_tick: u64,
    pub history: Vec<String>,
    pub current_era: String,
    pub total_influence_earned: f32,
    pub events_completed: Vec<u32>,
}

impl Civilization {
    pub fn new(id: u32, name: String, primary_trait: String, founding_tick: u64) -> Self {
        let identity = CivilizationIdentity::new(name.clone(), primary_trait);
        let government_type = "Tribal Council".to_string();
        let mut laws = HashMap::new();
        laws.insert("Inheritance".to_string(), "Shared".to_string());
        laws.insert("Land".to_string(), "Communal".to_string());
        laws.insert("Military".to_string(), "Militia".to_string());

        Civilization {
            id,
            identity,
            ideology: IdeologyValues::new(),
            divine_influence: 50.0,
            unlocked_focuses: vec![1], // Start with "Tribal Society"
            active_focuses: Vec::new(),
            adopted_beliefs: Vec::new(),
            tech_tree: Self::initial_tech_tree(),
            government_type,
            laws,
            stability: 50.0,
            cities: Vec::new(),
            founded_tick: founding_tick,
            history: vec![format!("The {} civilization is founded", name)],
            current_era: "Stone Age".to_string(),
            total_influence_earned: 50.0,
            events_completed: Vec::new(),
        }
    }

    fn initial_tech_tree() -> Vec<TechNode> {
        vec![
            TechNode::new(
                1,
                "Fire",
                "Stone Age",
                "Mastering fire for warmth and cooking",
                None,
            ),
            TechNode::new(2, "Tools", "Stone Age", "Crafting stone tools", Some(1)),
            TechNode::new(
                3,
                "Weapons",
                "Stone Age",
                "Hunting and defense tools",
                Some(2),
            ),
            TechNode::new(
                4,
                "Hunting Techniques",
                "Stone Age",
                "Coordinated hunting strategies",
                Some(3),
            ),
            TechNode::new(
                5,
                "Agriculture",
                "Agricultural Era",
                "Farming and crop domestication",
                Some(4),
            ),
            TechNode::new(
                6,
                "Irrigation",
                "Agricultural Era",
                "Water management for crops",
                Some(5),
            ),
            TechNode::new(
                7,
                "Animal Domestication",
                "Agricultural Era",
                "Taming animals for labor and food",
                Some(5),
            ),
            TechNode::new(
                8,
                "Food Storage",
                "Agricultural Era",
                "Preserving harvests",
                Some(5),
            ),
            TechNode::new(
                9,
                "Permanent Cities",
                "Agricultural Era",
                "Building lasting settlements",
                Some(6),
            ),
            TechNode::new(
                10,
                "Writing",
                "Knowledge Era",
                "Recording information",
                Some(9),
            ),
            TechNode::new(
                11,
                "Mathematics",
                "Knowledge Era",
                "Abstract reasoning and numbers",
                Some(10),
            ),
            TechNode::new(
                12,
                "Astronomy",
                "Knowledge Era",
                "Studying the stars",
                Some(10),
            ),
            TechNode::new(
                13,
                "Medicine",
                "Knowledge Era",
                "Healing and disease understanding",
                Some(10),
            ),
            TechNode::new(
                14,
                "Philosophy",
                "Knowledge Era",
                "Questioning existence and ethics",
                Some(10),
            ),
            TechNode::new(
                15,
                "Steam Power",
                "Industrial Era",
                "Harnessing steam for work",
                Some(11),
            ),
            TechNode::new(
                16,
                "Factories",
                "Industrial Era",
                "Mass production",
                Some(15),
            ),
            TechNode::new(
                17,
                "Electricity",
                "Industrial Era",
                "Harnessing electrical energy",
                Some(15),
            ),
            TechNode::new(
                18,
                "Computers",
                "Information Era",
                "Automated calculation",
                Some(16),
            ),
            TechNode::new(
                19,
                "Internet",
                "Information Era",
                "Global communication network",
                Some(18),
            ),
            TechNode::new(
                20,
                "AI",
                "Information Era",
                "Artificial intelligence",
                Some(18),
            ),
        ]
    }

    pub fn has_tech(&self, id: u32) -> bool {
        self.tech_tree.iter().any(|t| t.id == id && t.unlocked)
    }

    pub fn total_population(&self) -> u32 {
        self.cities.iter().map(|c| c.population).sum()
    }

    pub fn total_knowledge(&self) -> f32 {
        self.tech_tree.iter().filter(|t| t.unlocked).count() as f32 * 50.0
    }

    pub fn unlock_tech(&mut self, id: u32) {
        if let Some(tech) = self.tech_tree.iter_mut().find(|t| t.id == id) {
            if !tech.unlocked {
                tech.unlocked = true;
                tech.progress = 100.0;
                self.history
                    .push(format!("Technology unlocked: {}", tech.name));
            }
        }
    }

    pub fn advance_era(&mut self) {
        let eras = vec![
            "Stone Age",
            "Agricultural Era",
            "Knowledge Era",
            "Industrial Era",
            "Information Era",
        ];
        let current_idx = eras
            .iter()
            .position(|e| e == &self.current_era)
            .unwrap_or(0);
        if current_idx + 1 < eras.len() {
            self.current_era = eras[current_idx + 1].to_string();
            self.history
                .push(format!("Era advanced: {}", self.current_era));
        }
    }

    pub fn earn_influence(&mut self, amount: f32, reason: &str) {
        self.divine_influence += amount;
        self.total_influence_earned += amount;
        self.history
            .push(format!("+{:.0} Divine Influence: {}", amount, reason));
    }

    pub fn spend_influence(&mut self, amount: f32) -> bool {
        if self.divine_influence >= amount {
            self.divine_influence -= amount;
            return true;
        }
        false
    }

    pub fn apply_effects(&mut self, effects: &[FocusEffect]) -> Vec<String> {
        let mut logs = Vec::new();
        for effect in effects {
            match effect {
                FocusEffect::UnlockFocus(id) => self.unlock_focus(*id),
                FocusEffect::ModifyStat(stat, delta) => match stat.as_str() {
                    "stability" => {
                        self.stability = (self.stability + delta).clamp(0.0, 100.0);
                        logs.push(format!(
                            "Stability {}{:.0}%",
                            if *delta > 0.0 { "+" } else { "" },
                            delta * 100.0
                        ));
                    }
                    "authority" => {
                        self.ideology.authority = (self.ideology.authority + delta).clamp(0.0, 1.0);
                    }
                    "equality" => {
                        self.ideology.equality = (self.ideology.equality + delta).clamp(0.0, 1.0);
                    }
                    "tradition" => {
                        self.ideology.tradition = (self.ideology.tradition + delta).clamp(0.0, 1.0);
                    }
                    "spirituality" => {
                        self.ideology.spirituality =
                            (self.ideology.spirituality + delta).clamp(0.0, 1.0);
                    }
                    "militarism" => {
                        self.ideology.militarism =
                            (self.ideology.militarism + delta).clamp(0.0, 1.0);
                    }
                    "individualism" => {
                        self.ideology.individualism =
                            (self.ideology.individualism + delta).clamp(0.0, 1.0);
                    }
                    _ => {}
                },
                FocusEffect::UnlockTech(id) => self.unlock_tech(*id),
                FocusEffect::ChangeGovernment(gov) => {
                    self.government_type = gov.clone();
                    logs.push(format!("Government changed to {}", gov));
                }
                FocusEffect::SpawnEvent(desc) => {
                    logs.push(desc.clone());
                }
                FocusEffect::GrantInfluence(amount) => {
                    self.divine_influence += *amount;
                }
                FocusEffect::UnlockEra(era) => {
                    self.current_era = era.clone();
                    logs.push(format!("Era changed to {}", era));
                }
            }
        }
        if let Some(first) = effects.first() {
            if let FocusEffect::ChangeGovernment(_) = first {
                // already logged above
            }
        }
        logs
    }

    pub fn unlock_focus(&mut self, focus_id: u32) {
        if !self.unlocked_focuses.contains(&focus_id) {
            self.unlocked_focuses.push(focus_id);
        }
    }

    pub fn activate_focus(&mut self, focus_id: u32) {
        if !self.active_focuses.contains(&focus_id) {
            self.active_focuses.push(focus_id);
        }
    }

    pub fn add_city(&mut self, name: String, col: i32, row: i32, population: u32, tick: u64) {
        self.cities.push(City {
            name,
            col,
            row,
            population,
            founded_tick: tick,
        });
    }

    pub fn record_event(&mut self, event_id: u32, description: &str) {
        if !self.events_completed.contains(&event_id) {
            self.events_completed.push(event_id);
            self.history.push(description.to_string());
        }
    }

    pub fn available_focuses<'a>(&'a self, all_focuses: &'a [FocusNode]) -> Vec<&'a FocusNode> {
        all_focuses
            .iter()
            .filter(|f| !self.unlocked_focuses.contains(&f.id) && f.is_available(self))
            .collect()
    }
}

#[derive(Clone, Debug, Default)]
pub struct CivBonus {
    pub regen_bonus: f32,
    pub food_bonus: f32,
}

pub fn build_focus_tree() -> Vec<FocusNode> {
    vec![
        FocusNode {
            id: 1,
            name: "Tribal Society".to_string(),
            description: "Organize into a cohesive tribe".to_string(),
            era: "Stone Age".to_string(),
            cost: 0.0,
            requirements: vec![],
            effects: vec![
                FocusEffect::UnlockFocus(2),
                FocusEffect::ModifyStat("stability".to_string(), 0.1),
            ],
            unlocked_ids: vec![2],
            category: "Social".to_string(),
        },
        FocusNode {
            id: 2,
            name: "The First Settlements".to_string(),
            description: "Establish permanent dwellings".to_string(),
            era: "Stone Age".to_string(),
            cost: 30.0,
            requirements: vec![FocusRequirement::Population(30)],
            effects: vec![
                FocusEffect::UnlockFocus(3),
                FocusEffect::UnlockFocus(4),
                FocusEffect::ModifyStat("stability".to_string(), 0.1),
            ],
            unlocked_ids: vec![3, 4],
            category: "Social".to_string(),
        },
        FocusNode {
            id: 3,
            name: "Agricultural Path".to_string(),
            description: "Embrace farming and permanent cities".to_string(),
            era: "Agricultural Era".to_string(),
            cost: 50.0,
            requirements: vec![FocusRequirement::EraReached("Agricultural Era".to_string())],
            effects: vec![
                FocusEffect::UnlockFocus(5),
                FocusEffect::ModifyStat("tradition".to_string(), 0.1),
                FocusEffect::ModifyStat("equality".to_string(), -0.05),
            ],
            unlocked_ids: vec![5],
            category: "Technology".to_string(),
        },
        FocusNode {
            id: 4,
            name: "Nomadic Path".to_string(),
            description: "Embrace the great migration".to_string(),
            era: "Stone Age".to_string(),
            cost: 50.0,
            requirements: vec![FocusRequirement::EraReached("Agricultural Era".to_string())],
            effects: vec![
                FocusEffect::UnlockFocus(6),
                FocusEffect::ModifyStat("individualism".to_string(), 0.15),
                FocusEffect::ModifyStat("tradition".to_string(), -0.1),
            ],
            unlocked_ids: vec![6],
            category: "Social".to_string(),
        },
        FocusNode {
            id: 5,
            name: "Permanent Cities".to_string(),
            description: "Build lasting urban centers".to_string(),
            era: "Agricultural Era".to_string(),
            cost: 80.0,
            requirements: vec![
                FocusRequirement::Population(100),
                FocusRequirement::CityCount(1),
            ],
            effects: vec![
                FocusEffect::UnlockFocus(7),
                FocusEffect::UnlockFocus(8),
                FocusEffect::ModifyStat("stability".to_string(), 0.15),
            ],
            unlocked_ids: vec![7, 8],
            category: "Technology".to_string(),
        },
        FocusNode {
            id: 6,
            name: "Great Migration".to_string(),
            description: "Expand across the continent".to_string(),
            era: "Agricultural Era".to_string(),
            cost: 80.0,
            requirements: vec![FocusRequirement::Population(100)],
            effects: vec![
                FocusEffect::UnlockFocus(9),
                FocusEffect::ModifyStat("militarism".to_string(), 0.1),
                FocusEffect::ModifyStat("authority".to_string(), 0.1),
            ],
            unlocked_ids: vec![9],
            category: "Military".to_string(),
        },
        FocusNode {
            id: 7,
            name: "Centralized Government".to_string(),
            description: "Consolidate power under a central authority".to_string(),
            era: "Agricultural Era".to_string(),
            cost: 100.0,
            requirements: vec![FocusRequirement::Population(200)],
            effects: vec![
                FocusEffect::UnlockFocus(10),
                FocusEffect::ModifyStat("authority".to_string(), 0.2),
                FocusEffect::ModifyStat("stability".to_string(), 0.1),
            ],
            unlocked_ids: vec![10],
            category: "Political".to_string(),
        },
        FocusNode {
            id: 8,
            name: "Local Clans Federation".to_string(),
            description: "Maintain clan autonomy with shared governance".to_string(),
            era: "Agricultural Era".to_string(),
            cost: 100.0,
            requirements: vec![FocusRequirement::Population(200)],
            effects: vec![
                FocusEffect::UnlockFocus(11),
                FocusEffect::ModifyStat("equality".to_string(), 0.15),
                FocusEffect::ModifyStat("tradition".to_string(), 0.1),
            ],
            unlocked_ids: vec![11],
            category: "Political".to_string(),
        },
        FocusNode {
            id: 9,
            name: "Warrior Culture".to_string(),
            description: "Embrace martial values and conquest".to_string(),
            era: "Agricultural Era".to_string(),
            cost: 120.0,
            requirements: vec![FocusRequirement::Population(150)],
            effects: vec![
                FocusEffect::UnlockFocus(12),
                FocusEffect::ModifyStat("militarism".to_string(), 0.25),
                FocusEffect::ModifyStat("authority".to_string(), 0.15),
            ],
            unlocked_ids: vec![12],
            category: "Military".to_string(),
        },
        FocusNode {
            id: 10,
            name: "Monarchy".to_string(),
            description: "Establish a hereditary monarchy".to_string(),
            era: "Knowledge Era".to_string(),
            cost: 150.0,
            requirements: vec![
                FocusRequirement::Population(300),
                FocusRequirement::EraReached("Knowledge Era".to_string()),
            ],
            effects: vec![
                FocusEffect::ChangeGovernment("Monarchy".to_string()),
                FocusEffect::ModifyStat("authority".to_string(), 0.2),
                FocusEffect::ModifyStat("tradition".to_string(), 0.2),
            ],
            unlocked_ids: vec![],
            category: "Political".to_string(),
        },
        FocusNode {
            id: 11,
            name: "Republic".to_string(),
            description: "Establish a representative council".to_string(),
            era: "Knowledge Era".to_string(),
            cost: 150.0,
            requirements: vec![
                FocusRequirement::Population(300),
                FocusRequirement::EraReached("Knowledge Era".to_string()),
            ],
            effects: vec![
                FocusEffect::ChangeGovernment("Republic".to_string()),
                FocusEffect::ModifyStat("equality".to_string(), 0.2),
                FocusEffect::ModifyStat("individualism".to_string(), 0.15),
            ],
            unlocked_ids: vec![],
            category: "Political".to_string(),
        },
        FocusNode {
            id: 12,
            name: "Military Dictatorship".to_string(),
            description: "Rule through military strength".to_string(),
            era: "Agricultural Era".to_string(),
            cost: 200.0,
            requirements: vec![
                FocusRequirement::Population(400),
                FocusRequirement::EraReached("Knowledge Era".to_string()),
            ],
            effects: vec![
                FocusEffect::ChangeGovernment("Military Dictatorship".to_string()),
                FocusEffect::ModifyStat("militarism".to_string(), 0.3),
                FocusEffect::ModifyStat("authority".to_string(), 0.25),
            ],
            unlocked_ids: vec![],
            category: "Military".to_string(),
        },
    ]
}
