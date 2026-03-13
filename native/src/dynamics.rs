//! Dynamics engine: momentum, adaptive learning rates, nonlinear bounding,
//! homeostasis, and hysteresis.
//!
//! Mirrors the Python `DynamicsEngine` in core/dynamics.py exactly, including
//! all fallback constants and the conditioning update rule.

pub struct SimConfig {
    pub homeostasis_rate: f64,
    pub hysteresis_threshold: f64,
    pub hysteresis_resistance: f64,
}

/// Optional per-step additive/multiplicative inputs for `DynamicsEngine::update`.
pub struct UpdateInputs<'a> {
    /// Multiplicative learning-rate scalars (length n). `None` means all 1.0.
    pub energy_modifiers: Option<&'a [f64]>,
    /// Additive gradient bias from personality profile (length n). `None` means all 0.
    pub personality_biases: Option<&'a [f64]>,
    /// Additive gradient bias from memory system (length n). `None` means all 0.
    pub memory_influences: Option<&'a [f64]>,
}

pub struct DynamicsEngine {
    velocity: Vec<f64>,
    conditioning: Vec<f64>,
    resting: Vec<f64>,
    lr: Vec<f64>,
    mu: Vec<f64>,
    homeostasis_rate: f64,
    hysteresis_threshold: f64,
    hysteresis_resistance: f64,
}

impl DynamicsEngine {
    /// Create an uninitialised engine sized for `n` aspects.
    /// Call `init` before using `update`.
    pub fn new(n: usize) -> Self {
        Self {
            velocity: vec![0.0; n],
            conditioning: vec![0.0; n],
            resting: vec![0.0; n],
            lr: vec![0.02; n],
            mu: vec![0.80; n],
            homeostasis_rate: 0.005,
            hysteresis_threshold: 0.3,
            hysteresis_resistance: 0.7,
        }
    }

    /// Populate per-aspect parameters from slices and a `SimConfig`.
    ///
    /// `resting_weights`, `learning_rates`, and `momentum` must all be length
    /// `n`.  Velocities and conditioning are reset to 0.
    pub fn init(
        &mut self,
        resting_weights: &[f64],
        learning_rates: &[f64],
        momentum: &[f64],
        config: &SimConfig,
    ) {
        let n = self.velocity.len();
        self.resting[..n].copy_from_slice(&resting_weights[..n]);
        self.lr[..n].copy_from_slice(&learning_rates[..n]);
        self.mu[..n].copy_from_slice(&momentum[..n]);
        self.homeostasis_rate = config.homeostasis_rate;
        self.hysteresis_threshold = config.hysteresis_threshold;
        self.hysteresis_resistance = config.hysteresis_resistance;
        self.velocity.iter_mut().for_each(|v| *v = 0.0);
        self.conditioning.iter_mut().for_each(|c| *c = 0.0);
    }

    /// Apply one dynamics step.
    ///
    /// # Arguments
    /// * `rebalanced` – energy-rebalanced weights (input, length n)
    /// * `stimuli`    – per-aspect gradient signals (length n)
    /// * `n`          – number of aspects to process
    /// * `inputs`     – optional energy/personality/memory modifiers
    /// * `out`        – updated weights written here (length n)
    ///
    /// # Formula (per aspect i)
    /// ```text
    /// lr      = self.lr[i] * energy_modifiers[i]   (if present)
    /// grad    = stimuli[i] + personality_biases[i] + memory_influences[i]
    /// v       = mu[i] * velocity[i] + lr * grad
    /// w_new   = rebalanced[i] - v
    /// displ   = w_new - resting[i]
    /// pull    = homeostasis_rate * displ
    /// if conditioning[i] > threshold:
    ///     resistance = min(hysteresis_resistance, cond / (cond + 1.0))
    ///     pull      *= 1.0 - resistance
    /// w_new  -= pull
    /// w_new   = tanh(w_new)
    /// abs_d   = |w_new - resting[i]|
    /// if abs_d > threshold * 0.5:
    ///     conditioning[i] += 0.01 * abs_d
    /// else:
    ///     conditioning[i] *= 0.995
    /// out[i]  = w_new
    /// ```
    pub fn update(
        &mut self,
        rebalanced: &[f64],
        stimuli: &[f64],
        n: usize,
        inputs: UpdateInputs<'_>,
        out: &mut [f64],
    ) {
        for i in 0..n {
            let mut grad = stimuli[i];

            // Adaptive learning rate
            let mut lr = self.lr[i];
            if let Some(em) = inputs.energy_modifiers {
                lr *= em[i];
            }

            // Additive gradient terms
            if let Some(pb) = inputs.personality_biases {
                grad += pb[i];
            }
            if let Some(mi) = inputs.memory_influences {
                grad += mi[i];
            }

            // Momentum update
            let mu = self.mu[i];
            let v = mu * self.velocity[i] + lr * grad;
            self.velocity[i] = v;
            let mut w_new = rebalanced[i] - v;

            // Homeostasis: pull toward resting state
            let rest = self.resting[i];
            let displacement = w_new - rest;
            let mut pull = self.homeostasis_rate * displacement;
            let cond = self.conditioning[i];
            if cond > self.hysteresis_threshold {
                let resistance = self.hysteresis_resistance.min(cond / (cond + 1.0));
                pull *= 1.0 - resistance;
            }
            w_new -= pull;

            // Nonlinear bound via tanh
            w_new = w_new.tanh();

            // Conditioning update (habit formation)
            let abs_disp = (w_new - rest).abs();
            if abs_disp > self.hysteresis_threshold * 0.5 {
                self.conditioning[i] = cond + 0.01 * abs_disp;
            } else {
                self.conditioning[i] = cond * 0.995;
            }

            out[i] = w_new;
        }
    }

    /// Reset velocities and conditioning to zero (keep all other parameters).
    pub fn reset(&mut self) {
        self.velocity.iter_mut().for_each(|v| *v = 0.0);
        self.conditioning.iter_mut().for_each(|c| *c = 0.0);
    }

    // ── Accessors ──────────────────────────────────────────────────────────

    pub fn velocity(&self) -> &[f64] {
        &self.velocity
    }

    pub fn conditioning(&self) -> &[f64] {
        &self.conditioning
    }

    pub fn lr_mut(&mut self) -> &mut [f64] {
        &mut self.lr
    }

    pub fn mu_mut(&mut self) -> &mut [f64] {
        &mut self.mu
    }
}
