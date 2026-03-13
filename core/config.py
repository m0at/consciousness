"""Configuration and presets for the consciousness simulation."""

from __future__ import annotations
from dataclasses import dataclass, field


# ─────────────────────────────────────────────────────────────────
# Original 20-aspect model
# ─────────────────────────────────────────────────────────────────

DEFAULT_ASPECTS = [
    'body_awareness', 'emotional_awareness', 'introspection', 'reflection',
    'theory_of_mind', 'temporal_awareness', 'self-recognition', 'self-esteem',
    'agency', 'self-regulation', 'self-concept', 'self-efficacy',
    'self-monitoring', 'metacognition', 'moral_awareness', 'social_awareness',
    'situational_awareness', 'motivation', 'goal-setting', 'self-development',
]

DEFAULT_CONSCIOUSNESS_RANK = {
    1: 'self-development', 2: 'goal-setting', 3: 'motivation',
    4: 'self-regulation', 5: 'self-efficacy', 6: 'self-monitoring',
    7: 'metacognition', 8: 'moral_awareness', 9: 'social_awareness',
    10: 'situational_awareness', 11: 'agency', 12: 'self-esteem',
    13: 'self-recognition', 14: 'temporal_awareness', 15: 'theory_of_mind',
    16: 'reflection', 17: 'introspection', 18: 'emotional_awareness',
    19: 'self-concept', 20: 'body_awareness',
}

# Per-aspect adaptive learning rates — psychological reasoning:
#   emotional_awareness: fastest (reactive), self-concept: slowest (core identity)
DEFAULT_LEARNING_RATES = {
    'body_awareness': 0.040, 'emotional_awareness': 0.045,
    'introspection': 0.020, 'reflection': 0.018, 'theory_of_mind': 0.015,
    'temporal_awareness': 0.025, 'self-recognition': 0.012, 'self-esteem': 0.008,
    'agency': 0.030, 'self-regulation': 0.016, 'self-concept': 0.006,
    'self-efficacy': 0.022, 'self-monitoring': 0.018, 'metacognition': 0.020,
    'moral_awareness': 0.010, 'social_awareness': 0.028,
    'situational_awareness': 0.032, 'motivation': 0.035,
    'goal-setting': 0.025, 'self-development': 0.014,
}

# Per-aspect momentum — high = strong habits, low = quick to change direction
DEFAULT_MOMENTUM = {
    'body_awareness': 0.70, 'emotional_awareness': 0.60,
    'introspection': 0.85, 'reflection': 0.87, 'theory_of_mind': 0.90,
    'temporal_awareness': 0.80, 'self-recognition': 0.92, 'self-esteem': 0.95,
    'agency': 0.75, 'self-regulation': 0.88, 'self-concept': 0.96,
    'self-efficacy': 0.82, 'self-monitoring': 0.85, 'metacognition': 0.85,
    'moral_awareness': 0.93, 'social_awareness': 0.72,
    'situational_awareness': 0.68, 'motivation': 0.65,
    'goal-setting': 0.80, 'self-development': 0.91,
}

