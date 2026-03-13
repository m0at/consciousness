/// Emergent behavior detection: phases, oscillations, cascades,
/// attractors, resilience, and entropy.
///
/// Mirrors core/analyzer.py exactly.

use std::collections::{HashMap, VecDeque};

// ── Phase encoding ────────────────────────────────────────────────────────────
// 0=growth, 1=stability, 2=crisis, 3=recovery

pub struct PhaseState {
    pub phase: u8,
    pub aspect_idx: usize,
    pub confidence: f64,
    pub duration: u32,
    pub slope: f64,
}

// ── Oscillation encoding ─────────────────────────────────────────────────────
// 0=oscillating, 1=converging, 2=diverging

pub struct OscillationState {
    pub aspect_idx: usize,
    pub pattern: u8,
    pub frequency: f64,
    pub amplitude: f64,
    pub damping: f64,
}

pub struct Cascade {
    pub trigger_idx: usize,
    pub trigger_delta: f64,
    /// (aspect_idx, delta, depth_level)
    pub path: Vec<(usize, f64, u8)>,
    pub total_magnitude: f64,
    pub tick: u64,
}

pub struct Attractor {
    pub center: Vec<f64>,
    pub basin_radius: f64,
    pub strength: f64,
    pub age: u64,
    pub drift_rate: f64,
}

pub struct EntropyState {
    pub shannon: f64,
    pub normalized: f64,
    pub delta: f64,
    pub label: u8, // 0=settled, 1=active, 2=turbulent
}

pub struct SystemAnalyzer {
    aspects: Vec<String>,
    aspect_idx: HashMap<String, usize>,
    #[allow(dead_code)]
    interrelations: Vec<f64>, // n×n row-major (kept for protocol symmetry)
    history: VecDeque<Vec<f64>>,
    pub tick: u64,
    n: usize,
    window: usize,
    phase: Vec<u8>,
    phase_duration: Vec<u32>,
    prev_weights: Option<Vec<f64>>,
    cascade_log: Vec<Cascade>,
    attractors: Vec<Attractor>,
    entropy_history: VecDeque<f64>,
    perturbation_snapshot: Option<Vec<f64>>,
    perturbation_tick: Option<u64>,
    pub stability_threshold: f64,
    pub cascade_threshold: f64,
    interrelation_data: Vec<f64>,
    #[allow(dead_code)]
    interrelation_n: usize,
}

impl SystemAnalyzer {
    pub fn new(
        aspects: Vec<String>,
        interrelations: Vec<f64>,
        window: usize,
        cascade_threshold: f64,
        stability_threshold: f64,
    ) -> Self {
        let n = aspects.len();
        let aspect_idx: HashMap<String, usize> =
            aspects.iter().enumerate().map(|(i, a)| (a.clone(), i)).collect();
        let history_cap = (window * 4).max(200);
        let entropy_cap = window * 2;
        Self {
            aspects,
            aspect_idx,
            interrelations: interrelations.clone(),
            history: VecDeque::with_capacity(history_cap + 1),
            tick: 0,
            n,
            window,
            phase: vec![1u8; n], // 1=stability
            phase_duration: vec![0u32; n],
            prev_weights: None,
            cascade_log: Vec::new(),
            attractors: Vec::new(),
            entropy_history: VecDeque::with_capacity(entropy_cap + 1),
            perturbation_snapshot: None,
            perturbation_tick: None,
            stability_threshold,
            cascade_threshold,
            interrelation_data: interrelations,
            interrelation_n: n,
        }
    }

    /// Feed new weights. Returns a bitmask of what was computed (for callers
    /// that need to know), and fills the out-params.
    pub fn tick_update(&mut self, weights: Vec<f64>) -> TickResults {
        let history_cap = (self.window * 4).max(200);
        if self.history.len() == history_cap {
            self.history.pop_front();
        }
        self.history.push_back(weights.clone());
        self.tick += 1;

        let mut results = TickResults::default();

        if self.history.len() >= 3 {
            results.phases = Some(self.detect_phases());
            results.oscillations = Some(self.detect_oscillations());
            results.cascades = Some(self.detect_cascades(&weights));
            results.entropy = Some(self.compute_entropy(&weights));
        }

        if self.history.len() >= self.window {
            self.detect_attractors();
            results.attractors = true;
        }

        if self.perturbation_snapshot.is_some() {
            results.resilience = self.measure_resilience(&weights);
        }

        self.prev_weights = Some(weights);
        results
    }

    pub fn mark_perturbation(&mut self) {
        if self.history.len() >= 2 {
            let snap = self.history.iter().rev().nth(1).unwrap().clone();
            self.perturbation_snapshot = Some(snap);
            self.perturbation_tick = Some(self.tick);
        }
    }

