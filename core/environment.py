"""Environment: structured stimuli, user input, environmental memory,
spontaneous internal events, and stimulus decay."""

from __future__ import annotations
import random
import numpy as np
from collections import deque


# ── Stimulus catalog ──────────────────────────────────────────────────────

STIMULUS_CATALOG = {
    'social_interaction': {
        'social_awareness': 0.25, 'theory_of_mind': 0.20,
        'emotional_awareness': 0.15, 'self-monitoring': 0.10,
    },
    'challenge': {
        'agency': 0.20, 'self-efficacy': 0.15, 'motivation': 0.20,
        'self-regulation': 0.10, 'situational_awareness': 0.10,
    },
    'threat': {
        'situational_awareness': 0.30, 'emotional_awareness': 0.25,
        'self-regulation': -0.20, 'agency': -0.10,
        'body_awareness': 0.15,
    },
    'reward': {
        'self-esteem': 0.25, 'motivation': 0.20, 'self-efficacy': 0.20,
        'agency': 0.15,
    },
    'loss': {
        'emotional_awareness': 0.30, 'self-esteem': -0.20,
        'motivation': -0.15, 'reflection': 0.15,
    },
    'novelty': {
        'situational_awareness': 0.20, 'temporal_awareness': 0.15,
        'metacognition': 0.10, 'introspection': 0.10,
    },
    'moral_dilemma': {
        'moral_awareness': 0.30, 'theory_of_mind': 0.20,
        'reflection': 0.20, 'introspection': 0.15,
        'self-regulation': 0.10,
    },
    'flow_state': {
        'agency': 0.20, 'self-efficacy': 0.20, 'motivation': 0.15,
        'metacognition': 0.15, 'self-monitoring': -0.10,
    },
    'social_rejection': {
        'self-esteem': -0.25, 'social_awareness': 0.20,
        'emotional_awareness': 0.25, 'self-concept': -0.10,
        'agency': -0.10,
    },
    'accomplishment': {
        'self-efficacy': 0.25, 'self-esteem': 0.20,
        'motivation': 0.15, 'self-development': 0.15,
        'agency': 0.15,
    },
}

INTERNAL_EVENTS = {
    'mind_wandering': {
        'introspection': 0.15, 'temporal_awareness': 0.10,
        'self-monitoring': -0.10, 'situational_awareness': -0.10,
    },
    'sudden_insight': {
        'metacognition': 0.25, 'introspection': 0.15,
        'self-efficacy': 0.10, 'motivation': 0.10,
    },
    'intrusive_thought': {
        'emotional_awareness': 0.20, 'self-regulation': 0.15,
        'introspection': 0.10, 'self-esteem': -0.10,
    },
    'self_doubt': {
        'self-esteem': -0.20, 'self-efficacy': -0.15,
        'self-concept': -0.10, 'introspection': 0.15,
    },
    'nostalgia': {
        'temporal_awareness': 0.20, 'emotional_awareness': 0.15,
        'reflection': 0.15, 'self-concept': 0.10,
    },
    'creative_impulse': {
        'metacognition': 0.15, 'motivation': 0.15,
        'agency': 0.10, 'introspection': 0.10,
    },
}

# User input: coherent positive/negative experience clusters
POSITIVE_INPUT_EFFECTS = {
    'agency': 0.30, 'self-esteem': 0.25, 'motivation': 0.25,
    'self-efficacy': 0.20, 'self-development': 0.10,
    'self-concept': 0.10, 'self-regulation': 0.05,
}

NEGATIVE_INPUT_EFFECTS = {
    'emotional_awareness': 0.30, 'situational_awareness': 0.20,
    'self-regulation': -0.15, 'self-esteem': -0.20,
    'agency': -0.15, 'motivation': -0.10,
}


class ActiveStimulus:
    """A decaying stimulus currently affecting the system."""
    __slots__ = ('effects', 'strength', 'name')

    def __init__(self, name: str, effects: dict[str, float], strength: float = 1.0):
        self.name = name
        self.effects = effects
        self.strength = strength

    def decay(self, rate: float = 0.12):
        self.strength *= (1.0 - rate)

    @property
    def alive(self) -> bool:
        return self.strength > 0.005


