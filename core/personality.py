"""Personality/bias system: attractor-based personality profiles that
create behavioral tendencies which can be influenced through sustained input."""

from __future__ import annotations
import numpy as np
from copy import deepcopy
from dataclasses import dataclass


@dataclass
class PersonalityProfile:
    """A named set of bias vectors with per-aspect attractor strength."""
    name: str
    description: str
    bias_targets: dict[str, float]   # aspect -> attractor target
    rigidity: dict[str, float]       # aspect -> 0-1 attractor depth


# ── Predefined personality archetypes ─────────────────────────────────────

CONTEMPLATIVE = PersonalityProfile(
    name='contemplative',
    description='Drawn to introspection, reflection, and metacognitive awareness.',
    bias_targets={
        'introspection': 0.7, 'reflection': 0.7, 'metacognition': 0.6,
        'self-monitoring': 0.5, 'temporal_awareness': 0.4, 'moral_awareness': 0.4,
        'self-concept': 0.3, 'agency': -0.1, 'motivation': -0.1, 'goal-setting': -0.2,
    },
    rigidity={
        'introspection': 0.7, 'reflection': 0.7, 'metacognition': 0.65,
        'self-monitoring': 0.5, 'agency': 0.3,
    },
)

ACTION_ORIENTED = PersonalityProfile(
    name='action-oriented',
    description='Biased toward agency, goal-setting, and self-efficacy.',
    bias_targets={
        'agency': 0.7, 'motivation': 0.7, 'goal-setting': 0.6,
        'self-efficacy': 0.6, 'situational_awareness': 0.5,
        'self-regulation': 0.4, 'self-development': 0.4,
        'introspection': -0.2, 'reflection': -0.2, 'metacognition': -0.1,
    },
    rigidity={
        'agency': 0.7, 'motivation': 0.7, 'goal-setting': 0.65,
        'self-efficacy': 0.6, 'introspection': 0.3,
    },
)

EMPATHIC = PersonalityProfile(
    name='empathic',
    description='Strong pull toward social/emotional awareness and moral sensitivity.',
    bias_targets={
        'emotional_awareness': 0.7, 'social_awareness': 0.7,
        'theory_of_mind': 0.6, 'moral_awareness': 0.6,
        'self-esteem': 0.3, 'self-regulation': 0.3, 'situational_awareness': 0.4,
        'introspection': -0.1, 'self-concept': -0.1,
    },
    rigidity={
        'emotional_awareness': 0.75, 'social_awareness': 0.7,
        'theory_of_mind': 0.65, 'moral_awareness': 0.6,
    },
)

ANALYTICAL = PersonalityProfile(
    name='analytical',
    description='Biased toward metacognition, self-monitoring, and systematic analysis.',
    bias_targets={
        'metacognition': 0.7, 'self-monitoring': 0.7, 'situational_awareness': 0.6,
        'self-regulation': 0.5, 'temporal_awareness': 0.4, 'introspection': 0.3,
        'emotional_awareness': -0.2, 'social_awareness': -0.1, 'self-esteem': -0.1,
    },
    rigidity={
        'metacognition': 0.75, 'self-monitoring': 0.7,
        'situational_awareness': 0.65, 'self-regulation': 0.55,
        'emotional_awareness': 0.35,
    },
)

RESILIENT = PersonalityProfile(
    name='resilient',
    description='Strong self-regulation and self-esteem core. Hard to destabilize.',
    bias_targets={
        'self-regulation': 0.7, 'self-esteem': 0.6, 'self-concept': 0.6,
        'self-efficacy': 0.5, 'self-recognition': 0.4, 'agency': 0.4,
        'self-development': 0.3, 'moral_awareness': 0.3,
    },
    rigidity={
        'self-regulation': 0.8, 'self-esteem': 0.8, 'self-concept': 0.8,
        'self-efficacy': 0.65, 'self-recognition': 0.6, 'agency': 0.55,
    },
)

SEEKER = PersonalityProfile(
    name='seeker',
    description='Driven by self-development and growth. Low rigidity, explores freely.',
    bias_targets={
        'self-development': 0.7, 'motivation': 0.5, 'goal-setting': 0.5,
        'introspection': 0.4, 'metacognition': 0.4, 'reflection': 0.3, 'agency': 0.3,
    },
    rigidity={
        'self-development': 0.5, 'motivation': 0.4, 'goal-setting': 0.4,
        'introspection': 0.35, 'metacognition': 0.35, 'reflection': 0.3, 'agency': 0.3,
    },
)

PROFILES = {
    'contemplative': CONTEMPLATIVE, 'action-oriented': ACTION_ORIENTED,
    'empathic': EMPATHIC, 'analytical': ANALYTICAL,
    'resilient': RESILIENT, 'seeker': SEEKER,
}