    pub fn detect_phases(&mut self) -> Vec<PhaseState> {
        let arr = self.recent_array();
        if arr.is_empty() || arr.len() < 5 {
            return vec![];
        }

        let lookback = arr.len().min(self.window);
        let recent = &arr[arr.len() - lookback..];
        let t: Vec<f64> = (0..lookback).map(|x| x as f64).collect();

        let mut results = Vec::with_capacity(self.n);
        for i in 0..self.n {
            let series: Vec<f64> = recent.iter().map(|row| row[i]).collect();
            let slope = linreg_slope(&t, &series);

            // diffs and volatility
            let diffs: Vec<f64> = series.windows(2).map(|w| w[1] - w[0]).collect();
            let volatility = if diffs.len() > 1 { std_dev(&diffs) } else { 0.0 };

            let tail: Vec<f64> = series.iter().rev().take(10).copied().collect();
            let mean_abs: f64 =
                tail.iter().map(|x| x.abs()).sum::<f64>() / tail.len() as f64 + 1e-12;
            let rel_vol = volatility / mean_abs;

            let prev_phase = self.phase[i];
            let (phase, confidence) = if rel_vol > 0.15 {
                (2u8, (rel_vol / 0.3).min(1.0)) // crisis
            } else if slope.abs() > self.stability_threshold * 2.0 {
                let p = if prev_phase == 2 { 3u8 } else { 0u8 }; // recovery or growth
                let c = (slope.abs() / (self.stability_threshold * 6.0)).min(1.0);
                (p, c)
            } else {
                let c =
                    1.0 - slope.abs() / (self.stability_threshold * 2.0 + 1e-12);
                (1u8, c) // stability
            };

            if phase == prev_phase {
                self.phase_duration[i] += 1;
            } else {
                self.phase_duration[i] = 1;
            }
            self.phase[i] = phase;

            results.push(PhaseState {
                phase,
                aspect_idx: i,
                confidence: confidence.clamp(0.0, 1.0),
                duration: self.phase_duration[i],
                slope,
            });
        }
        results
    }

    pub fn detect_oscillations(&self) -> Vec<OscillationState> {
        let arr = self.recent_array();
        if arr.len() < 8 {
            return vec![];
        }

        let lookback = arr.len().min(self.window);
        let recent = &arr[arr.len() - lookback..];
        let mut results = Vec::with_capacity(self.n);

        for i in 0..self.n {
            let series: Vec<f64> = recent.iter().map(|row| row[i]).collect();
            let diffs: Vec<f64> = series.windows(2).map(|w| w[1] - w[0]).collect();
            let signs: Vec<f64> = diffs.iter().map(|&d| d.signum()).collect();
            let sign_changes = if signs.len() > 1 {
                signs.windows(2).filter(|w| w[0] != w[1]).count()
            } else {
                0
            };
            let freq = sign_changes as f64 / (signs.len().max(1) - 1).max(1) as f64;

            let amplitude = series_ptp(&series);
            let half = series.len() / 2;
            let amp_first = if half > 1 { series_ptp(&series[..half]) } else { amplitude };
            let amp_second = if half > 1 { series_ptp(&series[half..]) } else { amplitude };
            let damping = amp_second - amp_first;

            let pattern = if freq > 0.4 && amplitude > self.stability_threshold {
                if damping < -self.stability_threshold {
                    1u8 // converging
                } else if damping > self.stability_threshold {
                    2u8 // diverging
                } else {
                    0u8 // oscillating
                }
            } else if amplitude < self.stability_threshold * 2.0 {
                1u8 // converging
            } else if damping < 0.0 {
                1u8 // converging
            } else {
                2u8 // diverging
            };

            results.push(OscillationState {
                aspect_idx: i,
                pattern,
                frequency: freq,
                amplitude,
                damping,
            });
        }
        results
    }

