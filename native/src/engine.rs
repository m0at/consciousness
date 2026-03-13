//! ConsciousnessEngine — central orchestrator wiring all subsystems together.
//!
//! Mirrors the Python ConsciousnessEngine in core/engine.py exactly in terms
//! of subsystem order and arithmetic.

use crate::analyzer::{Attractor, SystemAnalyzer, TickResults};
use crate::behavior::{compute_behavior, BehaviorResult};
use crate::config::SimConfig;
use crate::dynamics::{DynamicsEngine, UpdateInputs};
use crate::dynamics::SimConfig as DynConfig;
use crate::energy::{EnergySystem, EnergyState};
use crate::environment::Environment;
use crate::interrelations::InterrelationMatrix;
use crate::memory::MemorySystem;
use crate::personality::PersonalitySystem;
use crate::rng::Rng;

// ── TickResult ────────────────────────────────────────────────────────────────

pub struct AttractorSnapshot {
    pub basin_radius: f64,
    pub strength: f64,
    pub drift_rate: f64,
}

pub struct TickResult {
    pub tick: u64,
    pub weights: Vec<f64>,
    pub energy: EnergyState,
    pub analysis: TickResults,
    pub attractors: Vec<AttractorSnapshot>,
    pub env_status: String,
    pub behavior: BehaviorResult,
}

// ── ConsciousnessEngine ───────────────────────────────────────────────────────

pub struct ConsciousnessEngine {
    pub config: SimConfig,
    pub personality: Option<PersonalitySystem>,
    tick: u64,
    weights: Vec<f64>,
    initial_abs_sum: f64,
    interrelation: InterrelationMatrix,
    dynamics: DynamicsEngine,
    environment: Environment,
    analyzer: SystemAnalyzer,
    memory: MemorySystem,
    energy: EnergySystem,
    rng: Rng,
    prev_stress: f64,
    current_behavior: String,
}

impl ConsciousnessEngine {
    pub fn new(config: SimConfig) -> Self {
        let n = config.n;
        let initial_weights = config.initial_weights.clone();
        let initial_abs_sum: f64 = initial_weights.iter().map(|w| w.abs()).sum();

        // Interrelation matrix
        let mut interrelation = InterrelationMatrix::new(n);
        interrelation.build(&config.interrelation_triples);

        // Dynamics engine
        let dyn_cfg = DynConfig {
            homeostasis_rate: config.homeostasis_rate,
            hysteresis_threshold: config.hysteresis_threshold,
            hysteresis_resistance: config.hysteresis_resistance,
        };
        let mut dynamics = DynamicsEngine::new(n);
        dynamics.init(
            &initial_weights,
            &config.learning_rates,
            &config.momentum,
            &dyn_cfg,
        );

        // Environment
        let environment = Environment::new(n);

        // Build flat interrelation data for the analyzer (n×n row-major)
        let interrelation_data: Vec<f64> = interrelation.matrix().to_vec();

        // Analyzer
        let analyzer = SystemAnalyzer::new(
            config.aspects.clone(),
            interrelation_data,
            50,
            0.02,
            0.005,
        );

        // Memory
        let memory = MemorySystem::new(
            config.aspects.clone(),
            config.memory_stm_size,
            config.memory_ltm_capacity,
            config.memory_influence_scale,
            200.0, // decay_half_life in frames
        );

        // Energy
        let energy = EnergySystem::new(
            config.aspects.clone(),
            &initial_weights,
            config.energy_budget,
            config.attention_slots,
            config.circadian_period,
        );

        // Personality (built later via set_personality)
        let personality = None;

        Self {
            tick: 0,
            weights: initial_weights,
            initial_abs_sum,
            interrelation,
            dynamics,
            environment,
            analyzer,
            memory,
            energy,
            personality,
            rng: Rng::new(42),
            prev_stress: 0.0,
            current_behavior: String::new(),
            config,
        }
    }

