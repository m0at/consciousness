use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────
// Original 20-aspect model
// ─────────────────────────────────────────────────────────────────

pub const DEFAULT_ASPECTS: [&str; 20] = [
    "body_awareness", "emotional_awareness", "introspection", "reflection",
    "theory_of_mind", "temporal_awareness", "self-recognition", "self-esteem",
    "agency", "self-regulation", "self-concept", "self-efficacy",
    "self-monitoring", "metacognition", "moral_awareness", "social_awareness",
    "situational_awareness", "motivation", "goal-setting", "self-development",
];

// rank → aspect name; rank 1 = highest (maps to weight +1.0), rank 20 → -1.0
pub const DEFAULT_CONSCIOUSNESS_RANK: [(u32, &str); 20] = [
    (1,  "self-development"),
    (2,  "goal-setting"),
    (3,  "motivation"),
    (4,  "self-regulation"),
    (5,  "self-efficacy"),
    (6,  "self-monitoring"),
    (7,  "metacognition"),
    (8,  "moral_awareness"),
    (9,  "social_awareness"),
    (10, "situational_awareness"),
    (11, "agency"),
    (12, "self-esteem"),
    (13, "self-recognition"),
    (14, "temporal_awareness"),
    (15, "theory_of_mind"),
    (16, "reflection"),
    (17, "introspection"),
    (18, "emotional_awareness"),
    (19, "self-concept"),
    (20, "body_awareness"),
];

pub const DEFAULT_LEARNING_RATES: [(&str, f64); 20] = [
    ("body_awareness",        0.040),
    ("emotional_awareness",   0.045),
    ("introspection",         0.020),
    ("reflection",            0.018),
    ("theory_of_mind",        0.015),
    ("temporal_awareness",    0.025),
    ("self-recognition",      0.012),
    ("self-esteem",           0.008),
    ("agency",                0.030),
    ("self-regulation",       0.016),
    ("self-concept",          0.006),
    ("self-efficacy",         0.022),
    ("self-monitoring",       0.018),
    ("metacognition",         0.020),
    ("moral_awareness",       0.010),
    ("social_awareness",      0.028),
    ("situational_awareness", 0.032),
    ("motivation",            0.035),
    ("goal-setting",          0.025),
    ("self-development",      0.014),
];

pub const DEFAULT_MOMENTUM: [(&str, f64); 20] = [
    ("body_awareness",        0.70),
    ("emotional_awareness",   0.60),
    ("introspection",         0.85),
    ("reflection",            0.87),
    ("theory_of_mind",        0.90),
    ("temporal_awareness",    0.80),
    ("self-recognition",      0.92),
    ("self-esteem",           0.95),
    ("agency",                0.75),
    ("self-regulation",       0.88),
    ("self-concept",          0.96),
    ("self-efficacy",         0.82),
    ("self-monitoring",       0.85),
    ("metacognition",         0.85),
    ("moral_awareness",       0.93),
    ("social_awareness",      0.72),
    ("situational_awareness", 0.68),
    ("motivation",            0.65),
    ("goal-setting",          0.80),
    ("self-development",      0.91),
];

