/// Behavior scoring — multidimensional mapping from all 20 consciousness
/// weights to observable behaviors. Every aspect contributes to multiple
/// behaviors through cluster scoring.

use std::collections::HashMap;

pub const HYSTERESIS: f64 = 0.04;

pub const BEHAVIOR_NAMES: [&str; 14] = [
    "idle_calm",           // regulated, balanced, at peace
    "idle_restless",       // aroused but undirected
    "pacing_ruminate",     // introspective walking, head down
    "pacing_energized",    // motivated purposeful walking
    "sitting_contemplate", // deep thought at chair
    "sitting_dejected",    // low self-esteem, withdrawn
    "standing_confident",  // upright, chest out, looking around
    "standing_vigilant",   // high situational awareness, scanning
    "gesture_social",      // animated social gesturing
    "gesture_eureka",      // insight moment — sudden stop, look up
    "wander_curious",      // exploring the room, looking at things
    "wander_anxious",      // pacing but erratic, can't settle
    "retreat_corner",      // withdrawn, moves to edge of room
    "startle",             // stress spike
];

pub struct BehaviorResult {
    pub primary: String,
    pub scores: Vec<(String, f64)>,
    pub intensity: f64,
}

#[inline(always)]
fn neg(x: f64) -> f64 { if x < 0.0 { -x } else { 0.0 } }

#[inline(always)]
fn pos(x: f64) -> f64 { if x > 0.0 { x } else { 0.0 } }

#[inline(always)]
fn clamp01(x: f64) -> f64 { x.clamp(0.0, 1.0) }

#[inline(always)]
fn get(weights: &[f64], idx: &HashMap<String, usize>, name: &str) -> f64 {
    idx.get(name).map(|&i| weights[i]).unwrap_or(0.0)
}

