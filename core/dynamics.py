"""Dynamics engine: momentum, adaptive learning rates, nonlinear bounding,
homeostasis, and hysteresis."""

from __future__ import annotations
import numpy as np
from .config import SimConfig


class DynamicsEngine:
    """Per-aspect weight update with psychologically-motivated dynamics.

    - Momentum: aspects carry trajectory (habits)
    - Adaptive LR: fast-changing (emotional) vs slow (identity)
    - tanh bounding: natural [-1, 1] saturation
    - Homeostasis: drift toward resting state without input
    - Hysteresis: sustained displacement resists snapping back
    """

    def __init__(self):
        self.velocity: dict[str, float] = {}
        self.conditioning: dict[str, float] = {}
        self.resting: dict[str, float] = {}
        self._lr: dict[str, float] = {}
        self._mu: dict[str, float] = {}
        self._homeostasis_rate: float = 0.005
        self._hysteresis_threshold: float = 0.3
        self._hysteresis_resistance: float = 0.7

    def init_state(self, aspects: list[str], resting_weights: dict[str, float],
                   config: SimConfig):
        self._lr = {a: config.get_learning_rate(a) for a in aspects}
        self._mu = {a: config.get_momentum(a) for a in aspects}
        self._homeostasis_rate = config.homeostasis_rate
        self._hysteresis_threshold = config.hysteresis_threshold
        self._hysteresis_resistance = config.hysteresis_resistance
        self.resting = dict(resting_weights)
        self.velocity = {a: 0.0 for a in aspects}
        self.conditioning = {a: 0.0 for a in aspects}

    def update(self, rebalanced: list[float], stimuli: list[float],
               aspects: list[str],
               energy_modifiers: dict[str, float] | None = None,
               personality_biases: dict[str, float] | None = None,
               memory_influences: dict[str, float] | None = None,
               ) -> list[float]:
        updated = []
        for i, aspect in enumerate(aspects):
            w = rebalanced[i]
            grad = stimuli[i]

            # Adaptive learning rate
            lr = self._lr.get(aspect, 0.02)
            if energy_modifiers and aspect in energy_modifiers:
                lr *= energy_modifiers[aspect]

            # Add personality bias to gradient
            if personality_biases and aspect in personality_biases:
                grad += personality_biases[aspect]

            # Add memory influence to gradient
            if memory_influences and aspect in memory_influences:
                grad += memory_influences[aspect]

            # Momentum update
            mu = self._mu.get(aspect, 0.8)
            rest = self.resting.get(aspect, 0.0)
            v = mu * self.velocity.get(aspect, 0.0) + lr * grad
            self.velocity[aspect] = v
            w_new = w - v

            # Homeostasis: pull toward resting state
            displacement = w_new - rest
            pull = self._homeostasis_rate * displacement
            cond = self.conditioning.get(aspect, 0.0)
            if cond > self._hysteresis_threshold:
                resistance = min(self._hysteresis_resistance, cond / (cond + 1.0))
                pull *= (1.0 - resistance)
            w_new -= pull

            # Nonlinear bound via tanh
            w_new = float(np.tanh(w_new))

            # Conditioning update (habit formation)
            abs_disp = abs(w_new - rest)
            if abs_disp > self._hysteresis_threshold * 0.5:
                self.conditioning[aspect] = cond + 0.01 * abs_disp
            else:
                self.conditioning[aspect] = cond * 0.995

            updated.append(w_new)
        return updated

    def reset(self):
        for a in self.velocity:
            self.velocity[a] = 0.0
            self.conditioning[a] = 0.0