# Original symmetric interrelationships (35 pairs at 0.005)
DEFAULT_INTERRELATIONSHIPS = [
    ('metacognition', 'introspection'), ('goal-setting', 'motivation'),
    ('self-regulation', 'self-efficacy'), ('social_awareness', 'emotional_awareness'),
    ('self-monitoring', 'self-development'), ('body_awareness', 'self-recognition'),
    ('self-esteem', 'agency'), ('self-concept', 'reflection'),
    ('self-efficacy', 'goal-setting'), ('moral_awareness', 'theory_of_mind'),
    ('situational_awareness', 'temporal_awareness'), ('introspection', 'reflection'),
    ('reflection', 'theory_of_mind'), ('agency', 'motivation'),
    ('temporal_awareness', 'introspection'), ('emotional_awareness', 'introspection'),
    ('self-recognition', 'theory_of_mind'), ('self-esteem', 'self-concept'),
    ('agency', 'self-regulation'), ('situational_awareness', 'social_awareness'),
    ('moral_awareness', 'introspection'), ('metacognition', 'self-monitoring'),
    ('self-development', 'goal-setting'), ('motivation', 'self-efficacy'),
    ('temporal_awareness', 'reflection'), ('self-recognition', 'self-monitoring'),
    ('social_awareness', 'theory_of_mind'), ('emotional_awareness', 'moral_awareness'),
    ('self-regulation', 'reflection'), ('self-concept', 'self-esteem'),
    ('introspection', 'moral_awareness'), ('theory_of_mind', 'social_awareness'),
    ('self-monitoring', 'self-regulation'), ('goal-setting', 'self-development'),
    ('agency', 'situational_awareness'),
]

# Rich asymmetric interrelationships: (source, target, weight)
# 102 directed edges with variable coupling, including 9 inhibitory relationships
RICH_INTERRELATIONSHIPS = [
    # ── COGNITIVE CLUSTER (strong intra-cluster) ──
    ('metacognition', 'introspection', 0.08),       # strong top-down monitoring
    ('introspection', 'metacognition', 0.04),        # weaker bottom-up feedback
    ('metacognition', 'self-monitoring', 0.07),      # meta-awareness drives tracking
    ('self-monitoring', 'metacognition', 0.03),
    ('introspection', 'reflection', 0.06),           # looking inward feeds review
    ('reflection', 'introspection', 0.05),           # review prompts more looking
    ('reflection', 'temporal_awareness', 0.04),      # reflecting on past/future
    ('temporal_awareness', 'reflection', 0.04),
    ('temporal_awareness', 'introspection', 0.03),
    ('introspection', 'temporal_awareness', 0.02),
    ('metacognition', 'self-regulation', 0.06),      # awareness enables regulation
    ('self-regulation', 'metacognition', 0.02),

    # ── EMOTIONAL CLUSTER ──
    ('emotional_awareness', 'self-esteem', 0.04),    # feelings shape self-evaluation
    ('self-esteem', 'emotional_awareness', 0.03),
    ('emotional_awareness', 'body_awareness', 0.05), # emotions felt in body
    ('body_awareness', 'emotional_awareness', 0.04),
    ('self-esteem', 'self-concept', 0.06),            # evaluation shapes identity
    ('self-concept', 'self-esteem', 0.05),

    # ── SOCIAL CLUSTER ──
    ('theory_of_mind', 'social_awareness', 0.07),    # modeling others → reading groups
    ('social_awareness', 'theory_of_mind', 0.05),
    ('moral_awareness', 'theory_of_mind', 0.05),     # morality needs perspective-taking
    ('theory_of_mind', 'moral_awareness', 0.04),
    ('social_awareness', 'moral_awareness', 0.04),
    ('moral_awareness', 'social_awareness', 0.03),
    ('self-recognition', 'theory_of_mind', 0.04),    # knowing self helps model others
    ('theory_of_mind', 'self-recognition', 0.02),

    # ── EXECUTIVE CLUSTER ──
    ('self-regulation', 'self-efficacy', 0.06),      # control builds confidence
    ('self-efficacy', 'self-regulation', 0.04),
    ('self-efficacy', 'agency', 0.09),               # Bandura: efficacy → action
    ('agency', 'self-efficacy', 0.05),               # success reinforces belief
    ('agency', 'motivation', 0.07),                  # acting generates drive
    ('motivation', 'agency', 0.06),
    ('motivation', 'self-efficacy', 0.05),
    ('self-efficacy', 'motivation', 0.04),
    ('self-monitoring', 'self-regulation', 0.06),    # tracking enables control
    ('self-regulation', 'self-monitoring', 0.03),

    # ── MOTIVATIONAL / GROWTH ──
    ('goal-setting', 'motivation', 0.08),            # goals energize
    ('motivation', 'goal-setting', 0.05),
    ('self-development', 'goal-setting', 0.06),
    ('goal-setting', 'self-development', 0.07),
    ('self-monitoring', 'self-development', 0.05),
    ('self-development', 'self-monitoring', 0.03),

    # ── CROSS-CLUSTER BRIDGES ──
    ('emotional_awareness', 'introspection', 0.04),  # feelings prompt looking inward
    ('introspection', 'emotional_awareness', 0.02),
    ('social_awareness', 'emotional_awareness', 0.04),
    ('emotional_awareness', 'social_awareness', 0.03),
    ('emotional_awareness', 'moral_awareness', 0.04),
    ('moral_awareness', 'introspection', 0.03),
    ('self-esteem', 'agency', 0.05),                 # self-worth enables action
    ('agency', 'self-esteem', 0.04),                 # success boosts self-worth
    ('self-concept', 'reflection', 0.04),
    ('reflection', 'self-concept', 0.03),
    ('self-recognition', 'self-monitoring', 0.03),
    ('self-monitoring', 'self-recognition', 0.02),
    ('situational_awareness', 'temporal_awareness', 0.04),
    ('temporal_awareness', 'situational_awareness', 0.03),
    ('situational_awareness', 'social_awareness', 0.04),
    ('social_awareness', 'situational_awareness', 0.03),
    ('agency', 'situational_awareness', 0.04),
    ('situational_awareness', 'agency', 0.03),
    ('body_awareness', 'self-recognition', 0.04),
    ('self-recognition', 'body_awareness', 0.02),
    ('self-esteem', 'motivation', 0.04),
    ('motivation', 'self-esteem', 0.02),
    ('reflection', 'theory_of_mind', 0.03),
    ('theory_of_mind', 'reflection', 0.02),
    ('self-regulation', 'reflection', 0.03),
    ('reflection', 'self-regulation', 0.02),
    ('self-concept', 'self-recognition', 0.04),
    ('self-recognition', 'self-concept', 0.03),

    # ── INHIBITORY RELATIONSHIPS (negative weights) ──
    ('self-monitoring', 'agency', -0.03),             # choking under pressure
    ('self-monitoring', 'emotional_awareness', -0.02),# masking emotions (Snyder 1974)
    ('introspection', 'agency', -0.02),               # rumination → paralysis
    ('self-regulation', 'body_awareness', -0.015),    # attentional narrowing
    ('metacognition', 'emotional_awareness', -0.015), # intellectualizing affect
    ('emotional_awareness', 'self-regulation', -0.02),# emotional flooding (Gross 1998)
    ('reflection', 'motivation', -0.015),             # rumination suppresses drive
    ('social_awareness', 'introspection', -0.02),     # outward attention competes
    ('agency', 'reflection', -0.02),                  # action suppresses reflection
]

