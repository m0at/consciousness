"""Memory and temporal processing: short-term buffer, long-term memory
formation, emotional tagging, decay, cycle detection, history compression."""

from __future__ import annotations
import time
import numpy as np
from collections import deque


class Memory:
    """A single long-term memory."""
    __slots__ = ('timestamp', 'frame', 'state', 'intensity', 'valence',
                 'emotion', 'reinforcement_count', 'last_accessed')

    def __init__(self, timestamp, frame, state, intensity, valence, emotion):
        self.timestamp = timestamp
        self.frame = frame
        self.state = np.array(state, dtype=np.float32)
        self.intensity = float(intensity)
        self.valence = float(valence)
        self.emotion = str(emotion)
        self.reinforcement_count = 0
        self.last_accessed = frame

    def similarity(self, state_vec):
        dot = np.dot(self.state, state_vec)
        norm_a = np.linalg.norm(self.state)
        norm_b = np.linalg.norm(state_vec)
        if norm_a < 1e-9 or norm_b < 1e-9:
            return 0.0
        return float(dot / (norm_a * norm_b))

    def effective_strength(self, current_frame, half_life=2000.0):
        age = max(current_frame - self.frame, 1)
        decay = 2.0 ** (-age / half_life)
        emotion_mult = 1.0 + 0.5 * abs(self.valence)
        reinforce_mult = 1.0 + 0.1 * self.reinforcement_count
        return self.intensity * decay * emotion_mult * reinforce_mult


class HistoryBucket:
    """Compressed summary of a range of frames."""
    __slots__ = ('frame_start', 'frame_end', 'count', 'mean_state',
                 'var_state', 'mean_valence', 'max_intensity')

    def __init__(self, frame_start, frame_end, count, mean_state,
                 var_state, mean_valence, max_intensity):
        self.frame_start = frame_start
        self.frame_end = frame_end
        self.count = count
        self.mean_state = mean_state
        self.var_state = var_state
        self.mean_valence = mean_valence
        self.max_intensity = max_intensity