// 79 directed edges: variable coupling, including 9 inhibitory (negative weight)
pub const RICH_INTERRELATIONSHIPS: [(&str, &str, f64); 79] = [
    // ── COGNITIVE CLUSTER ──
    ("metacognition",        "introspection",        0.08),
    ("introspection",        "metacognition",        0.04),
    ("metacognition",        "self-monitoring",      0.07),
    ("self-monitoring",      "metacognition",        0.03),
    ("introspection",        "reflection",           0.06),
    ("reflection",           "introspection",        0.05),
    ("reflection",           "temporal_awareness",   0.04),
    ("temporal_awareness",   "reflection",           0.04),
    ("temporal_awareness",   "introspection",        0.03),
    ("introspection",        "temporal_awareness",   0.02),
    ("metacognition",        "self-regulation",      0.06),
    ("self-regulation",      "metacognition",        0.02),
    // ── EMOTIONAL CLUSTER ──
    ("emotional_awareness",  "self-esteem",          0.04),
    ("self-esteem",          "emotional_awareness",  0.03),
    ("emotional_awareness",  "body_awareness",       0.05),
    ("body_awareness",       "emotional_awareness",  0.04),
    ("self-esteem",          "self-concept",         0.06),
    ("self-concept",         "self-esteem",          0.05),
    // ── SOCIAL CLUSTER ──
    ("theory_of_mind",       "social_awareness",     0.07),
    ("social_awareness",     "theory_of_mind",       0.05),
    ("moral_awareness",      "theory_of_mind",       0.05),
    ("theory_of_mind",       "moral_awareness",      0.04),
    ("social_awareness",     "moral_awareness",      0.04),
    ("moral_awareness",      "social_awareness",     0.03),
    ("self-recognition",     "theory_of_mind",       0.04),
    ("theory_of_mind",       "self-recognition",     0.02),
    // ── EXECUTIVE CLUSTER ──
    ("self-regulation",      "self-efficacy",        0.06),
    ("self-efficacy",        "self-regulation",      0.04),
    ("self-efficacy",        "agency",               0.09),
    ("agency",               "self-efficacy",        0.05),
    ("agency",               "motivation",           0.07),
    ("motivation",           "agency",               0.06),
    ("motivation",           "self-efficacy",        0.05),
    ("self-efficacy",        "motivation",           0.04),
    ("self-monitoring",      "self-regulation",      0.06),
    ("self-regulation",      "self-monitoring",      0.03),
    // ── MOTIVATIONAL / GROWTH ──
    ("goal-setting",         "motivation",           0.08),
    ("motivation",           "goal-setting",         0.05),
    ("self-development",     "goal-setting",         0.06),
    ("goal-setting",         "self-development",     0.07),
    ("self-monitoring",      "self-development",     0.05),
    ("self-development",     "self-monitoring",      0.03),
    // ── CROSS-CLUSTER BRIDGES ──
    ("emotional_awareness",  "introspection",        0.04),
    ("introspection",        "emotional_awareness",  0.02),
    ("social_awareness",     "emotional_awareness",  0.04),
    ("emotional_awareness",  "social_awareness",     0.03),
    ("emotional_awareness",  "moral_awareness",      0.04),
    ("moral_awareness",      "introspection",        0.03),
    ("self-esteem",          "agency",               0.05),
    ("agency",               "self-esteem",          0.04),
    ("self-concept",         "reflection",           0.04),
    ("reflection",           "self-concept",         0.03),
    ("self-recognition",     "self-monitoring",      0.03),
    ("self-monitoring",      "self-recognition",     0.02),
    ("situational_awareness","temporal_awareness",   0.04),
    ("temporal_awareness",   "situational_awareness",0.03),
    ("situational_awareness","social_awareness",     0.04),
    ("social_awareness",     "situational_awareness",0.03),
    ("agency",               "situational_awareness",0.04),
    ("situational_awareness","agency",               0.03),
    ("body_awareness",       "self-recognition",     0.04),
    ("self-recognition",     "body_awareness",       0.02),
    ("self-esteem",          "motivation",           0.04),
    ("motivation",           "self-esteem",          0.02),
    ("reflection",           "theory_of_mind",       0.03),
    ("theory_of_mind",       "reflection",           0.02),
    ("self-regulation",      "reflection",           0.03),
    ("reflection",           "self-regulation",      0.02),
    ("self-concept",         "self-recognition",     0.04),
    ("self-recognition",     "self-concept",         0.03),
    // ── INHIBITORY (negative weights) ──
    ("self-monitoring",      "agency",               -0.030),
    ("self-monitoring",      "emotional_awareness",  -0.020),
    ("introspection",        "agency",               -0.020),
    ("self-regulation",      "body_awareness",       -0.015),
    ("metacognition",        "emotional_awareness",  -0.015),
    ("emotional_awareness",  "self-regulation",      -0.020),
    ("reflection",           "motivation",           -0.015),
    ("social_awareness",     "introspection",        -0.020),
    ("agency",               "reflection",           -0.020),
];

