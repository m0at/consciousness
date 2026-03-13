use crate::rng::Rng;
use std::collections::HashMap;

// ── Catalog entries stored as static slices ────────────────────────────────

struct CatalogEntry {
    name: &'static str,
    effects: &'static [(&'static str, f64)],
}

static STIMULUS_CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        name: "social_interaction",
        effects: &[
            ("social_awareness", 0.25),
            ("theory_of_mind", 0.20),
            ("emotional_awareness", 0.15),
            ("self-monitoring", 0.10),
        ],
    },
    CatalogEntry {
        name: "challenge",
        effects: &[
            ("agency", 0.20),
            ("self-efficacy", 0.15),
            ("motivation", 0.20),
            ("self-regulation", 0.10),
            ("situational_awareness", 0.10),
        ],
    },
    CatalogEntry {
        name: "threat",
        effects: &[
            ("situational_awareness", 0.30),
            ("emotional_awareness", 0.25),
            ("self-regulation", -0.20),
            ("agency", -0.10),
            ("body_awareness", 0.15),
        ],
    },
    CatalogEntry {
        name: "reward",
        effects: &[
            ("self-esteem", 0.25),
            ("motivation", 0.20),
            ("self-efficacy", 0.20),
            ("agency", 0.15),
        ],
    },
    CatalogEntry {
        name: "loss",
        effects: &[
            ("emotional_awareness", 0.30),
            ("self-esteem", -0.20),
            ("motivation", -0.15),
            ("reflection", 0.15),
        ],
    },
    CatalogEntry {
        name: "novelty",
        effects: &[
            ("situational_awareness", 0.20),
            ("temporal_awareness", 0.15),
            ("metacognition", 0.10),
            ("introspection", 0.10),
        ],
    },
    CatalogEntry {
        name: "moral_dilemma",
        effects: &[
            ("moral_awareness", 0.30),
            ("theory_of_mind", 0.20),
            ("reflection", 0.20),
            ("introspection", 0.15),
            ("self-regulation", 0.10),
        ],
    },
    CatalogEntry {
        name: "flow_state",
        effects: &[
            ("agency", 0.20),
            ("self-efficacy", 0.20),
            ("motivation", 0.15),
            ("metacognition", 0.15),
            ("self-monitoring", -0.10),
        ],
    },
    CatalogEntry {
        name: "social_rejection",
        effects: &[
            ("self-esteem", -0.25),
            ("social_awareness", 0.20),
            ("emotional_awareness", 0.25),
            ("self-concept", -0.10),
            ("agency", -0.10),
        ],
    },
    CatalogEntry {
        name: "accomplishment",
        effects: &[
            ("self-efficacy", 0.25),
            ("self-esteem", 0.20),
            ("motivation", 0.15),
            ("self-development", 0.15),
            ("agency", 0.15),
        ],
    },
];

static INTERNAL_EVENTS: &[CatalogEntry] = &[
    CatalogEntry {
        name: "mind_wandering",
        effects: &[
            ("introspection", 0.15),
            ("temporal_awareness", 0.10),
            ("self-monitoring", -0.10),
            ("situational_awareness", -0.10),
        ],
    },
    CatalogEntry {
        name: "sudden_insight",
        effects: &[
            ("metacognition", 0.25),
            ("introspection", 0.15),
            ("self-efficacy", 0.10),
            ("motivation", 0.10),
        ],
    },
    CatalogEntry {
        name: "intrusive_thought",
        effects: &[
            ("emotional_awareness", 0.20),
            ("self-regulation", 0.15),
            ("introspection", 0.10),
            ("self-esteem", -0.10),
        ],
    },
    CatalogEntry {
        name: "self_doubt",
        effects: &[
            ("self-esteem", -0.20),
            ("self-efficacy", -0.15),
            ("self-concept", -0.10),
            ("introspection", 0.15),
        ],
    },
    CatalogEntry {
        name: "nostalgia",
        effects: &[
            ("temporal_awareness", 0.20),
            ("emotional_awareness", 0.15),
            ("reflection", 0.15),
            ("self-concept", 0.10),
        ],
    },
    CatalogEntry {
        name: "creative_impulse",
        effects: &[
            ("metacognition", 0.15),
            ("motivation", 0.15),
            ("agency", 0.10),
            ("introspection", 0.10),
        ],
    },
];

static POSITIVE_INPUT_EFFECTS: &[(&str, f64)] = &[
    ("agency", 0.30),
    ("self-esteem", 0.25),
    ("motivation", 0.25),
    ("self-efficacy", 0.20),
    ("self-development", 0.10),
    ("self-concept", 0.10),
    ("self-regulation", 0.05),
];

static NEGATIVE_INPUT_EFFECTS: &[(&str, f64)] = &[
    ("emotional_awareness", 0.30),
    ("situational_awareness", 0.20),
    ("self-regulation", -0.15),
    ("self-esteem", -0.20),
    ("agency", -0.15),
    ("motivation", -0.10),
];

