"""Emergent behavior detection: phases, oscillations, cascades,
attractors, resilience, and entropy."""

import numpy as np
from collections import deque
from dataclasses import dataclass
from typing import Optional


@dataclass
class PhaseState:
    phase: str          # "growth", "stability", "crisis", "recovery"
    aspect: str
    confidence: float
    duration: int
    slope: float


@dataclass
class OscillationState:
    aspect: str
    pattern: str        # "oscillating", "converging", "diverging"
    frequency: float
    amplitude: float
    damping: float


@dataclass
class Cascade:
    trigger_aspect: str
    trigger_delta: float
    path: list
    total_magnitude: float
    timestamp: int


@dataclass
class Attractor:
    center: dict
    basin_radius: float
    strength: float
    age: int
    drift_rate: float


@dataclass
class EntropyState:
    shannon: float
    normalized: float
    delta: float
    complexity_label: str


def _linreg_slope(x, y):
    n = len(x)
    if n < 2:
        return 0.0
    sx, sy = np.sum(x), np.sum(y)
    sxy, sxx = np.sum(x * y), np.sum(x * x)
    denom = n * sxx - sx * sx
    if abs(denom) < 1e-15:
        return 0.0
    return float((n * sxy - sx * sy) / denom)


class SystemAnalyzer:
    """Emergent behavior detection for the consciousness weight system."""

    def __init__(self, aspects: list[str], interrelations: np.ndarray,
                 window: int = 50, cascade_threshold: float = 0.02,
                 stability_threshold: float = 0.005):
        self.aspects = list(aspects)
        self.n = len(aspects)
        self.aspect_idx = {a: i for i, a in enumerate(aspects)}
        self.interrelations = np.array(interrelations)
        self.window = window
        self.cascade_threshold = cascade_threshold
        self.stability_threshold = stability_threshold

        self.history: deque = deque(maxlen=max(window * 4, 200))
        self.tick = 0

        self._phase = {a: 'stability' for a in aspects}
        self._phase_duration = {a: 0 for a in aspects}
        self._prev_weights = None
        self._cascade_log: list[Cascade] = []
        self._attractors: list[Attractor] = []
        self._perturbation_snapshot: Optional[np.ndarray] = None
        self._perturbation_tick: Optional[int] = None
        self._resilience_log = []
        self._entropy_history: deque = deque(maxlen=window * 2)

    def tick_update(self, weight_dict: dict) -> dict:
        """Feed new weights. Returns detection summary."""
        weights = np.array([weight_dict[a] for a in self.aspects])
        self.history.append(weights)
        self.tick += 1

        results = {}
        if len(self.history) >= 3:
            results['phases'] = self.detect_phases()
            results['oscillations'] = self.detect_oscillations()
            results['cascades'] = self.detect_cascades(weights)
            results['entropy'] = self.compute_entropy(weights)

        if len(self.history) >= self.window:
            results['attractors'] = self.detect_attractors()

        if self._perturbation_snapshot is not None:
            results['resilience'] = self.measure_resilience(weights)

        self._prev_weights = weights.copy()
        return results

    def mark_perturbation(self):
        if len(self.history) >= 2:
            self._perturbation_snapshot = np.array(list(self.history)[-2]).copy()
            self._perturbation_tick = self.tick

    def detect_phases(self) -> list[PhaseState]:
        arr = self._recent_array()
        if arr.shape[0] < 5:
            return []

        results = []
        lookback = min(arr.shape[0], self.window)
        recent = arr[-lookback:]
        t = np.arange(lookback, dtype=float)

        for i, aspect in enumerate(self.aspects):
            series = recent[:, i]
            slope = _linreg_slope(t, series)
            diffs = np.diff(series)
            volatility = np.std(diffs) if len(diffs) > 1 else 0.0
            mean_abs = np.mean(np.abs(series[-10:])) + 1e-12
            rel_vol = volatility / mean_abs

            prev_phase = self._phase[aspect]
            if rel_vol > 0.15:
                phase = 'crisis'
                confidence = min(1.0, rel_vol / 0.3)
            elif abs(slope) > self.stability_threshold * 2:
                phase = 'recovery' if prev_phase == 'crisis' else 'growth'
                confidence = min(1.0, abs(slope) / (self.stability_threshold * 6))
            else:
                phase = 'stability'
                confidence = 1.0 - abs(slope) / (self.stability_threshold * 2 + 1e-12)

            if phase == prev_phase:
                self._phase_duration[aspect] += 1
            else:
                self._phase_duration[aspect] = 1
            self._phase[aspect] = phase

            results.append(PhaseState(
                phase=phase, aspect=aspect,
                confidence=max(0.0, min(1.0, confidence)),
                duration=self._phase_duration[aspect], slope=float(slope)))
        return results

    def detect_oscillations(self) -> list[OscillationState]:
        arr = self._recent_array()
        if arr.shape[0] < 8:
            return []

        lookback = min(arr.shape[0], self.window)
        recent = arr[-lookback:]
        results = []

        for i, aspect in enumerate(self.aspects):
            series = recent[:, i]
            diffs = np.diff(series)
            signs = np.sign(diffs)
            sign_changes = np.sum(signs[1:] != signs[:-1])
            freq = sign_changes / max(len(signs) - 1, 1)
            amplitude = float(np.max(series) - np.min(series))
            half = len(series) // 2
            amp_first = np.ptp(series[:half]) if half > 1 else amplitude
            amp_second = np.ptp(series[half:]) if half > 1 else amplitude
            damping = float(amp_second - amp_first)

            if freq > 0.4 and amplitude > self.stability_threshold:
                if damping < -self.stability_threshold:
                    pattern = 'converging'
                elif damping > self.stability_threshold:
                    pattern = 'diverging'
                else:
                    pattern = 'oscillating'
            elif amplitude < self.stability_threshold * 2:
                pattern = 'converging'
            else:
                pattern = 'converging' if damping < 0 else 'diverging'

            results.append(OscillationState(
                aspect=aspect, pattern=pattern, frequency=float(freq),
                amplitude=amplitude, damping=float(damping)))
        return results

    def detect_cascades(self, weights: np.ndarray) -> list[Cascade]:
        if self._prev_weights is None:
            self._prev_weights = weights.copy()
            return []

        deltas = weights - self._prev_weights
        abs_deltas = np.abs(deltas)
        triggers = np.where(abs_deltas > self.cascade_threshold)[0]
        if len(triggers) == 0:
            return []

        new_cascades = []
        for trig_idx in triggers:
            connections = self.interrelations[trig_idx]
            connected = np.where((connections > 0) & (np.arange(self.n) != trig_idx))[0]
            path = []
            for conn_idx in connected:
                conn_delta = float(deltas[conn_idx])
                if abs(conn_delta) > self.stability_threshold:
                    path.append((self.aspects[conn_idx], conn_delta, 0))
            for conn_idx in connected:
                second_conns = self.interrelations[conn_idx]
                sc_indices = np.where(
                    (second_conns > 0) & (np.arange(self.n) != conn_idx) &
                    (np.arange(self.n) != trig_idx))[0]
                for sc_idx in sc_indices:
                    sc_delta = float(deltas[sc_idx])
                    if abs(sc_delta) > self.stability_threshold * 0.5:
                        path.append((self.aspects[sc_idx], sc_delta, 1))

            if path:
                cascade = Cascade(
                    trigger_aspect=self.aspects[trig_idx],
                    trigger_delta=float(deltas[trig_idx]),
                    path=path, total_magnitude=sum(abs(d) for _, d, _ in path),
                    timestamp=self.tick)
                new_cascades.append(cascade)
                self._cascade_log.append(cascade)
        return new_cascades

    def detect_attractors(self) -> list[Attractor]:
        arr = self._recent_array()
        if arr.shape[0] < self.window:
            return self._attractors

        recent = arr[-self.window:]
        center = np.mean(recent, axis=0)
        distances = np.linalg.norm(recent - center, axis=1)
        basin_radius = float(np.mean(distances))

        if len(distances) > 10:
            d_centered = distances - np.mean(distances)
            var = np.var(d_centered)
            if var > 1e-12:
                ac = np.correlate(d_centered[:-1], d_centered[1:])[0]
                ac /= (var * (len(d_centered) - 1))
                strength = float(-np.log(max(abs(ac), 1e-6)))
            else:
                strength = 10.0
        else:
            strength = 0.0

        half = self.window // 2
        c_first = np.mean(recent[:half], axis=0)
        c_second = np.mean(recent[half:], axis=0)
        drift_rate = float(np.linalg.norm(c_second - c_first) / half)

        center_dict = {a: float(center[i]) for i, a in enumerate(self.aspects)}
        new_att = Attractor(center=center_dict, basin_radius=basin_radius,
                            strength=strength, age=self.tick, drift_rate=drift_rate)

        if self._attractors:
            old_c = np.array([self._attractors[-1].center[a] for a in self.aspects])
            if np.linalg.norm(center - old_c) < basin_radius * 2:
                new_att.age = self._attractors[-1].age
                self._attractors[-1] = new_att
            else:
                self._attractors.append(new_att)
                if len(self._attractors) > 10:
                    self._attractors = self._attractors[-10:]
        else:
            self._attractors.append(new_att)
        return self._attractors

    def measure_resilience(self, weights):
        if self._perturbation_snapshot is None:
            return None
        baseline = self._perturbation_snapshot
        disp_vec = weights - baseline
        displacement = float(np.linalg.norm(disp_vec))
        elapsed = self.tick - self._perturbation_tick
        recovered = displacement < self.stability_threshold * self.n * 0.5

        if recovered or elapsed > self.window * 2:
            self._perturbation_snapshot = None
            self._perturbation_tick = None

        elasticity = float(np.exp(-displacement / (self.stability_threshold * self.n + 1e-12)))
        return {'displacement': displacement, 'elapsed': elapsed,
                'elasticity': elasticity, 'recovered': recovered}

    def compute_entropy(self, weights: np.ndarray) -> EntropyState:
        abs_w = np.abs(weights)
        total = np.sum(abs_w)
        if total < 1e-12:
            return EntropyState(0.0, 0.0, 0.0, 'settled')

        probs = np.clip(abs_w / total, 1e-12, None)
        shannon = float(-np.sum(probs * np.log2(probs)))
        max_ent = np.log2(self.n)
        normalized = shannon / max_ent if max_ent > 0 else 0.0

        prev = self._entropy_history[-1] if self._entropy_history else shannon
        self._entropy_history.append(shannon)

        if normalized > 0.92:
            label = 'turbulent'
        elif normalized > 0.75:
            label = 'active'
        else:
            label = 'settled'

        return EntropyState(shannon=shannon, normalized=normalized,
                            delta=shannon - prev, complexity_label=label)

    def summary(self) -> dict:
        if len(self.history) < 3:
            return {'status': 'warming up', 'ticks': self.tick}

        weights = np.array(list(self.history)[-1])
        phases = self.detect_phases()
        entropy = self.compute_entropy(weights)

        phase_counts = {}
        for p in phases:
            phase_counts[p.phase] = phase_counts.get(p.phase, 0) + 1

        return {
            'tick': self.tick,
            'dominant_phase': max(phase_counts, key=phase_counts.get) if phase_counts else 'unknown',
            'entropy': {'normalized': round(entropy.normalized, 4),
                        'label': entropy.complexity_label},
            'attractors_found': len(self._attractors),
            'cascades_total': len(self._cascade_log),
        }

    def _recent_array(self) -> np.ndarray:
        if not self.history:
            return np.empty((0, self.n))
        return np.array(list(self.history))