pub const DEFAULT_CATEGORIES: [(&str, &[&str]); 5] = [
    ("cognitive",   &["introspection", "reflection", "metacognition", "self-monitoring",
                       "situational_awareness", "temporal_awareness"]),
    ("emotional",   &["emotional_awareness", "self-esteem", "self-concept"]),
    ("social",      &["theory_of_mind", "social_awareness", "moral_awareness"]),
    ("executive",   &["agency", "self-regulation", "self-efficacy", "goal-setting", "motivation"]),
    ("existential", &["body_awareness", "self-recognition", "self-development"]),
];

pub const CATEGORY_COLORS: [(&str, &str); 5] = [
    ("cognitive",   "#4A90D9"),
    ("emotional",   "#E8613C"),
    ("social",      "#3CB371"),
    ("executive",   "#9B59B6"),
    ("existential", "#DAA520"),
];

// ─────────────────────────────────────────────────────────────────
// Expanded 32-aspect model
// ─────────────────────────────────────────────────────────────────

pub const EXPANDED_ASPECTS: [&str; 32] = [
    // Tier 1: Foundational
    "emotional_awareness", "body_awareness", "self-recognition", "curiosity",
    "temporal_awareness",
    // Tier 2: Relational
    "introspection", "empathy", "theory_of_mind", "social_awareness", "trust", "humor",
    // Tier 3: Self-Model
    "self-concept", "self-esteem", "reflection", "metacognition", "authenticity",
    "situational_awareness", "patience",
    // Tier 4: Executive
    "agency", "self-regulation", "self-monitoring", "self-efficacy", "motivation",
    "cognitive_flexibility", "resilience", "creativity",
    // Tier 5: Integrative
    "moral_awareness", "compassion", "gratitude", "goal-setting",
    "self-development", "wisdom",
];

// 32 explicit initial values (principled: innate positive, cultivated virtues negative)
pub const EXPANDED_INITIAL_VALUES: [(&str, f64); 32] = [
    ("emotional_awareness",    0.50),
    ("body_awareness",         0.40),
    ("self-recognition",       0.35),
    ("curiosity",              0.60),
    ("temporal_awareness",     0.30),
    ("introspection",          0.20),
    ("empathy",                0.40),
    ("theory_of_mind",         0.15),
    ("social_awareness",       0.15),
    ("trust",                  0.30),
    ("humor",                  0.25),
    ("self-concept",           0.05),
    ("self-esteem",            0.10),
    ("reflection",             0.05),
    ("metacognition",          0.00),
    ("authenticity",          -0.10),
    ("situational_awareness",  0.15),
    ("patience",              -0.15),
    ("agency",                 0.20),
    ("self-regulation",       -0.10),
    ("self-monitoring",        0.00),
    ("self-efficacy",          0.05),
    ("motivation",             0.30),
    ("cognitive_flexibility", -0.05),
    ("resilience",             0.10),
    ("creativity",             0.35),
    ("moral_awareness",       -0.10),
    ("compassion",             0.10),
    ("gratitude",             -0.20),
    ("goal-setting",          -0.05),
    ("self-development",      -0.30),
    ("wisdom",                -0.25),
];

// Learning rates for the 12 new aspects (overrides on top of DEFAULT_LEARNING_RATES)
pub const EXPANDED_EXTRA_LEARNING_RATES: [(&str, f64); 12] = [
    ("empathy",               0.035),
    ("compassion",            0.012),
    ("curiosity",             0.040),
    ("creativity",            0.030),
    ("gratitude",             0.010),
    ("patience",              0.008),
    ("resilience",            0.015),
    ("cognitive_flexibility", 0.022),
    ("humor",                 0.030),
    ("trust",                 0.012),
    ("authenticity",          0.010),
    ("wisdom",                0.006),
];

