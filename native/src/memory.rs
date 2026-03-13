/// Memory and temporal processing: short-term buffer, long-term memory
/// formation, emotional tagging, decay, cycle detection, and history compression.
///
/// Mirrors core/memory.py exactly.

use std::collections::{HashMap, VecDeque};

// ── Emotion encoding ─────────────────────────────────────────────────────────
// 0=neutral, 1=excited, 2=confident, 3=driven, 4=distressed, 5=suppressed,
// 6=anxious, 7=calm

pub struct Memory {
    pub frame: u64,
    pub state: Vec<f32>,
    pub intensity: f32,
    pub valence: f32,
    pub emotion: u8,
    pub reinforcement_count: u16,
    pub last_accessed: u64,
}

impl Memory {
    fn similarity(&self, state_vec: &[f32]) -> f64 {
        let dot: f64 = self.state.iter().zip(state_vec).map(|(&a, &b)| a as f64 * b as f64).sum();
        let norm_a: f64 = self.state.iter().map(|&x| (x as f64) * (x as f64)).sum::<f64>().sqrt();
        let norm_b: f64 = state_vec.iter().map(|&x| (x as f64) * (x as f64)).sum::<f64>().sqrt();
        if norm_a < 1e-9 || norm_b < 1e-9 {
            return 0.0;
        }
        dot / (norm_a * norm_b)
    }

    pub fn effective_strength(&self, current_frame: u64, half_life: f64) -> f64 {
        let age = (current_frame.saturating_sub(self.frame)).max(1) as f64;
        let decay = (2.0f64).powf(-age / half_life);
        let emotion_mult = 1.0 + 0.5 * (self.valence as f64).abs();
        let reinforce_mult = 1.0 + 0.1 * self.reinforcement_count as f64;
        self.intensity as f64 * decay * emotion_mult * reinforce_mult
    }
}

pub struct MemorySystem {
    aspects: Vec<String>,
    aspect_indices: HashMap<String, usize>,
    stm: VecDeque<Vec<f32>>,
    memories: Vec<Memory>,
    pub frame: u64,
    n: usize,
    ltm_capacity: usize,
    influence_scale: f64,
    decay_half_life: f64,
    persist_buffer: VecDeque<f64>,
    last_memory_frame: i64,
    cycle_accum: Vec<f64>,
    cycle_counts: Vec<u64>,
    detected_periods: Vec<(usize, f64)>,
    stm_size: usize,
}

impl MemorySystem {
    pub fn new(
        aspects: Vec<String>,
        stm_size: usize,
        ltm_capacity: usize,
        influence_scale: f64,
        decay_half_life: f64,
    ) -> Self {
        let n = aspects.len();
        let aspect_indices: HashMap<String, usize> =
            aspects.iter().enumerate().map(|(i, a)| (a.clone(), i)).collect();
        Self {
            aspects,
            aspect_indices,
            stm: VecDeque::with_capacity(stm_size + 1),
            memories: Vec::new(),
            frame: 0,
            n,
            ltm_capacity,
            influence_scale,
            decay_half_life,
            persist_buffer: VecDeque::with_capacity(21),
            last_memory_frame: -100,
            cycle_accum: vec![0.0; n],
            cycle_counts: vec![0u64; n],
            detected_periods: Vec::new(),
            stm_size,
        }
    }

    /// Main entry point. Returns additive memory bias (length n).
    pub fn process_frame(&mut self, weights: &[f64], stimuli: &[f64]) -> Vec<f64> {
        let state: Vec<f32> = weights.iter().map(|&x| x as f32).collect();
        self.frame += 1;

        // STM push (bounded)
        if self.stm.len() == self.stm_size {
            self.stm.pop_front();
        }
        self.stm.push_back(state.clone());

        let resp: Vec<f64> = stimuli.to_vec();
        let intensity = vec_norm_f64(&resp) as f32;
        let valence = self.compute_valence(&state);
        let emotion = self.classify_emotion(&state);

        // persist_buffer is maxlen=20
        if self.persist_buffer.len() == 20 {
            self.persist_buffer.pop_front();
        }
        self.persist_buffer.push_back(valence as f64);

        if self.should_form_memory(intensity as f64, valence as f64) {
            self.form_memory(state.clone(), intensity, valence, emotion);
        }

        self.reinforce_similar(&state);
        self.update_cycle_detection(&state);

        self.compute_memory_bias(&state)
    }

    /// Protocol-compatible: return per-aspect influence from a weight slice.
    pub fn compute_influences(&self, weights: &[f64]) -> Vec<f64> {
        let state: Vec<f32> = weights.iter().map(|&x| x as f32).collect();
        self.compute_memory_bias(&state)
    }

    // ── Internals ────────────────────────────────────────────────────────────

    fn get_idx(&self, name: &str) -> Option<usize> {
        self.aspect_indices.get(name).copied()
    }

    fn get_state_val(&self, state: &[f32], name: &str) -> f32 {
        self.get_idx(name).map(|i| state[i]).unwrap_or(0.0)
    }

    fn compute_valence(&self, state: &[f32]) -> f32 {
        let pos = ["self-esteem", "motivation", "agency", "self-efficacy"];
        let v: f32 = pos
            .iter()
            .filter_map(|&a| self.get_idx(a))
            .map(|i| state[i])
            .sum();
        let sr = self.get_state_val(state, "self-regulation");
        let adjusted = v - 0.3 * sr.abs();
        let divisor = pos.len().max(1) as f32;
        (adjusted / divisor).tanh()
    }