    pub fn detect_cascades(&mut self, weights: &[f64]) -> Vec<Cascade> {
        let prev = match self.prev_weights.take() {
            Some(p) => p,
            None => {
                self.prev_weights = Some(weights.to_vec());
                return vec![];
            }
        };

        let deltas: Vec<f64> = weights
            .iter()
            .zip(prev.iter())
            .map(|(&w, &p)| w - p)
            .collect();

        let triggers: Vec<usize> = deltas
            .iter()
            .enumerate()
            .filter(|(_, &d)| d.abs() > self.cascade_threshold)
            .map(|(i, _)| i)
            .collect();

        let n = self.n;
        let mut new_cascades = Vec::new();

        for &trig_idx in &triggers {
            let conn_row = &self.interrelation_data[trig_idx * n..(trig_idx + 1) * n];
            let connected: Vec<usize> = conn_row
                .iter()
                .enumerate()
                .filter(|(j, &v)| v > 0.0 && *j != trig_idx)
                .map(|(j, _)| j)
                .collect();

            let mut path: Vec<(usize, f64, u8)> = Vec::new();

            for &conn_idx in &connected {
                let conn_delta = deltas[conn_idx];
                if conn_delta.abs() > self.stability_threshold {
                    path.push((conn_idx, conn_delta, 0));
                }
            }

            for &conn_idx in &connected {
                let second_row =
                    &self.interrelation_data[conn_idx * n..(conn_idx + 1) * n];
                let sc_indices: Vec<usize> = second_row
                    .iter()
                    .enumerate()
                    .filter(|(j, &v)| v > 0.0 && *j != conn_idx && *j != trig_idx)
                    .map(|(j, _)| j)
                    .collect();

                for sc_idx in sc_indices {
                    let sc_delta = deltas[sc_idx];
                    if sc_delta.abs() > self.stability_threshold * 0.5 {
                        path.push((sc_idx, sc_delta, 1));
                    }
                }
            }

            if !path.is_empty() {
                let total_magnitude: f64 = path.iter().map(|(_, d, _)| d.abs()).sum();
                new_cascades.push(Cascade {
                    trigger_idx: trig_idx,
                    trigger_delta: deltas[trig_idx],
                    path,
                    total_magnitude,
                    tick: self.tick,
                });
            }
        }

        // Log each cascade and build the return list simultaneously
        let mut result: Vec<Cascade> = Vec::with_capacity(new_cascades.len());
        for c in new_cascades {
            result.push(Cascade {
                trigger_idx: c.trigger_idx,
                trigger_delta: c.trigger_delta,
                path: c.path.clone(),
                total_magnitude: c.total_magnitude,
                tick: c.tick,
            });
            self.cascade_log.push(c);
        }
        result
    }