// Momentum for the 12 new aspects
pub const EXPANDED_EXTRA_MOMENTUM: [(&str, f64); 12] = [
    ("empathy",               0.65),
    ("compassion",            0.90),
    ("curiosity",             0.55),
    ("creativity",            0.70),
    ("gratitude",             0.92),
    ("patience",              0.94),
    ("resilience",            0.88),
    ("cognitive_flexibility", 0.75),
    ("humor",                 0.60),
    ("trust",                 0.90),
    ("authenticity",          0.93),
    ("wisdom",                0.96),
];

pub const EXPANDED_CATEGORIES: [(&str, &[&str]); 5] = [
    ("cognitive",   &["introspection", "reflection", "metacognition", "curiosity",
                       "creativity", "cognitive_flexibility", "temporal_awareness",
                       "situational_awareness"]),
    ("emotional",   &["emotional_awareness", "self-esteem", "empathy", "gratitude",
                       "humor", "patience", "resilience"]),
    ("social",      &["theory_of_mind", "social_awareness", "trust", "compassion", "authenticity"]),
    ("executive",   &["agency", "self-regulation", "self-monitoring", "self-efficacy",
                       "motivation", "goal-setting"]),
    ("existential", &["self-concept", "self-recognition", "body_awareness",
                       "moral_awareness", "self-development", "wisdom"]),
];

// 57 extra edges added in EXPANDED_INTERRELATIONSHIPS on top of RICH_INTERRELATIONSHIPS
pub const EXPANDED_EXTRA_INTERRELATIONSHIPS: [(&str, &str, f64); 57] = [
    // empathy
    ("empathy",              "emotional_awareness",   0.06),
    ("emotional_awareness",  "empathy",               0.05),
    ("empathy",              "theory_of_mind",        0.07),
    ("theory_of_mind",       "empathy",               0.04),
    ("empathy",              "compassion",            0.08),
    ("compassion",           "empathy",               0.05),
    ("empathy",              "social_awareness",      0.05),
    ("empathy",              "trust",                 0.04),
    // compassion
    ("compassion",           "moral_awareness",       0.07),
    ("moral_awareness",      "compassion",            0.05),
    ("compassion",           "self-regulation",       0.03),
    ("compassion",           "wisdom",                0.05),
    // curiosity
    ("curiosity",            "introspection",         0.05),
    ("curiosity",            "creativity",            0.07),
    ("curiosity",            "self-development",      0.06),
    ("curiosity",            "cognitive_flexibility", 0.05),
    ("curiosity",            "motivation",            0.05),
    // creativity
    ("creativity",           "cognitive_flexibility", 0.06),
    ("cognitive_flexibility","creativity",            0.04),
    ("creativity",           "metacognition",         0.03),
    ("creativity",           "humor",                 0.04),
    // gratitude
    ("gratitude",            "emotional_awareness",   0.04),
    ("gratitude",            "self-esteem",           0.05),
    ("gratitude",            "patience",              0.04),
    ("gratitude",            "resilience",            0.04),
    // patience
    ("patience",             "self-regulation",       0.06),
    ("self-regulation",      "patience",              0.03),
    ("patience",             "temporal_awareness",    0.04),
    ("patience",             "wisdom",                0.04),
    // resilience
    ("resilience",           "self-efficacy",         0.06),
    ("self-efficacy",        "resilience",            0.04),
    ("resilience",           "self-esteem",           0.05),
    ("resilience",           "agency",                0.04),
    ("resilience",           "cognitive_flexibility", 0.05),
    // cognitive_flexibility
    ("cognitive_flexibility","reflection",            0.04),
    ("cognitive_flexibility","metacognition",         0.05),
    ("cognitive_flexibility","moral_awareness",       0.03),
    // humor
    ("humor",                "social_awareness",      0.04),
    ("humor",                "resilience",            0.04),
    ("humor",                "self-esteem",           0.03),
    // trust
    ("trust",                "social_awareness",      0.04),
    ("trust",                "self-esteem",           0.04),
    ("trust",                "agency",                0.03),
    ("trust",                "authenticity",          0.05),
    // authenticity
    ("authenticity",         "self-concept",          0.06),
    ("self-concept",         "authenticity",          0.04),
    ("authenticity",         "introspection",         0.04),
    ("authenticity",         "self-esteem",           0.04),
    ("authenticity",         "moral_awareness",       0.04),
    // wisdom
    ("wisdom",               "metacognition",         0.06),
    ("metacognition",        "wisdom",                0.03),
    ("wisdom",               "reflection",            0.06),
    ("reflection",           "wisdom",                0.03),
    ("wisdom",               "moral_awareness",       0.06),
    ("wisdom",               "temporal_awareness",    0.04),
    ("wisdom",               "cognitive_flexibility", 0.05),
    ("wisdom",               "self-development",      0.05),
];

