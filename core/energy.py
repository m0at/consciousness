"""Energy and arousal system: finite energy budget, arousal modulation,
selective attention, flow states, stress responses, circadian rhythm."""

from __future__ import annotations
import numpy as np
from collections import deque

# Aspects prioritized during stress
STRESS_PRIORITY = {'self-regulation', 'situational_awareness', 'agency', 'body_awareness'}

# Clusters for flow state detection
FLOW_CLUSTERS = {
    'executive': ['metacognition', 'self-regulation', 'self-monitoring', 'goal-setting'],
    'reflective': ['introspection', 'reflection', 'temporal_awareness', 'moral_awareness'],
    'social': ['theory_of_mind', 'social_awareness', 'emotional_awareness', 'situational_awareness'],
    'identity': ['self-concept', 'self-esteem', 'self-recognition', 'self-efficacy'],
    'drive': ['motivation', 'agency', 'goal-setting', 'self-development'],
}


class EnergySystem:
    """Resource-constrained modulation layer.

    Returns modifiers for learning rate, noise amplitude, and per-aspect
    coupling each tick.
    """

    def __init__(self, aspects: list[str], resting_weights: dict[str, float], *,
                 max_energy: float = 100.0, attention_slots: int = 5,
                 circadian_period: int = 2000):
        self.aspects = list(aspects)
        self.n = len(aspects)
        self.resting = np.array([resting_weights.get(a, 0.0) for a in aspects])
        self._idx = {a: i for i, a in enumerate(aspects)}

        self.max_energy = max_energy
        self.energy = max_energy
        self.energy_regen_rate = 0.15
        self.energy_cost_scale = 0.02

        self.arousal = 0.7
        self.arousal_baseline = 0.7
        self.arousal_decay = 0.02
        self.arousal_min = 0.05
        self.arousal_max = 2.0

        self.attention_slots = attention_slots
        self.attention = np.full(self.n, 0.15)
        self._prev_weights = None

        self.flow_active: dict[str, bool] = {}
        self.flow_timer = {c: 0 for c in FLOW_CLUSTERS}
        self.flow_threshold = 15
        self.flow_range = (0.2, 0.85)
        self.flow_stability_band = 0.15

        self.stress_level = 0.0
        self.stress_decay = 0.03
        self.stress_threshold = 0.35

        self.circadian_period = circadian_period
        self.iteration = 0

    def step(self, weight_dict: dict[str, float]) -> dict:
        """Advance one tick. Returns {lr_modifier, noise_modifier, aspect_scales}."""
        weights = np.array([weight_dict.get(a, 0.0) for a in self.aspects])
        displacement = np.abs(weights - self.resting)

        roc = np.abs(weights - self._prev_weights) if self._prev_weights is not None \
            else np.zeros(self.n)
        self._prev_weights = weights.copy()

        self._update_energy(displacement)
        self._update_stress(displacement, roc)
        self._update_arousal()
        self._update_attention(displacement, roc)
        self._update_flow(weights)

        circadian = self._circadian_factor()
        energy_frac = self.energy / self.max_energy

        lr_mod = self.arousal * max(energy_frac, 0.1) * circadian

        noise_mod = self.arousal * circadian
        if self.stress_level > 0.3:
            noise_mod *= (1.0 + 0.5 * self.stress_level)
        if any(self.flow_active.get(c, False) for c in FLOW_CLUSTERS):
            noise_mod *= 0.7

        aspect_scales = {}
        for i, a in enumerate(self.aspects):
            scale = self.attention[i]
            if energy_frac < 0.3:
                scale *= 0.3 + 0.7 * (energy_frac / 0.3)
            if self.stress_level > 0.3 and a in STRESS_PRIORITY:
                scale *= (1.0 + self.stress_level)
            for cname, members in FLOW_CLUSTERS.items():
                if self.flow_active.get(cname, False) and a in members:
                    scale *= 1.3
                    break
            aspect_scales[a] = float(np.clip(scale, 0.05, 3.0))

        self.iteration += 1

        return {
            'lr_modifier': float(lr_mod),
            'noise_modifier': float(noise_mod),
            'aspect_scales': aspect_scales,
        }

    def get_state(self) -> dict:
        attended = [self.aspects[i] for i in np.argsort(self.attention)[-self.attention_slots:]]
        return {
            'energy': round(self.energy, 2),
            'energy_pct': round(100 * self.energy / self.max_energy, 1),
            'arousal': round(self.arousal, 4),
            'stress': round(self.stress_level, 4),
            'attended': attended,
            'flow_states': [c for c, v in self.flow_active.items() if v],
            'circadian': round(self._circadian_factor(), 3),
        }

    def inject_perturbation(self, magnitude=0.8):
        self.stress_level = min(1.0, self.stress_level + min(1.0, magnitude))
        self.arousal = min(self.arousal_max, self.arousal + min(1.0, magnitude) * 0.6)

    # ── Internal ──

    def _update_energy(self, displacement):
        cost = self.energy_cost_scale * np.sum(displacement ** 2) * (1.0 + self.stress_level)
        self.energy -= cost
        mean_disp = np.mean(displacement)
        if mean_disp < 0.25:
            self.energy += self.energy_regen_rate * (1.0 - mean_disp / 0.25)
        self.energy = float(np.clip(self.energy, 0.0, self.max_energy))

    def _update_arousal(self):
        target = self.arousal_baseline
        if self.stress_level > 0.2:
            target += self.stress_level * 0.8
        energy_frac = self.energy / self.max_energy
        if energy_frac < 0.3:
            target *= energy_frac / 0.3
        target *= self._circadian_factor()
        self.arousal += self.arousal_decay * (target - self.arousal)
        self.arousal = float(np.clip(self.arousal, self.arousal_min, self.arousal_max))

    def _update_attention(self, displacement, roc):
        salience = 0.6 * roc + 0.4 * displacement
        if self.stress_level > 0.3:
            for a in STRESS_PRIORITY:
                if a in self._idx:
                    salience[self._idx[a]] += self.stress_level * 0.5
        top = set(np.argsort(salience)[-self.attention_slots:])
        for i in range(self.n):
            if i in top:
                self.attention[i] += 0.15 * (1.0 - self.attention[i])
            else:
                self.attention[i] += 0.08 * (0.15 - self.attention[i])
        self.attention = np.clip(self.attention, 0.05, 1.0)

    def _update_stress(self, displacement, roc):
        perturbation = max(np.max(roc), np.max(displacement) * 0.3)
        if perturbation > self.stress_threshold:
            spike = min(1.0, (perturbation - self.stress_threshold) / self.stress_threshold)
            self.stress_level = min(1.0, self.stress_level + spike * 0.4)
        self.stress_level *= (1.0 - self.stress_decay)
        self.stress_level = float(np.clip(self.stress_level, 0.0, 1.0))

    def _update_flow(self, weights):
        for cname, members in FLOW_CLUSTERS.items():
            indices = [self._idx[a] for a in members if a in self._idx]
            if not indices:
                continue
            cluster_abs = np.abs(weights[indices] - self.resting[indices])
            lo, hi = self.flow_range
            in_range = np.all((cluster_abs >= lo) & (cluster_abs <= hi))
            stable = np.std(cluster_abs) < self.flow_stability_band
            if in_range and stable and self.stress_level < 0.3:
                self.flow_timer[cname] += 1
            else:
                self.flow_timer[cname] = max(0, self.flow_timer[cname] - 2)
            self.flow_active[cname] = self.flow_timer[cname] >= self.flow_threshold

    def _circadian_factor(self):
        if self.circadian_period <= 0:
            return 1.0
        phase = 2.0 * np.pi * self.iteration / self.circadian_period
        return 0.75 + 0.25 * np.cos(phase)