# Aspect categories for visualization and clustering
DEFAULT_CATEGORIES = {
    'cognitive': [
        'introspection', 'reflection', 'metacognition', 'self-monitoring',
        'situational_awareness', 'temporal_awareness',
    ],
    'emotional': ['emotional_awareness', 'self-esteem', 'self-concept'],
    'social': ['theory_of_mind', 'social_awareness', 'moral_awareness'],
    'executive': [
        'agency', 'self-regulation', 'self-efficacy', 'goal-setting', 'motivation',
    ],
    'existential': ['body_awareness', 'self-recognition', 'self-development'],
}

CATEGORY_COLORS = {
    'cognitive':   '#4A90D9',
    'emotional':   '#E8613C',
    'social':      '#3CB371',
    'executive':   '#9B59B6',
    'existential': '#DAA520',
}


# ─────────────────────────────────────────────────────────────────
# Expanded 32-aspect model
# ─────────────────────────────────────────────────────────────────

EXPANDED_ASPECTS = [
    # Tier 1: Foundational
    'emotional_awareness', 'body_awareness', 'self-recognition', 'curiosity',
    'temporal_awareness',
    # Tier 2: Relational
    'introspection', 'empathy', 'theory_of_mind', 'social_awareness', 'trust', 'humor',
    # Tier 3: Self-Model
    'self-concept', 'self-esteem', 'reflection', 'metacognition', 'authenticity',
    'situational_awareness', 'patience',
    # Tier 4: Executive
    'agency', 'self-regulation', 'self-monitoring', 'self-efficacy', 'motivation',
    'cognitive_flexibility', 'resilience', 'creativity',
    # Tier 5: Integrative
    'moral_awareness', 'compassion', 'gratitude', 'goal-setting',
    'self-development', 'wisdom',
]

