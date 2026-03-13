//! Energy and arousal system: finite energy budget, arousal modulation,
//! selective attention, flow states, stress responses, circadian rhythm.
//!
//! Mirrors core/energy.py exactly.

use std::f64::consts::TAU;

// Aspect names that receive priority salience during stress.
const STRESS_PRIORITY: &[&str] = &[
    "self-regulation",
    "situational_awareness",
    "agency",
    "body_awareness",
];

// Flow cluster definitions: (name, member aspect names).
const FLOW_CLUSTERS: &[(&str, &[&str])] = &[
    (
        "executive",
        &["metacognition", "self-regulation", "self-monitoring", "goal-setting"],
    ),
    (
        "reflective",
        &["introspection", "reflection", "temporal_awareness", "moral_awareness"],
    ),
    (
        "social",
        &["theory_of_mind", "social_awareness", "emotional_awareness", "situational_awareness"],
    ),
    (
        "identity",
        &["self-concept", "self-esteem", "self-recognition", "self-efficacy"],
    ),
    (
        "drive",
        &["motivation", "agency", "goal-setting", "self-development"],
    ),
];

const N_CLUSTERS: usize = 5;

pub struct EnergyState {
    pub energy: f64,
    pub energy_pct: f64,
    pub arousal: f64,
    pub stress: f64,
    pub attended: Vec<String>,    // aspect names currently attended
    pub flow_states: Vec<String>, // active flow cluster names
    pub circadian: f64,
    pub lr_modifier: f64,
    pub noise_modifier: f64,
    pub aspect_scales: Vec<f64>, // per-aspect coupling scale
}

pub struct EnergySystem {
    energy: f64,
    max_energy: f64,
    arousal: f64,
    arousal_baseline: f64,
    arousal_decay: f64,
    stress_level: f64,
    stress_decay: f64,
    stress_threshold: f64,
    attention: Vec<f64>,       // per-aspect attention weight
    attention_slots: usize,
    flow_timer: Vec<u32>,      // per-cluster timer
    flow_active: Vec<bool>,
    flow_threshold: u32,
    prev_weights: Option<Vec<f64>>,
    resting: Vec<f64>,
    iteration: u64,
    circadian_period: u32,
    n: usize,
    aspect_names: Vec<String>,

    // Resolved indices: per cluster, which aspect indices belong to it.
    cluster_indices: Vec<Vec<usize>>,
    // Per aspect: is it a stress-priority aspect?
    is_stress_priority: Vec<bool>,
    // Per aspect: which cluster indices include it (for flow boost).
    aspect_cluster_membership: Vec<Vec<usize>>,
}

impl EnergySystem {
    /// Construct from a list of aspect names, their resting weights, and tuning params.
    ///
    /// `resting_weights` is parallel to `aspects`.
    pub fn new(
        aspects: Vec<String>,
        resting_weights: &[f64],
        max_energy: f64,
        attention_slots: usize,
        circadian_period: u32,
    ) -> Self {
        let n = aspects.len();

        // Build a name→index map.
        let idx_of = |name: &str| aspects.iter().position(|a| a == name);

        // Resolve cluster membership indices.
        let cluster_indices: Vec<Vec<usize>> = FLOW_CLUSTERS
            .iter()
            .map(|(_cname, members)| {
                members.iter().filter_map(|m| idx_of(m)).collect()
            })
            .collect();

        // Per-aspect stress-priority flag.
        let is_stress_priority: Vec<bool> = aspects
            .iter()
            .map(|a| STRESS_PRIORITY.contains(&a.as_str()))
            .collect();

        // Per-aspect: list of cluster indices that include this aspect.
        let aspect_cluster_membership: Vec<Vec<usize>> = (0..n)
            .map(|ai| {
                cluster_indices
                    .iter()
                    .enumerate()
                    .filter(|(_ci, members)| members.contains(&ai))
                    .map(|(ci, _)| ci)
                    .collect()
            })
            .collect();

        let resting = resting_weights.to_vec();

        Self {
            energy: max_energy,
            max_energy,
            arousal: 0.7,
            arousal_baseline: 0.7,
            arousal_decay: 0.02,
            stress_level: 0.0,
            stress_decay: 0.03,
            stress_threshold: 0.35,
            attention: vec![0.15; n],
            attention_slots,
            flow_timer: vec![0u32; N_CLUSTERS],
            flow_active: vec![false; N_CLUSTERS],
            flow_threshold: 15,
            prev_weights: None,
            resting,
            iteration: 0,
            circadian_period,
            n,
            aspect_names: aspects,
            cluster_indices,
            is_stress_priority,
            aspect_cluster_membership,
        }
    }