pub fn compute_behavior(
    weights: &[f64],
    aspect_index: &HashMap<String, usize>,
    stress: f64,
    stress_spike: f64,
    energy_pct: f64,
    current: &str,
) -> BehaviorResult {
    let w = |name: &str| -> f64 { get(weights, aspect_index, name) };

    // ── Composite dimensions (clusters of related weights) ──────────────
    // Each composite captures a psychological dimension from multiple aspects

    // COGNITIVE: thinking, analyzing, understanding
    let cognitive = (w("introspection") + w("reflection") + w("metacognition")
                     + w("temporal_awareness")) / 4.0;

    // EMOTIONAL: feeling intensity (absolute value — both positive and negative)
    let emotional = (w("emotional_awareness").abs() + w("self-esteem").abs()
                     + w("self-concept").abs()) / 3.0;

    // SOCIAL: outward orientation toward others
    let social = (w("social_awareness") + w("theory_of_mind")
                  + w("moral_awareness")) / 3.0;

    // DRIVE: motivation and directed energy
    let drive = (w("motivation") + w("agency") + w("goal-setting")
                 + w("self-efficacy")) / 4.0;

    // REGULATION: self-control, composure
    let regulation = (w("self-regulation") + w("self-monitoring")) / 2.0;

    // AWARENESS: environmental scanning
    let awareness = (w("situational_awareness") + w("body_awareness")
                     + w("self-recognition")) / 3.0;

    // SELF-WORTH: positive self-regard
    let self_worth = (w("self-esteem") + w("self-concept")
                      + w("self-efficacy")) / 3.0;

    // GROWTH: development orientation
    let growth = (w("self-development") + w("goal-setting")
                  + w("metacognition")) / 3.0;

    let energy_frac = (energy_pct / 100.0).clamp(0.0, 1.0);

    // ── Score each behavior from the composites ─────────────────────────
    // Each behavior is driven by a COMBINATION of dimensions

    // Calm idle: high regulation + low emotional intensity + low drive
    let idle_calm = clamp01(
        0.2 + 0.3 * pos(regulation) + 0.2 * (1.0 - emotional)
        - 0.2 * drive.abs() - 0.1 * stress
    );

    // Restless idle: high emotion + low regulation + moderate drive
    let idle_restless = clamp01(
        0.1 + 0.25 * emotional + 0.15 * stress
        - 0.2 * regulation + 0.1 * drive.abs()
        + 0.1 * w("body_awareness")
    );

    // Pacing with rumination: high cognitive + high introspection + moderate drive
    let pacing_ruminate = clamp01(
        0.05 + 0.25 * pos(cognitive) + 0.2 * pos(w("introspection"))
        + 0.15 * w("reflection").abs()
        - 0.1 * pos(social) + 0.1 * drive.abs()
    );

    // Energized pacing: high drive + high agency + moderate energy
    let pacing_energized = clamp01(
        0.05 + 0.25 * pos(drive) + 0.2 * pos(w("motivation"))
        + 0.15 * pos(w("agency")) + 0.1 * energy_frac
        - 0.1 * neg(w("motivation"))
    );

    // Seated contemplation: high cognitive + low drive + moderate regulation
    let sitting_contemplate = clamp01(
        0.05 + 0.25 * pos(cognitive) + 0.2 * pos(w("reflection"))
        + 0.15 * pos(w("metacognition"))
        - 0.15 * pos(drive) - 0.1 * stress
    );

    // Dejected sitting: low self-worth + low motivation + low energy
    let sitting_dejected = clamp01(
        0.05 + 0.25 * neg(self_worth) + 0.2 * neg(w("motivation"))
        + 0.15 * neg(w("agency")) + 0.1 * stress
        + 0.1 * (1.0 - energy_frac)
    );

    // Confident standing: high self-worth + high agency + high regulation
    let standing_confident = clamp01(
        0.05 + 0.25 * pos(self_worth) + 0.2 * pos(w("agency"))
        + 0.15 * pos(regulation) + 0.1 * pos(w("self-efficacy"))
        - 0.1 * stress
    );

    // Vigilant standing: high awareness + high situational + moderate stress
    let standing_vigilant = clamp01(
        0.05 + 0.25 * pos(awareness) + 0.2 * pos(w("situational_awareness"))
        + 0.15 * stress + 0.1 * pos(regulation)
        - 0.1 * pos(cognitive)
    );

    // Social gesturing: high social + high emotional + high agency
    let gesture_social = clamp01(
        0.05 + 0.25 * pos(social) + 0.2 * pos(w("theory_of_mind"))
        + 0.15 * pos(w("emotional_awareness")) + 0.1 * pos(w("agency"))
        + 0.1 * pos(w("moral_awareness"))
    );

    // Eureka/insight: high metacognition spike + high growth + moderate arousal
    let gesture_eureka = clamp01(
        0.0 + 0.3 * pos(w("metacognition")) + 0.2 * pos(growth)
        + 0.15 * pos(w("self-development")) + 0.1 * pos(w("self-efficacy"))
        - 0.15 * neg(w("metacognition"))
    );

    // Curious wandering: high awareness + moderate drive + positive emotion
    let wander_curious = clamp01(
        0.05 + 0.2 * pos(awareness) + 0.15 * pos(drive)
        + 0.15 * pos(w("situational_awareness"))
        + 0.1 * pos(self_worth) + 0.1 * energy_frac
        - 0.1 * pos(cognitive)
    );

    // Anxious wandering: high emotion + low regulation + moderate awareness
    let wander_anxious = clamp01(
        0.05 + 0.2 * emotional + 0.2 * neg(regulation)
        + 0.15 * stress + 0.1 * awareness.abs()
        - 0.1 * pos(self_worth)
    );

    // Retreat: very low self-worth + low social + high negative emotion
    let retreat_corner = clamp01(
        0.0 + 0.25 * neg(self_worth) + 0.2 * neg(social)
        + 0.15 * neg(w("social_awareness"))
        + 0.1 * neg(w("self-esteem")) + 0.1 * stress
        - 0.15 * pos(w("agency"))
    );

    // Startle: stress spike
    let startle = clamp01(
        0.5 * stress_spike + 0.3 * pos(w("situational_awareness"))
        + 0.2 * pos(w("body_awareness"))
    );

    let raw: [f64; 14] = [
        idle_calm, idle_restless, pacing_ruminate, pacing_energized,
        sitting_contemplate, sitting_dejected, standing_confident,
        standing_vigilant, gesture_social, gesture_eureka,
        wander_curious, wander_anxious, retreat_corner, startle,
    ];

    // ── Hysteresis ──────────────────────────────────────────────────────
    let best_idx = raw.iter().enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i).unwrap_or(0);

    let new_behavior_idx = if current.is_empty() {
        best_idx
    } else {
        let cur_idx = BEHAVIOR_NAMES.iter().position(|&n| n == current)
            .unwrap_or(usize::MAX);
        if cur_idx == usize::MAX || cur_idx >= 14 {
            best_idx
        } else {
            if raw[best_idx] >= raw[cur_idx] + HYSTERESIS { best_idx } else { cur_idx }
        }
    };

    let primary = BEHAVIOR_NAMES[new_behavior_idx].to_string();
    let intensity = (raw[new_behavior_idx] * 1e4).round() / 1e4;

    let scores: Vec<(String, f64)> = BEHAVIOR_NAMES.iter().zip(raw.iter())
        .map(|(&name, &score)| (name.to_string(), (score * 1e4).round() / 1e4))
        .collect();

    BehaviorResult { primary, scores, intensity }
}