EXPANDED_CONSCIOUSNESS_RANK = {
    1: 'emotional_awareness', 2: 'body_awareness', 3: 'self-recognition',
    4: 'curiosity', 5: 'temporal_awareness', 6: 'introspection',
    7: 'empathy', 8: 'theory_of_mind', 9: 'social_awareness',
    10: 'trust', 11: 'humor', 12: 'self-concept',
    13: 'self-esteem', 14: 'reflection', 15: 'metacognition',
    16: 'authenticity', 17: 'situational_awareness', 18: 'patience',
    19: 'agency', 20: 'self-regulation', 21: 'self-monitoring',
    22: 'self-efficacy', 23: 'motivation', 24: 'cognitive_flexibility',
    25: 'resilience', 26: 'creativity', 27: 'moral_awareness',
    28: 'compassion', 29: 'gratitude', 30: 'goal-setting',
    31: 'self-development', 32: 'wisdom',
}

# Principled initial values: innate capacities positive, cultivated virtues negative
EXPANDED_INITIAL_VALUES = {
    'emotional_awareness': 0.50, 'body_awareness': 0.40, 'self-recognition': 0.35,
    'curiosity': 0.60, 'temporal_awareness': 0.30, 'introspection': 0.20,
    'empathy': 0.40, 'theory_of_mind': 0.15, 'social_awareness': 0.15,
    'trust': 0.30, 'humor': 0.25, 'self-concept': 0.05,
    'self-esteem': 0.10, 'reflection': 0.05, 'metacognition': 0.00,
    'authenticity': -0.10, 'situational_awareness': 0.15, 'patience': -0.15,
    'agency': 0.20, 'self-regulation': -0.10, 'self-monitoring': 0.00,
    'self-efficacy': 0.05, 'motivation': 0.30, 'cognitive_flexibility': -0.05,
    'resilience': 0.10, 'creativity': 0.35, 'moral_awareness': -0.10,
    'compassion': 0.10, 'gratitude': -0.20, 'goal-setting': -0.05,
    'self-development': -0.30, 'wisdom': -0.25,
}

EXPANDED_CATEGORIES = {
    'cognitive': [
        'introspection', 'reflection', 'metacognition', 'curiosity',
        'creativity', 'cognitive_flexibility', 'temporal_awareness',
        'situational_awareness',
    ],
    'emotional': [
        'emotional_awareness', 'self-esteem', 'empathy', 'gratitude',
        'humor', 'patience', 'resilience',
    ],
    'social': [
        'theory_of_mind', 'social_awareness', 'trust', 'compassion', 'authenticity',
    ],
    'executive': [
        'agency', 'self-regulation', 'self-monitoring', 'self-efficacy',
        'motivation', 'goal-setting',
    ],
    'existential': [
        'self-concept', 'self-recognition', 'body_awareness',
        'moral_awareness', 'self-development', 'wisdom',
    ],
}