class PersonalitySystem:
    """Attractor-based personality layer.

    Biases act as persistent background forces pulling weights toward
    preferred states. Sustained contrary input slowly evolves bias targets
    (personality growth) and injects volatility (internal conflict).
    """

    def __init__(self, profile: PersonalityProfile | str,
                 aspects: list[str],
                 evolution_rate: float = 0.0003,
                 conflict_gain: float = 2.0):
        if isinstance(profile, str):
            profile = PROFILES[profile]

        self.profile = profile
        self.aspects = list(aspects)
        self.evolution_rate = evolution_rate
        self.conflict_gain = conflict_gain

        # Default rigidity for aspects not in profile
        default_rig = 0.5

        # Live bias targets (evolve over time)
        self.bias_targets = {a: profile.bias_targets.get(a, 0.0) for a in aspects}
        self.rigidity = {a: profile.rigidity.get(a, default_rig) for a in aspects}
        self._original_targets = dict(self.bias_targets)

        # Per-aspect tracking
        self.contrary_pressure = {a: 0.0 for a in aspects}
        self.conflict_level = {a: 0.0 for a in aspects}
        self.volatility_injection = {a: 0.0 for a in aspects}

    def compute_biases(self, weight_dict: dict[str, float]) -> dict[str, float]:
        """Return per-aspect attractor pull forces."""
        biases = {}
        for a in self.aspects:
            w = weight_dict.get(a, 0.0)
            target = self.bias_targets.get(a, 0.0)
            rig = self.rigidity.get(a, 0.5)
            biases[a] = rig * 0.05 * (target - w)
        return biases

    def register_input(self, weight_dict: dict[str, float],
                       responses: list[float]):
        """Track contrary pressure, conflict, and evolve biases."""
        for i, a in enumerate(self.aspects):
            w = weight_dict.get(a, 0.0)
            resp = responses[i] if i < len(responses) else 0.0
            target = self.bias_targets.get(a, 0.0)
            rig = self.rigidity.get(a, 0.5)

            direction_to_target = np.sign(target - w) if abs(target - w) > 0.01 else 0.0
            response_direction = np.sign(resp)

            is_contrary = (direction_to_target != 0.0 and response_direction != 0.0
                           and direction_to_target != response_direction)

            if is_contrary:
                self.contrary_pressure[a] += abs(resp)
                conflict_delta = abs(resp) * rig * self.conflict_gain
                self.conflict_level[a] = min(
                    1.0, self.conflict_level[a] * 0.95 + conflict_delta * 0.1)
                self.volatility_injection[a] = self.conflict_level[a] * 0.02
            else:
                self.contrary_pressure[a] *= 0.98
                self.conflict_level[a] *= 0.92
                self.volatility_injection[a] *= 0.9

            # Bias evolution
            threshold = rig * 10.0
            if self.contrary_pressure[a] > threshold:
                shift = self.evolution_rate * (w - target) * (1.0 - rig * 0.5)
                self.bias_targets[a] += shift
                self.rigidity[a] = max(0.1, self.rigidity[a] - 0.00005)

    def apply_conflict_volatility(self, responses: list[float]) -> list[float]:
        """Inject extra noise on aspects experiencing internal conflict."""
        modified = []
        for i, a in enumerate(self.aspects):
            vol = self.volatility_injection.get(a, 0.0)
            r = responses[i] if i < len(responses) else 0.0
            if vol > 0.001:
                r += np.random.normal(0, vol)
            modified.append(r)
        return modified

    def get_visualization_data(self, weight_dict: dict[str, float]) -> dict:
        displacement = {}
        bias_drift = {}
        for a in self.aspects:
            w = weight_dict.get(a, 0.0)
            displacement[a] = w - self.bias_targets.get(a, 0.0)
            bias_drift[a] = self.bias_targets.get(a, 0.0) - self._original_targets.get(a, 0.0)
        return {
            'personality_name': self.profile.name,
            'bias_targets': dict(self.bias_targets),
            'displacement': displacement,
            'rigidity': dict(self.rigidity),
            'conflict_level': dict(self.conflict_level),
            'bias_drift': bias_drift,
        }

    @staticmethod
    def blend(profile_a: PersonalityProfile, profile_b: PersonalityProfile,
              mix: float = 0.5, aspects: list[str] | None = None) -> PersonalityProfile:
        """Create blended personality profile."""
        all_aspects = set(list(profile_a.bias_targets.keys()) +
                          list(profile_b.bias_targets.keys()))
        if aspects:
            all_aspects = all_aspects.intersection(aspects)
        targets = {a: (1 - mix) * profile_a.bias_targets.get(a, 0.0) +
                      mix * profile_b.bias_targets.get(a, 0.0) for a in all_aspects}
        rigidity = {a: (1 - mix) * profile_a.rigidity.get(a, 0.5) +
                       mix * profile_b.rigidity.get(a, 0.5) for a in all_aspects}
        return PersonalityProfile(
            name=f'{profile_a.name}/{profile_b.name}({mix:.2f})',
            description=f'Blend of {profile_a.name} and {profile_b.name}',
            bias_targets=targets, rigidity=rigidity)