    fn classify_emotion(&self, state: &[f32]) -> u8 {
        let ea = self.get_state_val(state, "emotional_awareness");
        let se = self.get_state_val(state, "self-esteem");
        let mo = self.get_state_val(state, "motivation");
        let ag = self.get_state_val(state, "agency");

        if ea > 0.3 && mo > 0.2 && se > 0.1 {
            return 1; // excited
        }
        if se > 0.3 && ag > 0.2 {
            return 2; // confident
        }
        if mo > 0.3 && ag > 0.1 {
            return 3; // driven
        }
        if se < -0.2 && mo < 0.0 {
            return 4; // distressed
        }
        let sr = self.get_state_val(state, "self-regulation");
        if ea < -0.2 && sr > 0.3 {
            return 5; // suppressed
        }
        if ea < -0.1 && se < -0.1 {
            return 6; // anxious
        }
        if ea.abs() < 0.15 && mo.abs() < 0.15 {
            return 7; // calm
        }
        0 // neutral
    }

    fn should_form_memory(&self, intensity: f64, valence: f64) -> bool {
        if self.frame as i64 - self.last_memory_frame < 30 {
            return false;
        }
        if intensity > 1.5 {
            return true;
        }
        let buf_len = self.persist_buffer.len();
        if buf_len >= 15 {
            let recent: Vec<f64> = self.persist_buffer.iter().rev().take(15).copied().collect();
            let mean_v: f64 = recent.iter().sum::<f64>() / 15.0;
            if mean_v.abs() > 0.25 {
                let consistent = recent.iter().filter(|&&v| v * mean_v > 0.0).count();
                if consistent >= 12 {
                    return true;
                }
            }
        }
        if intensity > 0.8 && valence.abs() > 0.4 {
            return true;
        }
        false
    }

    fn form_memory(&mut self, state: Vec<f32>, intensity: f32, valence: f32, emotion: u8) {
        let mem = Memory {
            frame: self.frame,
            state,
            intensity: intensity.min(3.0),
            valence,
            emotion,
            reinforcement_count: 0,
            last_accessed: self.frame,
        };
        self.memories.push(mem);
        self.last_memory_frame = self.frame as i64;
        if self.memories.len() > self.ltm_capacity {
            self.evict_weakest();
        }
    }

    fn evict_weakest(&mut self) {
        if self.memories.is_empty() {
            return;
        }
        let frame = self.frame;
        let hl = self.decay_half_life;
        let worst = self
            .memories
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.effective_strength(frame, hl)
                    .partial_cmp(&b.effective_strength(frame, hl))
                    .unwrap()
            })
            .map(|(i, _)| i)
            .unwrap();
        self.memories.swap_remove(worst);
    }

    fn reinforce_similar(&mut self, state: &[f32]) {
        let frame = self.frame;
        for mem in &mut self.memories {
            if mem.similarity(state) > 0.92 {
                mem.reinforcement_count = mem.reinforcement_count.saturating_add(1);
                mem.last_accessed = frame;
            }
        }
    }

    fn compute_memory_bias(&self, current_state: &[f32]) -> Vec<f64> {
        let mut bias = vec![0.0f64; self.n];
        for mem in &self.memories {
            let strength = mem.effective_strength(self.frame, self.decay_half_life);
            if strength < 1e-4 {
                continue;
            }
            let sim = mem.similarity(current_state);
            if sim.abs() < 0.1 {
                continue;
            }
            let sign = if mem.valence >= 0.0 { 1.0f64 } else { -1.0f64 };
            let magnitude = self.influence_scale * strength * sim.abs() * sign;
            for j in 0..self.n {
                let dir = mem.state[j] as f64 - current_state[j] as f64;
                bias[j] += magnitude * dir;
            }
        }

        // Clamp norm to 0.05
        let norm = vec_norm_f64(&bias);
        if norm > 0.05 {
            let scale = 0.05 / norm;
            for b in &mut bias {
                *b *= scale;
            }
        }
        bias
    }

    fn update_cycle_detection(&mut self, _state: &[f32]) {
        if self.stm.len() < 10 {
            return;
        }
        // Take the last 10 entries
        let start = self.stm.len() - 10;
        let recent: Vec<&Vec<f32>> = self.stm.iter().skip(start).collect();

        for j in 0..self.n {
            let mean: f32 = recent.iter().map(|s| s[j]).sum::<f32>() / 10.0;
            let centered: Vec<f32> = recent.iter().map(|s| s[j] - mean).collect();
            let crossings: usize = (1..centered.len())
                .filter(|&k| centered[k - 1] * centered[k] < 0.0)
                .count();
            self.cycle_accum[j] += crossings as f64;
            self.cycle_counts[j] += 1;
        }

        if self.frame % 100 == 0 {
            self.detected_periods.clear();
            for j in 0..self.n {
                if self.cycle_counts[j] > 0 {
                    let avg = self.cycle_accum[j] / self.cycle_counts[j] as f64;
                    if avg > 0.5 {
                        // round to 1 decimal place
                        let period = (20.0 / avg * 10.0).round() / 10.0;
                        self.detected_periods.push((j, period));
                    }
                }
            }
            for v in &mut self.cycle_accum {
                *v = 0.0;
            }
            for c in &mut self.cycle_counts {
                *c = 0;
            }
        }
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn detected_periods(&self) -> &[(usize, f64)] {
        &self.detected_periods
    }

    pub fn memory_count(&self) -> usize {
        self.memories.len()
    }

    pub fn aspect_name(&self, idx: usize) -> Option<&str> {
        self.aspects.get(idx).map(|s| s.as_str())
    }
}

fn vec_norm_f64(v: &[f64]) -> f64 {
    v.iter().map(|&x| x * x).sum::<f64>().sqrt()
}