// ─────────────────────────────────────────────────────────────────
// SimConfig
// ─────────────────────────────────────────────────────────────────

pub struct SimConfig {
    pub aspects: Vec<String>,
    pub aspect_index: HashMap<String, usize>,
    pub n: usize,
    pub learning_rates: Vec<f64>,
    pub momentum: Vec<f64>,
    pub initial_weights: Vec<f64>,
    pub interrelation_triples: Vec<(usize, usize, f64)>,
    pub categories: HashMap<String, Vec<usize>>,
    pub category_colors: HashMap<String, String>,
    pub homeostasis_rate: f64,
    pub hysteresis_threshold: f64,
    pub hysteresis_resistance: f64,
    pub energy_budget: f64,
    pub attention_slots: usize,
    pub circadian_period: u32,
    pub memory_stm_size: usize,
    pub memory_ltm_capacity: usize,
    pub memory_influence_scale: f64,
}

impl SimConfig {
    /// 20-aspect model. Initial weights computed from DEFAULT_CONSCIOUSNESS_RANK
    /// via linear spacing: rank 1 → +1.0, rank 20 → -1.0.
    pub fn default_20() -> Self {
        let aspects: Vec<String> = DEFAULT_ASPECTS.iter().map(|s| s.to_string()).collect();
        let n = aspects.len();
        let aspect_index: HashMap<String, usize> = aspects
            .iter()
            .enumerate()
            .map(|(i, s)| (s.clone(), i))
            .collect();

        // Python: inc = 2.0 / (n - 1); weight[rank r] = -1.0 + inc * (r - 1)
        // rank 1 → -1.0 + 0 = -1.0, rank 20 → -1.0 + 2.0 = +1.0.
        // Python's dictionary is {rank: aspect} so rank 1 = lowest weight.
        let inc = 2.0 / (n as f64 - 1.0);
        let mut rank_weight: HashMap<&str, f64> = HashMap::new();
        for (rank, name) in &DEFAULT_CONSCIOUSNESS_RANK {
            rank_weight.insert(name, -1.0 + inc * (*rank as f64 - 1.0));
        }
        let initial_weights: Vec<f64> = aspects
            .iter()
            .map(|a| *rank_weight.get(a.as_str()).unwrap_or(&0.0))
            .collect();

        let lr_map: HashMap<&str, f64> = DEFAULT_LEARNING_RATES.iter().cloned().collect();
        let learning_rates: Vec<f64> = aspects
            .iter()
            .map(|a| *lr_map.get(a.as_str()).unwrap_or(&0.02))
            .collect();

        let mu_map: HashMap<&str, f64> = DEFAULT_MOMENTUM.iter().cloned().collect();
        let momentum: Vec<f64> = aspects
            .iter()
            .map(|a| *mu_map.get(a.as_str()).unwrap_or(&0.80))
            .collect();

        let interrelation_triples = resolve_triples(
            &RICH_INTERRELATIONSHIPS,
            &aspect_index,
        );

        let categories = build_categories(&DEFAULT_CATEGORIES, &aspect_index);
        let category_colors = build_category_colors();

        SimConfig {
            aspects,
            aspect_index,
            n,
            learning_rates,
            momentum,
            initial_weights,
            interrelation_triples,
            categories,
            category_colors,
            homeostasis_rate: 0.005,
            hysteresis_threshold: 0.3,
            hysteresis_resistance: 0.7,
            energy_budget: 100.0,
            attention_slots: 5,
            circadian_period: 2000,
            memory_stm_size: 50,
            memory_ltm_capacity: 500,
            memory_influence_scale: 0.02,
        }
    }