    /// Advance one tick. Returns full EnergyState including modifiers.
    pub fn step(&mut self, weights: &[f64]) -> EnergyState {
        // Displacement from resting.
        let displacement: Vec<f64> = (0..self.n)
            .map(|i| (weights[i] - self.resting[i]).abs())
            .collect();

        // Rate of change since last tick.
        let roc: Vec<f64> = match &self.prev_weights {
            Some(prev) => (0..self.n).map(|i| (weights[i] - prev[i]).abs()).collect(),
            None => vec![0.0; self.n],
        };
        self.prev_weights = Some(weights.to_vec());

        self.update_energy(&displacement);
        self.update_stress(&displacement, &roc);
        self.update_arousal();
        self.update_attention(&displacement, &roc);
        self.update_flow(weights);

        let circadian = self.circadian_factor();
        let energy_frac = if self.max_energy > 0.0 {
            (self.energy / self.max_energy).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let lr_mod = self.arousal * energy_frac.max(0.1) * circadian;

        let mut noise_mod = self.arousal * circadian;
        if self.stress_level > 0.3 {
            noise_mod *= 1.0 + 0.5 * self.stress_level;
        }
        if self.flow_active.iter().any(|&f| f) {
            noise_mod *= 0.7;
        }

        // Per-aspect scales.
        let mut aspect_scales = Vec::with_capacity(self.n);
        for i in 0..self.n {
            let mut scale = self.attention[i];
            if energy_frac < 0.3 {
                scale *= 0.3 + 0.7 * (energy_frac / 0.3);
            }
            if self.stress_level > 0.3 && self.is_stress_priority[i] {
                scale *= 1.0 + self.stress_level;
            }
            // Flow boost: first matching active cluster wins.
            for &ci in &self.aspect_cluster_membership[i] {
                if self.flow_active[ci] {
                    scale *= 1.3;
                    break;
                }
            }
            aspect_scales.push(scale.clamp(0.05, 3.0));
        }

        // Attended aspects: top-k by attention weight.
        let attended = self.top_attended();
        let flow_states: Vec<String> = FLOW_CLUSTERS
            .iter()
            .enumerate()
            .filter(|(ci, _)| self.flow_active[*ci])
            .map(|(_, (cname, _))| cname.to_string())
            .collect();

        self.iteration += 1;

        let energy_pct = if self.max_energy > 0.0 {
            (100.0 * self.energy / self.max_energy).clamp(0.0, 100.0)
        } else {
            0.0
        };

        EnergyState {
            energy: self.energy,
            energy_pct,
            arousal: self.arousal,
            stress: self.stress_level,
            attended,
            flow_states,
            circadian,
            lr_modifier: lr_mod,
            noise_modifier: noise_mod,
            aspect_scales,
        }
    }

    /// Spike stress and arousal by `magnitude`.
    pub fn inject_perturbation(&mut self, magnitude: f64) {
        self.stress_level = (self.stress_level + magnitude.min(1.0)).min(1.0);
        self.arousal = (self.arousal + magnitude.min(1.0) * 0.6).min(2.0);
    }

    /// `0.75 + 0.25 * cos(2π * iteration / period)`
    pub fn circadian_factor(&self) -> f64 {
        if self.circadian_period == 0 {
            return 1.0;
        }
        let phase = TAU * self.iteration as f64 / self.circadian_period as f64;
        0.75 + 0.25 * phase.cos()
    }

    // ── Internal helpers ──

    fn update_energy(&mut self, displacement: &[f64]) {
        let cost = 0.02 * displacement.iter().map(|d| d * d).sum::<f64>()
            * (1.0 + self.stress_level);
        self.energy -= cost;

        let mean_disp = displacement.iter().sum::<f64>() / self.n as f64;
        if mean_disp < 0.25 {
            self.energy += 0.15 * (1.0 - mean_disp / 0.25);
        }
        self.energy = self.energy.clamp(0.0, self.max_energy);
    }

    fn update_stress(&mut self, displacement: &[f64], roc: &[f64]) {
        let max_roc = roc.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let max_disp = displacement.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let perturbation = max_roc.max(max_disp * 0.3);
        if perturbation > self.stress_threshold {
            let spike =
                ((perturbation - self.stress_threshold) / self.stress_threshold).min(1.0);
            self.stress_level = (self.stress_level + spike * 0.4).min(1.0);
        }
        self.stress_level *= 1.0 - self.stress_decay;
        self.stress_level = self.stress_level.clamp(0.0, 1.0);
    }

    fn update_arousal(&mut self) {
        let mut target = self.arousal_baseline;
        if self.stress_level > 0.2 {
            target += self.stress_level * 0.8;
        }
        let energy_frac = if self.max_energy > 0.0 {
            (self.energy / self.max_energy).clamp(0.0, 1.0)
        } else {
            0.0
        };
        if energy_frac < 0.3 {
            target *= energy_frac / 0.3;
        }
        target *= self.circadian_factor();
        self.arousal += self.arousal_decay * (target - self.arousal);
        self.arousal = self.arousal.clamp(0.05, 2.0);
    }

    fn update_attention(&mut self, displacement: &[f64], roc: &[f64]) {
        let mut salience: Vec<f64> = (0..self.n)
            .map(|i| 0.6 * roc[i] + 0.4 * displacement[i])
            .collect();

        if self.stress_level > 0.3 {
            for (i, slot) in salience.iter_mut().enumerate().take(self.n) {
                if self.is_stress_priority[i] {
                    *slot += self.stress_level * 0.5;
                }
            }
        }

        // Indices of top-k by salience.
        let top = top_k_indices(&salience, self.attention_slots);

        for i in 0..self.n {
            if top.contains(&i) {
                self.attention[i] += 0.15 * (1.0 - self.attention[i]);
            } else {
                self.attention[i] += 0.08 * (0.15 - self.attention[i]);
            }
            self.attention[i] = self.attention[i].clamp(0.05, 1.0);
        }
    }

    fn update_flow(&mut self, weights: &[f64]) {
        for ci in 0..N_CLUSTERS {
            let indices = &self.cluster_indices[ci];
            if indices.is_empty() {
                continue;
            }
            // Displacement of cluster members from resting.
            let cluster_abs: Vec<f64> = indices
                .iter()
                .map(|&i| (weights[i] - self.resting[i]).abs())
                .collect();

            let in_range = cluster_abs.iter().all(|&v| (0.2..=0.85).contains(&v));
            let std_dev = std_dev(&cluster_abs);
            let stable = std_dev < 0.15;

            if in_range && stable && self.stress_level < 0.3 {
                self.flow_timer[ci] += 1;
            } else {
                self.flow_timer[ci] = self.flow_timer[ci].saturating_sub(2);
            }
            self.flow_active[ci] = self.flow_timer[ci] >= self.flow_threshold;
        }
    }

    /// Return the names of the top `attention_slots` attended aspects.
    fn top_attended(&self) -> Vec<String> {
        let indices = top_k_indices(&self.attention, self.attention_slots);
        indices.iter().map(|&i| self.aspect_names[i].clone()).collect()
    }
}

// ── Pure utilities ──

/// Population standard deviation of a slice.
fn std_dev(xs: &[f64]) -> f64 {
    if xs.len() <= 1 {
        return 0.0;
    }
    let mean = xs.iter().sum::<f64>() / xs.len() as f64;
    let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / xs.len() as f64;
    var.max(0.0).sqrt()
}

/// Indices of the top-k largest values (unsorted; matches numpy argsort behaviour).
fn top_k_indices(vals: &[f64], k: usize) -> Vec<usize> {
    let k = k.min(vals.len());
    let mut indices: Vec<usize> = (0..vals.len()).collect();
    // Partial sort: bring the k largest to the end (mirrors numpy argsort[-k:]).
    indices.sort_unstable_by(|&a, &b| {
        vals[a].partial_cmp(&vals[b]).unwrap_or(std::cmp::Ordering::Equal)
    });
    indices[vals.len() - k..].to_vec()
}