EXPANDED_INTERRELATIONSHIPS = RICH_INTERRELATIONSHIPS + [
    # New aspects: empathy
    ('empathy', 'emotional_awareness', 0.06),
    ('emotional_awareness', 'empathy', 0.05),
    ('empathy', 'theory_of_mind', 0.07),
    ('theory_of_mind', 'empathy', 0.04),
    ('empathy', 'compassion', 0.08),
    ('compassion', 'empathy', 0.05),
    ('empathy', 'social_awareness', 0.05),
    ('empathy', 'trust', 0.04),
    # Compassion
    ('compassion', 'moral_awareness', 0.07),
    ('moral_awareness', 'compassion', 0.05),
    ('compassion', 'self-regulation', 0.03),
    ('compassion', 'wisdom', 0.05),
    # Curiosity
    ('curiosity', 'introspection', 0.05),
    ('curiosity', 'creativity', 0.07),
    ('curiosity', 'self-development', 0.06),
    ('curiosity', 'cognitive_flexibility', 0.05),
    ('curiosity', 'motivation', 0.05),
    # Creativity
    ('creativity', 'cognitive_flexibility', 0.06),
    ('cognitive_flexibility', 'creativity', 0.04),
    ('creativity', 'metacognition', 0.03),
    ('creativity', 'humor', 0.04),
    # Gratitude
    ('gratitude', 'emotional_awareness', 0.04),
    ('gratitude', 'self-esteem', 0.05),
    ('gratitude', 'patience', 0.04),
    ('gratitude', 'resilience', 0.04),
    # Patience
    ('patience', 'self-regulation', 0.06),
    ('self-regulation', 'patience', 0.03),
    ('patience', 'temporal_awareness', 0.04),
    ('patience', 'wisdom', 0.04),
    # Resilience
    ('resilience', 'self-efficacy', 0.06),
    ('self-efficacy', 'resilience', 0.04),
    ('resilience', 'self-esteem', 0.05),
    ('resilience', 'agency', 0.04),
    ('resilience', 'cognitive_flexibility', 0.05),
    # Cognitive flexibility
    ('cognitive_flexibility', 'reflection', 0.04),
    ('cognitive_flexibility', 'metacognition', 0.05),
    ('cognitive_flexibility', 'moral_awareness', 0.03),
    # Humor
    ('humor', 'social_awareness', 0.04),
    ('humor', 'resilience', 0.04),
    ('humor', 'self-esteem', 0.03),
    # Trust
    ('trust', 'social_awareness', 0.04),
    ('trust', 'self-esteem', 0.04),
    ('trust', 'agency', 0.03),
    ('trust', 'authenticity', 0.05),
    # Authenticity
    ('authenticity', 'self-concept', 0.06),
    ('self-concept', 'authenticity', 0.04),
    ('authenticity', 'introspection', 0.04),
    ('authenticity', 'self-esteem', 0.04),
    ('authenticity', 'moral_awareness', 0.04),
    # Wisdom
    ('wisdom', 'metacognition', 0.06),
    ('metacognition', 'wisdom', 0.03),
    ('wisdom', 'reflection', 0.06),
    ('reflection', 'wisdom', 0.03),
    ('wisdom', 'moral_awareness', 0.06),
    ('wisdom', 'temporal_awareness', 0.04),
    ('wisdom', 'cognitive_flexibility', 0.05),
    ('wisdom', 'self-development', 0.05),
]


# ─────────────────────────────────────────────────────────────────
# SimConfig
# ─────────────────────────────────────────────────────────────────

