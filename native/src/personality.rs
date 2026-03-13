//! Attractor-based personality layer.
//!
//! Biases act as persistent background forces pulling weights toward preferred
//! states. Sustained contrary input slowly evolves bias targets (personality
//! growth) and injects volatility (internal conflict).

use std::collections::HashMap;
use crate::rng::Rng;

pub struct PersonalitySystem {
    pub bias_targets: Vec<f64>,
    pub rigidity: Vec<f64>,
    pub original_targets: Vec<f64>,
    pub contrary_pressure: Vec<f64>,
    pub conflict_level: Vec<f64>,
    pub volatility_injection: Vec<f64>,
    pub evolution_rate: f64,
    pub conflict_gain: f64,
    pub n: usize,
    pub profile_name: String,
}

// Default rigidity for aspects not listed in a profile's rigidity map.
const DEFAULT_RIGIDITY: f64 = 0.5;

impl PersonalitySystem {
    /// Build a `PersonalitySystem` from one of the six named profiles.
    ///
    /// `aspect_index` maps aspect name → dense index into weight / stimuli
    /// slices. `n` is the total number of aspects (length of those slices).
    /// Returns `None` if `name` is not a recognised profile.
    pub fn from_profile(
        name: &str,
        aspect_index: &HashMap<String, usize>,
        n: usize,
    ) -> Option<Self> {
        // bias_targets and rigidity keyed by aspect name for the chosen profile.
        let (targets_map, rigidity_map): (HashMap<&str, f64>, HashMap<&str, f64>) = match name {
            "contemplative" => (
                [
                    ("introspection",     0.7),
                    ("reflection",        0.7),
                    ("metacognition",     0.6),
                    ("self-monitoring",   0.5),
                    ("temporal_awareness",0.4),
                    ("moral_awareness",   0.4),
                    ("self-concept",      0.3),
                    ("agency",           -0.1),
                    ("motivation",       -0.1),
                    ("goal-setting",     -0.2),
                ]
                .into_iter().collect(),
                [
                    ("introspection",   0.7),
                    ("reflection",      0.7),
                    ("metacognition",   0.65),
                    ("self-monitoring", 0.5),
                    ("agency",          0.3),
                ]
                .into_iter().collect(),
            ),

            "action-oriented" => (
                [
                    ("agency",                0.7),
                    ("motivation",            0.7),
                    ("goal-setting",          0.6),
                    ("self-efficacy",         0.6),
                    ("situational_awareness", 0.5),
                    ("self-regulation",       0.4),
                    ("self-development",      0.4),
                    ("introspection",        -0.2),
                    ("reflection",           -0.2),
                    ("metacognition",        -0.1),
                ]
                .into_iter().collect(),
                [
                    ("agency",        0.7),
                    ("motivation",    0.7),
                    ("goal-setting",  0.65),
                    ("self-efficacy", 0.6),
                    ("introspection", 0.3),
                ]
                .into_iter().collect(),
            ),

            "empathic" => (
                [
                    ("emotional_awareness",   0.7),
                    ("social_awareness",      0.7),
                    ("theory_of_mind",        0.6),
                    ("moral_awareness",       0.6),
                    ("self-esteem",           0.3),
                    ("self-regulation",       0.3),
                    ("situational_awareness", 0.4),
                    ("introspection",        -0.1),
                    ("self-concept",         -0.1),
                ]
                .into_iter().collect(),
                [
                    ("emotional_awareness", 0.75),
                    ("social_awareness",    0.7),
                    ("theory_of_mind",      0.65),
                    ("moral_awareness",     0.6),
                ]
                .into_iter().collect(),
            ),

            "analytical" => (
                [
                    ("metacognition",         0.7),
                    ("self-monitoring",       0.7),
                    ("situational_awareness", 0.6),
                    ("self-regulation",       0.5),
                    ("temporal_awareness",    0.4),
                    ("introspection",         0.3),
                    ("emotional_awareness",  -0.2),
                    ("social_awareness",     -0.1),
                    ("self-esteem",          -0.1),
                ]
                .into_iter().collect(),
                [
                    ("metacognition",         0.75),
                    ("self-monitoring",       0.7),
                    ("situational_awareness", 0.65),
                    ("self-regulation",       0.55),
                    ("emotional_awareness",   0.35),
                ]
                .into_iter().collect(),
            ),

            "resilient" => (
                [
                    ("self-regulation",  0.7),
                    ("self-esteem",      0.6),
                    ("self-concept",     0.6),
                    ("self-efficacy",    0.5),
                    ("self-recognition", 0.4),
                    ("agency",           0.4),
                    ("self-development", 0.3),
                    ("moral_awareness",  0.3),
                ]
                .into_iter().collect(),
                [
                    ("self-regulation",  0.8),
                    ("self-esteem",      0.8),
                    ("self-concept",     0.8),
                    ("self-efficacy",    0.65),
                    ("self-recognition", 0.6),
                    ("agency",           0.55),
                ]
                .into_iter().collect(),
            ),

            "seeker" => (
                [
                    ("self-development", 0.7),
                    ("motivation",       0.5),
                    ("goal-setting",     0.5),
                    ("introspection",    0.4),
                    ("metacognition",    0.4),
                    ("reflection",       0.3),
                    ("agency",           0.3),
                ]
                .into_iter().collect(),
                [
                    ("self-development", 0.5),
                    ("motivation",       0.4),
                    ("goal-setting",     0.4),
                    ("introspection",    0.35),
                    ("metacognition",    0.35),
                    ("reflection",       0.3),
                    ("agency",           0.3),
                ]
                .into_iter().collect(),
            ),

            _ => return None,
        };

        // Build dense vectors; aspects absent from the profile maps get 0.0
        // targets and DEFAULT_RIGIDITY.
        let mut bias_targets = vec![0.0f64; n];
        let mut rigidity = vec![DEFAULT_RIGIDITY; n];

        for (aspect, idx) in aspect_index {
            if *idx < n {
                if let Some(&t) = targets_map.get(aspect.as_str()) {
                    bias_targets[*idx] = t;
                }
                if let Some(&r) = rigidity_map.get(aspect.as_str()) {
                    rigidity[*idx] = r;
                }
            }
        }

        let original_targets = bias_targets.clone();

        Some(Self {
            contrary_pressure: vec![0.0; n],
            conflict_level: vec![0.0; n],
            volatility_injection: vec![0.0; n],
            bias_targets,
            rigidity,
            original_targets,
            evolution_rate: 0.0003,
            conflict_gain: 2.0,
            n,
            profile_name: name.to_string(),
        })
    }