// Indices into INTERNAL_EVENTS for valence-gated pools
static SAFE_POOL: &[usize] = &[1, 5, 4]; // sudden_insight, creative_impulse, nostalgia
static HOSTILE_POOL: &[usize] = &[2, 3, 0]; // intrusive_thought, self_doubt, mind_wandering

// ── Types ──────────────────────────────────────────────────────────────────

pub struct ActiveStimulus {
    pub name: String,
    pub effects: Vec<(usize, f64)>,
    pub strength: f64,
}

pub struct Environment {
    pub active: Vec<ActiveStimulus>,
    pub valence_memory: f64,
    n: usize,
    stimulus_prob_base: f64,
    internal_event_prob: f64,
    valence_decay: f64,
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn resolve_effects(
    raw: &[(&'static str, f64)],
    aspect_index: &HashMap<String, usize>,
) -> Vec<(usize, f64)> {
    raw.iter()
        .filter_map(|(name, mag)| aspect_index.get(*name).map(|&i| (i, *mag)))
        .collect()
}

// ── Implementation ─────────────────────────────────────────────────────────

impl Environment {
    pub fn new(n: usize) -> Self {
        Self {
            active: Vec::new(),
            valence_memory: 0.0,
            n,
            stimulus_prob_base: 0.03,
            internal_event_prob: 0.08,
            valence_decay: 0.002,
        }
    }

    pub fn generate_stimuli(
        &mut self,
        _weights: &[f64],
        tick: u64,
        rng: &mut Rng,
        aspect_index: &HashMap<String, usize>,
    ) -> Vec<f64> {
        let mut signal = vec![0.0f64; self.n];

        // Random external stimulus with ramped probability
        let prob = (self.stimulus_prob_base + tick as f64 * 0.0002).min(0.25);
        if rng.bool_with_prob(prob) {
            let idx = rng.choice_index(STIMULUS_CATALOG.len());
            let entry = &STIMULUS_CATALOG[idx];
            let effects = resolve_effects(entry.effects, aspect_index);
            self.active.push(ActiveStimulus {
                name: entry.name.to_string(),
                effects,
                strength: 1.0,
            });
        }

        // Spontaneous internal events modulated by environmental valence
        if rng.bool_with_prob(self.internal_event_prob) {
            let pool: &[usize] = if self.valence_memory > 0.2 {
                SAFE_POOL
            } else if self.valence_memory < -0.2 {
                HOSTILE_POOL
            } else {
                &[0, 1, 2, 3, 4, 5]
            };
            let entry = &INTERNAL_EVENTS[pool[rng.choice_index(pool.len())]];
            let effects = resolve_effects(entry.effects, aspect_index);
            self.active.push(ActiveStimulus {
                name: entry.name.to_string(),
                effects,
                strength: 0.7,
            });
        }

        // Aggregate active stimuli into signal
        for stim in &self.active {
            for &(idx, mag) in &stim.effects {
                signal[idx] += mag * stim.strength;
            }
        }

        // Decay (strength *= 0.88) then prune (strength < 0.005)
        for stim in &mut self.active {
            stim.strength *= 0.88;
        }
        self.active.retain(|s| s.strength >= 0.005);

        // Background noise shaped by environmental memory
        let noise_scale = if self.valence_memory > 0.0 {
            1.0 - 0.4 * self.valence_memory
        } else if self.valence_memory < 0.0 {
            1.0 + 0.6 * self.valence_memory.abs()
        } else {
            1.0
        };
        let randomness = (1.0 - (-( tick as f64 / 100.0)).exp()) * noise_scale;
        for s in signal.iter_mut() {
            *s += rng.uniform(-randomness, randomness);
        }

        // Decay environmental memory toward neutral
        self.valence_memory *= 1.0 - self.valence_decay;

        signal
    }

    pub fn apply_input(&mut self, direction: &str, aspect_index: &HashMap<String, usize>) {
        let (raw, delta): (&[(&str, f64)], f64) = if direction == "positive" {
            (POSITIVE_INPUT_EFFECTS, 0.05)
        } else {
            (NEGATIVE_INPUT_EFFECTS, -0.05)
        };

        let effects = resolve_effects(raw, aspect_index);
        self.active.push(ActiveStimulus {
            name: "user_input".to_string(),
            effects,
            strength: 1.0,
        });

        self.valence_memory = (self.valence_memory + delta).clamp(-1.0, 1.0);
    }

    pub fn get_active_stimulus_names(&self) -> Vec<String> {
        self.active
            .iter()
            .filter(|s| s.strength > 0.05)
            .map(|s| s.name.clone())
            .collect()
    }

    pub fn get_status(&self) -> String {
        // Deduplicate names, take up to 3, mirror Python's set behaviour
        let mut seen = std::collections::HashSet::new();
        let names: Vec<&str> = self
            .active
            .iter()
            .filter(|s| s.strength > 0.05)
            .filter_map(|s| {
                if seen.insert(s.name.as_str()) {
                    Some(s.name.as_str())
                } else {
                    None
                }
            })
            .take(3)
            .collect();

        let valence_str = format!("valence={:+.2}", self.valence_memory);
        if names.is_empty() {
            valence_str
        } else {
            format!("{} | {}", names.join(", "), valence_str)
        }
    }
}