    pub fn detect_attractors(&mut self) -> &[Attractor] {
        let arr = self.recent_array();
        if arr.len() < self.window {
            return &self.attractors;
        }

        let recent = &arr[arr.len() - self.window..];

        // center = mean over window
        let center: Vec<f64> = (0..self.n)
            .map(|j| recent.iter().map(|row| row[j]).sum::<f64>() / self.window as f64)
            .collect();

        // distances from center
        let distances: Vec<f64> = recent
            .iter()
            .map(|row| {
                row.iter()
                    .zip(center.iter())
                    .map(|(&r, &c)| (r - c).powi(2))
                    .sum::<f64>()
                    .sqrt()
            })
            .collect();

        let basin_radius = distances.iter().sum::<f64>() / distances.len() as f64;

        let strength = if distances.len() > 10 {
            let d_mean = distances.iter().sum::<f64>() / distances.len() as f64;
            let d_centered: Vec<f64> = distances.iter().map(|&d| d - d_mean).collect();
            let var = d_centered.iter().map(|&x| x * x).sum::<f64>()
                / d_centered.len() as f64;
            if var > 1e-12 {
                let ac: f64 = d_centered[..d_centered.len() - 1]
                    .iter()
                    .zip(d_centered[1..].iter())
                    .map(|(&a, &b)| a * b)
                    .sum::<f64>();
                let ac_norm = ac / (var * (d_centered.len() - 1) as f64);
                -(ac_norm.abs().max(1e-6)).ln()
            } else {
                10.0
            }
        } else {
            0.0
        };

        let half = self.window / 2;
        let c_first: Vec<f64> = (0..self.n)
            .map(|j| recent[..half].iter().map(|row| row[j]).sum::<f64>() / half as f64)
            .collect();
        let c_second: Vec<f64> = (0..self.n)
            .map(|j| recent[half..].iter().map(|row| row[j]).sum::<f64>() / (self.window - half) as f64)
            .collect();
        let drift_rate = c_first
            .iter()
            .zip(c_second.iter())
            .map(|(&a, &b)| (b - a).powi(2))
            .sum::<f64>()
            .sqrt()
            / half as f64;

        let new_att = Attractor {
            center: center.clone(),
            basin_radius,
            strength,
            age: self.tick,
            drift_rate,
        };

        if let Some(last) = self.attractors.last() {
            let old_c = &last.center;
            let dist = center
                .iter()
                .zip(old_c.iter())
                .map(|(&a, &b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();
            if dist < basin_radius * 2.0 {
                let old_age = last.age;
                let last = self.attractors.last_mut().unwrap();
                *last = new_att;
                last.age = old_age;
            } else {
                self.attractors.push(new_att);
                if self.attractors.len() > 10 {
                    let len = self.attractors.len();
                    self.attractors.drain(..len - 10);
                }
            }
        } else {
            self.attractors.push(new_att);
        }

        &self.attractors
    }

    pub fn measure_resilience(&mut self, weights: &[f64]) -> Option<ResilienceResult> {
        let baseline = self.perturbation_snapshot.as_ref()?;
        let disp_vec: Vec<f64> = weights.iter().zip(baseline.iter()).map(|(&w, &b)| w - b).collect();
        let displacement: f64 = disp_vec.iter().map(|&x| x * x).sum::<f64>().sqrt();
        let elapsed = self.tick - self.perturbation_tick.unwrap_or(self.tick);
        let recovered = displacement < self.stability_threshold * self.n as f64 * 0.5;

        if recovered || elapsed > self.window as u64 * 2 {
            self.perturbation_snapshot = None;
            self.perturbation_tick = None;
        }

        let elasticity =
            (-displacement / (self.stability_threshold * self.n as f64 + 1e-12)).exp();

        Some(ResilienceResult {
            displacement,
            elapsed,
            elasticity,
            recovered,
        })
    }

    pub fn compute_entropy(&mut self, weights: &[f64]) -> EntropyState {
        let total: f64 = weights.iter().map(|x| x.abs()).sum();
        if total < 1e-12 {
            let prev = self.entropy_history.back().copied().unwrap_or(0.0);
            if self.entropy_history.len() == self.window * 2 {
                self.entropy_history.pop_front();
            }
            self.entropy_history.push_back(0.0);
            return EntropyState {
                shannon: 0.0,
                normalized: 0.0,
                delta: -prev,
                label: 0,
            };
        }

        let probs: Vec<f64> = weights
            .iter()
            .map(|x| (x.abs() / total).max(1e-12))
            .collect();
        let shannon: f64 = -probs.iter().map(|&p| p * p.log2()).sum::<f64>();
        let max_ent = (self.n as f64).log2();
        let normalized = if max_ent > 0.0 { shannon / max_ent } else { 0.0 };

        let prev = self.entropy_history.back().copied().unwrap_or(shannon);

        if self.entropy_history.len() == self.window * 2 {
            self.entropy_history.pop_front();
        }
        self.entropy_history.push_back(shannon);

        let label = if normalized > 0.92 {
            2u8 // turbulent
        } else if normalized > 0.75 {
            1u8 // active
        } else {
            0u8 // settled
        };

        EntropyState {
            shannon,
            normalized,
            delta: shannon - prev,
            label,
        }
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn cascade_log(&self) -> &[Cascade] {
        &self.cascade_log
    }

    pub fn attractors(&self) -> &[Attractor] {
        &self.attractors
    }

    pub fn aspect_name(&self, idx: usize) -> Option<&str> {
        self.aspects.get(idx).map(|s| s.as_str())
    }

    pub fn aspect_index(&self, name: &str) -> Option<usize> {
        self.aspect_idx.get(name).copied()
    }

    // ── Private ──────────────────────────────────────────────────────────────

    fn recent_array(&self) -> Vec<Vec<f64>> {
        self.history.iter().cloned().collect()
    }
}

pub struct ResilienceResult {
    pub displacement: f64,
    pub elapsed: u64,
    pub elasticity: f64,
    pub recovered: bool,
}

#[derive(Default)]
pub struct TickResults {
    pub phases: Option<Vec<PhaseState>>,
    pub oscillations: Option<Vec<OscillationState>>,
    pub cascades: Option<Vec<Cascade>>,
    pub entropy: Option<EntropyState>,
    pub attractors: bool, // set to true when attractors were updated
    pub resilience: Option<ResilienceResult>,
}

// ── Math helpers ──────────────────────────────────────────────────────────────

/// `(n*sum(xy) - sum(x)*sum(y)) / (n*sum(xx) - sum(x)^2)`
fn linreg_slope(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len();
    if n < 2 {
        return 0.0;
    }
    let sx: f64 = x.iter().sum();
    let sy: f64 = y.iter().sum();
    let sxy: f64 = x.iter().zip(y.iter()).map(|(&a, &b)| a * b).sum();
    let sxx: f64 = x.iter().map(|&a| a * a).sum();
    let denom = n as f64 * sxx - sx * sx;
    if denom.abs() < 1e-15 {
        return 0.0;
    }
    (n as f64 * sxy - sx * sy) / denom
}

fn std_dev(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    let var = v.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / v.len() as f64;
    var.sqrt()
}

fn series_ptp(v: &[f64]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    let min = v.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    max - min
}