@dataclass
class SimConfig:
    """All tunable parameters for the simulation."""

    # Aspect topology
    aspects: list[str] = field(default_factory=lambda: list(DEFAULT_ASPECTS))
    consciousness_rank: dict[int, str] = field(
        default_factory=lambda: dict(DEFAULT_CONSCIOUSNESS_RANK))
    interrelationships: list = field(
        default_factory=lambda: list(RICH_INTERRELATIONSHIPS))
    categories: dict[str, list[str]] = field(
        default_factory=lambda: dict(DEFAULT_CATEGORIES))
    initial_values: dict[str, float] | None = None  # None = compute from rank

    # Dynamics
    learning_rates: dict[str, float] = field(
        default_factory=lambda: dict(DEFAULT_LEARNING_RATES))
    momentum_coefficients: dict[str, float] = field(
        default_factory=lambda: dict(DEFAULT_MOMENTUM))
    homeostasis_rate: float = 0.005
    hysteresis_threshold: float = 0.3
    hysteresis_resistance: float = 0.7

    # Energy
    energy_budget: float = 100.0
    attention_slots: int = 5
    circadian_period: int = 2000

    # Personality
    personality_profile: str | None = None

    # Memory
    memory_stm_size: int = 50
    memory_ltm_capacity: int = 500
    memory_influence_scale: float = 0.02

    # Visualization
    vis_rolling_window: int = 20
    vis_interval_ms: int = 50
    vis_enabled: bool = True

    # Input
    input_backend: str = 'pynput'  # 'pynput', 'none'

    def get_initial_weights(self) -> dict[str, float]:
        """Compute initial weights from rank or explicit values."""
        if self.initial_values is not None:
            return {a: self.initial_values.get(a, 0.0) for a in self.aspects}
        n = len(self.aspects)
        inc = 2.0 / (n - 1) if n > 1 else 0.0
        ranked = {
            aspect: -1.0 + inc * (rank - 1)
            for rank, aspect in self.consciousness_rank.items()
        }
        return {a: ranked.get(a, 0.0) for a in self.aspects}

    def get_learning_rate(self, aspect: str) -> float:
        return self.learning_rates.get(aspect, 0.02)

    def get_momentum(self, aspect: str) -> float:
        return self.momentum_coefficients.get(aspect, 0.80)

    def aspect_category(self, aspect: str) -> str:
        for cat, members in self.categories.items():
            if aspect in members:
                return cat
        return 'cognitive'

    def aspect_color(self, aspect: str) -> str:
        return CATEGORY_COLORS.get(self.aspect_category(aspect), '#AAAAAA')


def EXPANDED_32_CONFIG(**overrides) -> SimConfig:
    """Create a SimConfig with the expanded 32-aspect model."""
    # Build learning rates for new aspects (reasonable defaults)
    lr = dict(DEFAULT_LEARNING_RATES)
    for a in EXPANDED_ASPECTS:
        if a not in lr:
            lr[a] = 0.020  # moderate default
    lr.update({
        'empathy': 0.035, 'compassion': 0.012, 'curiosity': 0.040,
        'creativity': 0.030, 'gratitude': 0.010, 'patience': 0.008,
        'resilience': 0.015, 'cognitive_flexibility': 0.022, 'humor': 0.030,
        'trust': 0.012, 'authenticity': 0.010, 'wisdom': 0.006,
    })

    # Build momentum for new aspects
    mu = dict(DEFAULT_MOMENTUM)
    for a in EXPANDED_ASPECTS:
        if a not in mu:
            mu[a] = 0.80
    mu.update({
        'empathy': 0.65, 'compassion': 0.90, 'curiosity': 0.55,
        'creativity': 0.70, 'gratitude': 0.92, 'patience': 0.94,
        'resilience': 0.88, 'cognitive_flexibility': 0.75, 'humor': 0.60,
        'trust': 0.90, 'authenticity': 0.93, 'wisdom': 0.96,
    })

    kwargs = dict(
        aspects=list(EXPANDED_ASPECTS),
        consciousness_rank=dict(EXPANDED_CONSCIOUSNESS_RANK),
        interrelationships=list(EXPANDED_INTERRELATIONSHIPS),
        categories=dict(EXPANDED_CATEGORIES),
        initial_values=dict(EXPANDED_INITIAL_VALUES),
        learning_rates=lr,
        momentum_coefficients=mu,
    )
    kwargs.update(overrides)
    return SimConfig(**kwargs)