    /// Run one simulation tick. Returns a `TickResult`.
    pub fn step(&mut self) -> TickResult {
        self.tick += 1;
        let n = self.config.n;

        // 1. Energy ─────────────────────────────────────────────────────────
        let energy_state = self.energy.step(&self.weights);

        // 2. Environment: stimuli ───────────────────────────────────────────
        let mut stimuli = self.environment.generate_stimuli(
            &self.weights,
            self.tick,
            &mut self.rng,
            &self.config.aspect_index,
        );

        // Scale stimuli by per-aspect energy scales and global noise modifier
        for i in 0..n {
            stimuli[i] *= energy_state.aspect_scales[i];
        }
        for s in &mut stimuli {
            *s *= energy_state.noise_modifier;
        }

        // 3. Personality ────────────────────────────────────────────────────
        let mut personality_biases: Option<Vec<f64>> = None;
        if let Some(ps) = &mut self.personality {
            ps.register_input(&self.weights, &stimuli);
            ps.apply_conflict_volatility(&mut stimuli, &mut self.rng);
            let mut biases = vec![0.0f64; n];
            ps.compute_biases(&self.weights, &mut biases);
            personality_biases = Some(biases);
        }

        // 4. Interrelation matrix ───────────────────────────────────────────
        let mut rebalanced = vec![0.0f64; n];
        self.interrelation.propagate(&self.weights, &mut rebalanced);

        // 5. Memory influences ──────────────────────────────────────────────
        let memory_influences = self.memory.compute_influences(&self.weights);

        // 6. Dynamics update ────────────────────────────────────────────────
        let energy_mods: Vec<f64> = vec![energy_state.lr_modifier; n];
        let inputs = UpdateInputs {
            energy_modifiers: Some(&energy_mods),
            personality_biases: personality_biases.as_deref(),
            memory_influences: Some(&memory_influences),
        };
        let mut updated = vec![0.0f64; n];
        self.dynamics.update(&rebalanced, &stimuli, n, inputs, &mut updated);

        // Scale to preserve total magnitude
        let current_abs: f64 = updated.iter().map(|w| w.abs()).sum();
        if current_abs > 1e-12 && self.initial_abs_sum > 1e-12 {
            let factor = self.initial_abs_sum / current_abs;
            for w in &mut updated {
                *w *= factor;
            }
        }
        self.weights = updated;

        // 7. Memory: record frame ───────────────────────────────────────────
        self.memory.process_frame(&self.weights, &stimuli);

        // 8. Analyzer ───────────────────────────────────────────────────────
        let analysis = self.analyzer.tick_update(self.weights.clone());

        // Capture attractor snapshots (last 3)
        let attractors: Vec<AttractorSnapshot> = self.analyzer.attractors()
            .iter()
            .rev()
            .take(3)
            .map(|a: &Attractor| AttractorSnapshot {
                basin_radius: a.basin_radius,
                strength: a.strength,
                drift_rate: a.drift_rate,
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        // Behavior scoring ──────────────────────────────────────────────────
        let current_stress = energy_state.stress;
        let stress_spike = (current_stress - self.prev_stress).max(0.0);
        let behavior = compute_behavior(
            &self.weights,
            &self.config.aspect_index,
            current_stress,
            stress_spike,
            energy_state.energy_pct,
            &self.current_behavior,
        );
        self.current_behavior = behavior.primary.clone();
        self.prev_stress = current_stress;

        // Environment status string
        let env_status = self.environment.get_status();

        TickResult {
            tick: self.tick,
            weights: self.weights.clone(),
            energy: energy_state,
            analysis,
            attractors,
            env_status,
            behavior,
        }
    }

    /// Inject a user input direction ("positive" or "negative").
    pub fn inject_input(&mut self, direction: &str) {
        self.environment.apply_input(direction, &self.config.aspect_index);
        self.analyzer.mark_perturbation();
        self.energy.inject_perturbation(0.3);
    }

    /// Change the personality profile. Pass `None` to disable.
    pub fn set_personality(&mut self, name: Option<&str>) {
        self.personality = name.and_then(|n| {
            PersonalitySystem::from_profile(n, &self.config.aspect_index, self.config.n)
        });
    }

    /// Reset to initial weights and zero tick counter.
    pub fn reset(&mut self) {
        self.weights = self.config.initial_weights.clone();
        self.tick = 0;
        self.prev_stress = 0.0;
        self.current_behavior = String::new();
        self.dynamics.reset();
    }

    pub fn weights_ref(&self) -> &[f64] { &self.weights }
    pub fn dynamics(&self) -> &crate::dynamics::DynamicsEngine { &self.dynamics }

    /// Randomize weights AND per-aspect learning rates and momentum,
    /// creating a unique personality with different response dynamics.
    pub fn randomize_weights(&mut self, rng: &mut crate::rng::Rng) {
        // Randomize starting weights — these are the initial biases
        for w in self.weights.iter_mut() {
            *w = rng.uniform(-0.8, 0.8);
        }
        // Randomize learning rates (how fast each aspect changes)
        // Range: 0.005 to 0.05 — some aspects will be very reactive, others glacial
        for lr in self.dynamics.lr_mut().iter_mut() {
            *lr = rng.uniform(0.005, 0.05);
        }
        // Randomize momentum (how much inertia each aspect has)
        // Range: 0.4 to 0.96 — some aspects flip easily, others carry trajectory
        for mu in self.dynamics.mu_mut().iter_mut() {
            *mu = rng.uniform(0.4, 0.96);
        }
        self.current_behavior = String::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimConfig;

    #[test]
    fn engine_runs_100_ticks() {
        let config = SimConfig::default_20();
        let mut engine = ConsciousnessEngine::new(config);
        for _ in 0..100 {
            let result = engine.step();
            assert!(!result.behavior.primary.is_empty());
            assert!(result.energy.energy_pct.is_finite());
        }
    }
}