    /// Compute per-aspect attractor pull forces.
    ///
    /// `out[i] = rigidity[i] * 0.05 * (target[i] - weights[i])`
    pub fn compute_biases(&self, weights: &[f64], out: &mut [f64]) {
        for i in 0..self.n {
            out[i] = self.rigidity[i] * 0.05 * (self.bias_targets[i] - weights[i]);
        }
    }

    /// Track contrary pressure, conflict, and evolve biases.
    ///
    /// Mirrors `PersonalitySystem.register_input` from the Python source
    /// exactly, including the `np.sign` convention (0.0 when abs ≤ 0.01).
    pub fn register_input(&mut self, weights: &[f64], stimuli: &[f64]) {
        for i in 0..self.n {
            let w = weights[i];
            let resp = if i < stimuli.len() { stimuli[i] } else { 0.0 };
            let target = self.bias_targets[i];
            let rig = self.rigidity[i];

            let diff = target - w;
            let direction_to_target = if diff.abs() > 0.01 { diff.signum() } else { 0.0 };
            let response_direction = if resp == 0.0 { 0.0 } else { resp.signum() };

            let is_contrary = direction_to_target != 0.0
                && response_direction != 0.0
                && direction_to_target != response_direction;

            if is_contrary {
                self.contrary_pressure[i] += resp.abs();
                let conflict_delta = resp.abs() * rig * self.conflict_gain;
                self.conflict_level[i] =
                    (self.conflict_level[i] * 0.95 + conflict_delta * 0.1).min(1.0);
                self.volatility_injection[i] = self.conflict_level[i] * 0.02;
            } else {
                self.contrary_pressure[i] *= 0.98;
                self.conflict_level[i] *= 0.92;
                self.volatility_injection[i] *= 0.9;
            }

            // Bias evolution: if contrary pressure exceeds threshold, nudge
            // target toward the current weight.
            let threshold = rig * 10.0;
            if self.contrary_pressure[i] > threshold {
                let shift = self.evolution_rate * (w - target) * (1.0 - rig * 0.5);
                self.bias_targets[i] += shift;
                self.rigidity[i] = (self.rigidity[i] - 0.00005).max(0.1);
            }
        }
    }