class MemorySystem:
    """Memory and temporal processing layer.

    Call process_frame() each tick with current weights and responses.
    Returns a bias vector to add to weights.
    """

    def __init__(self, aspects: list[str], *, stm_size: int = 50,
                 ltm_capacity: int = 500, compression_interval: int = 500,
                 influence_scale: float = 0.02, decay_half_life: float = 2000.0):
        self.aspects = list(aspects)
        self.n = len(aspects)
        self._idx = {a: i for i, a in enumerate(aspects)}

        self.stm: deque = deque(maxlen=stm_size)
        self.memories: list[Memory] = []
        self.ltm_capacity = ltm_capacity
        self.influence_scale = influence_scale
        self.decay_half_life = decay_half_life

        self._raw_history: list = []
        self._compressed: list[HistoryBucket] = []
        self._last_compress_frame = 0
        self.compression_interval = compression_interval

        self._cycle_accum = np.zeros(self.n)
        self._cycle_counts = np.zeros(self.n, dtype=int)
        self._detected_periods: dict[str, float] = {}

        self.frame = 0
        self._persist_buffer: deque = deque(maxlen=20)
        self._last_memory_frame = -100

    def process_frame(self, weights, responses) -> np.ndarray:
        """Main entry point. Returns additive bias from memory influence."""
        state = np.asarray(weights, dtype=np.float32)
        resp = np.asarray(responses, dtype=np.float32)
        self.frame += 1

        self.stm.append(state.copy())

        intensity = float(np.linalg.norm(resp))
        valence = self._compute_valence(state)
        emotion = self._classify_emotion(state)

        self._raw_history.append((self.frame, state.copy(), valence))
        self._persist_buffer.append(valence)

        if self._should_form_memory(intensity, valence):
            self._form_memory(state, intensity, valence, emotion)

        self._reinforce_similar(state)
        self._update_cycle_detection(state)

        if self.frame - self._last_compress_frame >= self.compression_interval:
            self._compress_history()

        return self._compute_memory_bias(state)

    def compute_influences(self, weight_dict: dict[str, float]) -> dict[str, float]:
        """Protocol-compatible: return per-aspect influence dict."""
        state = np.array([weight_dict.get(a, 0.0) for a in self.aspects], dtype=np.float32)
        bias = self._compute_memory_bias(state)
        return {a: float(bias[i]) for i, a in enumerate(self.aspects)}

    def get_stm_trend(self, window=10):
        if len(self.stm) < max(window, 2):
            return np.zeros(self.n)
        recent = np.array(list(self.stm)[-window:])
        x = np.arange(window, dtype=float)
        x -= x.mean()
        denom = np.dot(x, x)
        if denom < 1e-12:
            return np.zeros(self.n)
        slopes = np.zeros(self.n)
        for j in range(self.n):
            slopes[j] = np.dot(x, recent[:, j] - recent[:, j].mean()) / denom
        return slopes

    def get_detected_cycles(self):
        return dict(self._detected_periods)

    def get_strongest_memories(self, k=5):
        scored = [(m, m.effective_strength(self.frame, self.decay_half_life))
                  for m in self.memories]
        scored.sort(key=lambda x: x[1], reverse=True)
        return scored[:k]

    # ── Internals ──

    def _compute_valence(self, state):
        pos = ['self-esteem', 'motivation', 'agency', 'self-efficacy']
        v = sum(state[self._idx[a]] for a in pos if a in self._idx)
        if 'self-regulation' in self._idx:
            v -= 0.3 * abs(state[self._idx['self-regulation']])
        return float(np.tanh(v / max(len(pos), 1)))

    def _classify_emotion(self, state):
        def _get(name):
            return state[self._idx[name]] if name in self._idx else 0

        ea, se, mo, ag = _get('emotional_awareness'), _get('self-esteem'), \
                          _get('motivation'), _get('agency')
        sr = _get('self-regulation')

        if ea > 0.3 and mo > 0.2 and se > 0.1:
            return 'excited'
        if se > 0.3 and ag > 0.2:
            return 'confident'
        if mo > 0.3 and ag > 0.1:
            return 'driven'
        if se < -0.2 and mo < 0:
            return 'distressed'
        if ea < -0.2 and sr > 0.3:
            return 'suppressed'
        if ea < -0.1 and se < -0.1:
            return 'anxious'
        if abs(ea) < 0.15 and abs(mo) < 0.15:
            return 'calm'
        return 'neutral'

    def _should_form_memory(self, intensity, valence):
        if self.frame - self._last_memory_frame < 30:
            return False
        if intensity > 1.5:
            return True
        if len(self._persist_buffer) >= 15:
            recent = list(self._persist_buffer)[-15:]
            mean_v = sum(recent) / len(recent)
            if abs(mean_v) > 0.25:
                consistent = sum(1 for v in recent if v * mean_v > 0)
                if consistent >= 12:
                    return True
        if intensity > 0.8 and abs(valence) > 0.4:
            return True
        return False

    def _form_memory(self, state, intensity, valence, emotion):
        mem = Memory(time.monotonic(), self.frame, state.copy(),
                     min(intensity, 3.0), valence, emotion)
        self.memories.append(mem)
        self._last_memory_frame = self.frame
        if len(self.memories) > self.ltm_capacity:
            self._evict_weakest()

    def _evict_weakest(self):
        if not self.memories:
            return
        worst_i = min(range(len(self.memories)),
                      key=lambda i: self.memories[i].effective_strength(
                          self.frame, self.decay_half_life))
        self.memories.pop(worst_i)

    def _reinforce_similar(self, state):
        for mem in self.memories:
            if mem.similarity(state) > 0.92:
                mem.reinforcement_count += 1
                mem.last_accessed = self.frame

    def _compute_memory_bias(self, current_state):
        bias = np.zeros(self.n, dtype=np.float64)
        for mem in self.memories:
            strength = mem.effective_strength(self.frame, self.decay_half_life)
            if strength < 1e-4:
                continue
            sim = mem.similarity(current_state)
            if abs(sim) < 0.1:
                continue
            direction = mem.state - current_state
            sign = 1.0 if mem.valence >= 0 else -1.0
            magnitude = self.influence_scale * strength * abs(sim) * sign
            bias += magnitude * direction

        norm = np.linalg.norm(bias)
        if norm > 0.05:
            bias *= 0.05 / norm
        return bias

    def _update_cycle_detection(self, state):
        if len(self.stm) < 10:
            return
        recent = np.array(list(self.stm)[-10:])
        means = recent.mean(axis=0)
        for j in range(self.n):
            centered = recent[:, j] - means[j]
            crossings = sum(1 for k in range(1, len(centered))
                            if centered[k-1] * centered[k] < 0)
            self._cycle_accum[j] += crossings
            self._cycle_counts[j] += 1

        if self.frame % 100 == 0:
            self._detected_periods.clear()
            for j in range(self.n):
                if self._cycle_counts[j] > 0:
                    avg = self._cycle_accum[j] / self._cycle_counts[j]
                    if avg > 0.5:
                        self._detected_periods[self.aspects[j]] = round(20.0 / avg, 1)
            self._cycle_accum[:] = 0
            self._cycle_counts[:] = 0

    def _compress_history(self):
        cutoff = self.frame - self.compression_interval
        to_compress = [e for e in self._raw_history if e[0] <= cutoff]
        self._raw_history = [e for e in self._raw_history if e[0] > cutoff]
        self._last_compress_frame = self.frame

        bucket_size = 100
        for start in range(0, len(to_compress), bucket_size):
            chunk = to_compress[start:start + bucket_size]
            states = np.array([c[1] for c in chunk], dtype=np.float64)
            valences = [c[2] for c in chunk]
            self._compressed.append(HistoryBucket(
                frame_start=chunk[0][0], frame_end=chunk[-1][0],
                count=len(chunk), mean_state=states.mean(axis=0),
                var_state=states.var(axis=0),
                mean_valence=sum(valences) / len(valences),
                max_intensity=float(np.max(np.linalg.norm(
                    states - states.mean(axis=0), axis=1)))))