class Environment:
    """Rich environment model with structured stimuli, environmental memory,
    internal events, and stimulus decay.

    Call generate_stimuli() each tick to get per-aspect responses.
    Call get_user_input() + apply_input() for keyboard interaction.
    """

    def __init__(self, aspects: list[str], backend: str = 'pynput'):
        self.aspects = list(aspects)
        self.n = len(aspects)
        self._idx = {a: i for i, a in enumerate(aspects)}

        # Active decaying stimuli
        self._active: list[ActiveStimulus] = []

        # Environmental memory: [-1, 1] — negative=hostile, positive=safe
        self.valence_memory: float = 0.0
        self._valence_decay: float = 0.002

        # Stimulus probability ramps with iteration
        self._stimulus_prob_base: float = 0.03
        self._internal_event_prob: float = 0.08

        # Keyboard input
        self._pressed_keys: set[str] = set()
        self._listener = None
        self._setup_input(backend)

    def _setup_input(self, backend: str):
        if backend == 'none':
            return
        if backend == 'pynput':
            try:
                from pynput import keyboard as kb
                def on_press(key):
                    try:
                        if key == kb.Key.up:
                            self._pressed_keys.add('up')
                        elif key == kb.Key.down:
                            self._pressed_keys.add('down')
                    except AttributeError:
                        pass
                def on_release(key):
                    try:
                        if key == kb.Key.up:
                            self._pressed_keys.discard('up')
                        elif key == kb.Key.down:
                            self._pressed_keys.discard('down')
                    except AttributeError:
                        pass
                self._listener = kb.Listener(on_press=on_press, on_release=on_release)
                self._listener.daemon = True
                self._listener.start()
                return
            except ImportError:
                pass
        # Fallback
        self._pressed_keys = set()

    def generate_stimuli(self, weights: list[float], tick: int) -> list[float]:
        """Produce a per-aspect stimulus vector for this tick."""
        signal = np.zeros(self.n)

        # Random external stimulus
        prob = min(self._stimulus_prob_base + tick * 0.0002, 0.25)
        if random.random() < prob:
            name = random.choice(list(STIMULUS_CATALOG.keys()))
            effects = STIMULUS_CATALOG[name]
            self._active.append(ActiveStimulus(name, effects))

        # Spontaneous internal events (modulated by environmental safety)
        if random.random() < self._internal_event_prob:
            # Safe environments: more insight/creativity. Hostile: more intrusive/doubt
            if self.valence_memory > 0.2:
                pool = ['sudden_insight', 'creative_impulse', 'nostalgia']
            elif self.valence_memory < -0.2:
                pool = ['intrusive_thought', 'self_doubt', 'mind_wandering']
            else:
                pool = list(INTERNAL_EVENTS.keys())
            name = random.choice(pool)
            effects = INTERNAL_EVENTS[name]
            self._active.append(ActiveStimulus(name, effects, strength=0.7))

        # Aggregate all active stimuli
        for stim in self._active:
            for aspect, mag in stim.effects.items():
                if aspect in self._idx:
                    signal[self._idx[aspect]] += mag * stim.strength

        # Decay and prune
        for stim in self._active:
            stim.decay()
        self._active = [s for s in self._active if s.alive]

        # Background noise shaped by environmental memory
        noise_scale = 1.0
        if self.valence_memory > 0:
            noise_scale -= 0.4 * self.valence_memory  # safe = less noise
        elif self.valence_memory < 0:
            noise_scale += 0.6 * abs(self.valence_memory)  # hostile = more noise

        randomness = (1 - np.exp(-tick / 100)) * noise_scale
        noise = np.array([random.uniform(-randomness, randomness)
                          for _ in range(self.n)])
        signal += noise

        # Decay environmental memory toward neutral
        self.valence_memory *= (1.0 - self._valence_decay)

        return signal.tolist()

    def get_user_input(self) -> dict:
        if 'up' in self._pressed_keys:
            return {'active': True, 'direction': 'positive', 'text': 'positive input'}
        if 'down' in self._pressed_keys:
            return {'active': True, 'direction': 'negative', 'text': 'negative input'}
        return {'active': False, 'direction': None, 'text': None}

    def apply_input(self, weight_dict: dict[str, float],
                    user_input: dict) -> dict[str, float]:
        if not user_input.get('active'):
            return weight_dict

        if user_input['direction'] == 'positive':
            effects = POSITIVE_INPUT_EFFECTS
            self.valence_memory = min(1.0, self.valence_memory + 0.05)
        else:
            effects = NEGATIVE_INPUT_EFFECTS
            self.valence_memory = max(-1.0, self.valence_memory - 0.05)

        # Inject as decaying stimulus
        self._active.append(ActiveStimulus('user_input', effects, strength=1.0))
        return weight_dict

    def get_status(self) -> str:
        """Compact status string for visualization."""
        active_names = list({s.name for s in self._active if s.strength > 0.05})
        valence_str = f'valence={self.valence_memory:+.2f}'
        if active_names:
            return f'{", ".join(active_names[:3])} | {valence_str}'
        return valence_str

    def __del__(self):
        if self._listener is not None:
            try:
                self._listener.stop()
            except Exception:
                pass