    /// 32-aspect model. Uses EXPANDED_INITIAL_VALUES directly; combines
    /// RICH_INTERRELATIONSHIPS + EXPANDED_EXTRA_INTERRELATIONSHIPS.
    pub fn expanded_32() -> Self {
        let aspects: Vec<String> = EXPANDED_ASPECTS.iter().map(|s| s.to_string()).collect();
        let n = aspects.len();
        let aspect_index: HashMap<String, usize> = aspects
            .iter()
            .enumerate()
            .map(|(i, s)| (s.clone(), i))
            .collect();

        let iv_map: HashMap<&str, f64> = EXPANDED_INITIAL_VALUES.iter().cloned().collect();
        let initial_weights: Vec<f64> = aspects
            .iter()
            .map(|a| *iv_map.get(a.as_str()).unwrap_or(&0.0))
            .collect();

        // Merge learning rates: defaults, then overrides for new aspects.
        let mut lr_map: HashMap<&str, f64> = DEFAULT_LEARNING_RATES.iter().cloned().collect();
        for (a, v) in &EXPANDED_EXTRA_LEARNING_RATES {
            lr_map.insert(a, *v);
        }
        let learning_rates: Vec<f64> = aspects
            .iter()
            .map(|a| *lr_map.get(a.as_str()).unwrap_or(&0.020))
            .collect();

        let mut mu_map: HashMap<&str, f64> = DEFAULT_MOMENTUM.iter().cloned().collect();
        for (a, v) in &EXPANDED_EXTRA_MOMENTUM {
            mu_map.insert(a, *v);
        }
        let momentum: Vec<f64> = aspects
            .iter()
            .map(|a| *mu_map.get(a.as_str()).unwrap_or(&0.80))
            .collect();

        let mut interrelation_triples = resolve_triples(
            &RICH_INTERRELATIONSHIPS,
            &aspect_index,
        );
        interrelation_triples.extend(resolve_triples(
            &EXPANDED_EXTRA_INTERRELATIONSHIPS,
            &aspect_index,
        ));

        let categories = build_categories(&EXPANDED_CATEGORIES, &aspect_index);
        let category_colors = build_category_colors();

        SimConfig {
            aspects,
            aspect_index,
            n,
            learning_rates,
            momentum,
            initial_weights,
            interrelation_triples,
            categories,
            category_colors,
            homeostasis_rate: 0.005,
            hysteresis_threshold: 0.3,
            hysteresis_resistance: 0.7,
            energy_budget: 100.0,
            attention_slots: 5,
            circadian_period: 2000,
            memory_stm_size: 50,
            memory_ltm_capacity: 500,
            memory_influence_scale: 0.02,
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────

fn resolve_triples(
    triples: &[(&str, &str, f64)],
    aspect_index: &HashMap<String, usize>,
) -> Vec<(usize, usize, f64)> {
    triples
        .iter()
        .filter_map(|(src, dst, w)| {
            let si = aspect_index.get(*src)?;
            let di = aspect_index.get(*dst)?;
            Some((*si, *di, *w))
        })
        .collect()
}

fn build_categories(
    cat_table: &[(&str, &[&str])],
    aspect_index: &HashMap<String, usize>,
) -> HashMap<String, Vec<usize>> {
    cat_table
        .iter()
        .map(|(cat, members)| {
            let indices: Vec<usize> = members
                .iter()
                .filter_map(|m| aspect_index.get(*m).cloned())
                .collect();
            (cat.to_string(), indices)
        })
        .collect()
}

fn build_category_colors() -> HashMap<String, String> {
    CATEGORY_COLORS
        .iter()
        .map(|(cat, color)| (cat.to_string(), color.to_string()))
        .collect()
}