    /// Inject Gaussian noise on aspects that are experiencing internal conflict.
    ///
    /// Mutates `stimuli` in-place; matches `apply_conflict_volatility` from
    /// the Python source (`r += normal(0, vol)` when `vol > 0.001`).
    pub fn apply_conflict_volatility(&self, stimuli: &mut [f64], rng: &mut Rng) {
        for i in 0..self.n {
            let vol = self.volatility_injection[i];
            if vol > 0.001 && i < stimuli.len() {
                stimuli[i] += rng.normal(0.0, vol);
            }
        }
    }

    /// The six canonical profile names.
    pub fn available_profiles() -> &'static [&'static str] {
        &[
            "contemplative",
            "action-oriented",
            "empathic",
            "analytical",
            "resilient",
            "seeker",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_index(names: &[&str]) -> HashMap<String, usize> {
        names.iter().enumerate().map(|(i, &n)| (n.to_string(), i)).collect()
    }

    #[test]
    fn unknown_profile_returns_none() {
        let idx = make_index(&["agency"]);
        assert!(PersonalitySystem::from_profile("unknown", &idx, 1).is_none());
    }

    #[test]
    fn all_profiles_construct() {
        let aspects = [
            "introspection", "reflection", "metacognition", "self-monitoring",
            "temporal_awareness", "moral_awareness", "self-concept", "agency",
            "motivation", "goal-setting", "self-efficacy", "situational_awareness",
            "self-regulation", "self-development", "emotional_awareness",
            "social_awareness", "theory_of_mind", "self-esteem", "self-recognition",
        ];
        let idx = make_index(&aspects);
        let n = aspects.len();
        for &name in PersonalitySystem::available_profiles() {
            let ps = PersonalitySystem::from_profile(name, &idx, n)
                .unwrap_or_else(|| panic!("profile {name} failed to construct"));
            assert_eq!(ps.profile_name, name);
            assert_eq!(ps.n, n);
            assert_eq!(ps.bias_targets.len(), n);
            assert_eq!(ps.rigidity.len(), n);
        }
    }

    #[test]
    fn compute_biases_formula() {
        let idx = make_index(&["agency"]);
        let mut ps = PersonalitySystem::from_profile("contemplative", &idx, 1).unwrap();
        // agency target for contemplative is -0.1; rigidity absent → DEFAULT 0.5
        ps.bias_targets[0] = -0.1;
        ps.rigidity[0] = 0.5;
        let weights = [0.3_f64];
        let mut out = [0.0_f64];
        ps.compute_biases(&weights, &mut out);
        let expected = 0.5 * 0.05 * (-0.1 - 0.3);
        assert!((out[0] - expected).abs() < 1e-12, "got {}, expected {}", out[0], expected);
    }

    #[test]
    fn contemplative_targets_and_rigidity() {
        let aspects = ["introspection", "reflection", "metacognition", "agency"];
        let idx = make_index(&aspects);
        let ps = PersonalitySystem::from_profile("contemplative", &idx, 4).unwrap();
        assert!((ps.bias_targets[0] - 0.7).abs() < 1e-12); // introspection
        assert!((ps.bias_targets[3] - (-0.1)).abs() < 1e-12); // agency
        assert!((ps.rigidity[0] - 0.7).abs() < 1e-12);   // introspection rigidity
        assert!((ps.rigidity[2] - 0.65).abs() < 1e-12);  // metacognition rigidity
        assert!((ps.rigidity[3] - 0.3).abs() < 1e-12);   // agency rigidity
    }

    #[test]
    fn resilient_high_rigidity() {
        let aspects = ["self-regulation", "self-esteem", "self-concept"];
        let idx = make_index(&aspects);
        let ps = PersonalitySystem::from_profile("resilient", &idx, 3).unwrap();
        for i in 0..3 {
            assert!((ps.rigidity[i] - 0.8).abs() < 1e-12);
        }
    }

    #[test]
    fn seeker_low_rigidity() {
        let aspects = ["self-development", "agency"];
        let idx = make_index(&aspects);
        let ps = PersonalitySystem::from_profile("seeker", &idx, 2).unwrap();
        assert!((ps.rigidity[0] - 0.5).abs() < 1e-12); // self-development
        assert!((ps.rigidity[1] - 0.3).abs() < 1e-12); // agency
    }

    #[test]
    fn register_input_contrary_accumulates() {
        // agency target = -0.1 (contemplative), weight = 0.5 → target is below weight.
        // direction_to_target = sign(-0.1 - 0.5) = -1
        // stimuli positive → response_direction = +1 → contrary
        let aspects = ["agency"];
        let idx = make_index(&aspects);
        let mut ps = PersonalitySystem::from_profile("contemplative", &idx, 1).unwrap();
        let weights = [0.5_f64];
        let stimuli = [0.3_f64];
        ps.register_input(&weights, &stimuli);
        assert!(ps.contrary_pressure[0] > 0.0);
        assert!(ps.conflict_level[0] > 0.0);
        assert!(ps.volatility_injection[0] > 0.0);
    }

    #[test]
    fn register_input_non_contrary_decays() {
        let aspects = ["agency"];
        let idx = make_index(&aspects);
        let mut ps = PersonalitySystem::from_profile("contemplative", &idx, 1).unwrap();
        // Seed some prior state.
        ps.contrary_pressure[0] = 1.0;
        ps.conflict_level[0] = 0.5;
        ps.volatility_injection[0] = 0.05;
        // weight below target (-0.1 > -0.5), so direction_to_target = +1
        // stimuli also positive → aligned, not contrary
        let weights = [-0.5_f64];
        let stimuli = [0.3_f64];
        ps.register_input(&weights, &stimuli);
        assert!((ps.contrary_pressure[0] - 0.98).abs() < 1e-9);
        assert!((ps.conflict_level[0] - 0.5 * 0.92).abs() < 1e-9);
        assert!((ps.volatility_injection[0] - 0.05 * 0.9).abs() < 1e-9);
    }

    #[test]
    fn apply_conflict_volatility_mutates_when_high_vol() {
        let aspects = ["agency"];
        let idx = make_index(&aspects);
        let mut ps = PersonalitySystem::from_profile("contemplative", &idx, 1).unwrap();
        ps.volatility_injection[0] = 0.05; // well above 0.001 threshold
        let mut stimuli = [1.0_f64];
        let original = stimuli[0];
        let mut rng = Rng::new(42);
        ps.apply_conflict_volatility(&mut stimuli, &mut rng);
        // With nonzero noise the value should (with overwhelming probability) differ.
        assert_ne!(stimuli[0], original);
    }

    #[test]
    fn apply_conflict_volatility_skips_when_low_vol() {
        let aspects = ["agency"];
        let idx = make_index(&aspects);
        let ps = PersonalitySystem::from_profile("contemplative", &idx, 1).unwrap();
        // volatility_injection initialised to 0.0, below threshold
        let mut stimuli = [1.0_f64];
        let mut rng = Rng::new(42);
        ps.apply_conflict_volatility(&mut stimuli, &mut rng);
        assert_eq!(stimuli[0], 1.0);
    }

    #[test]
    fn available_profiles_has_six() {
        assert_eq!(PersonalitySystem::available_profiles().len(), 6);
    }

    #[test]
    fn original_targets_unchanged_after_evolution() {
        let aspects = ["self-development", "motivation"];
        let idx = make_index(&aspects);
        let mut ps = PersonalitySystem::from_profile("seeker", &idx, 2).unwrap();
        let orig = ps.original_targets.clone();
        // Drive contrary pressure past threshold to trigger evolution.
        ps.contrary_pressure[0] = 999.0;
        let weights = [0.0_f64, 0.0];
        let stimuli = [-0.5_f64, -0.5];
        ps.register_input(&weights, &stimuli);
        // bias_targets may have shifted, originals must not.
        assert_eq!(ps.original_targets, orig);
    }
}
