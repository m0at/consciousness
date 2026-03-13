//! Serialization layer — converts engine state to JSON matching DESIGN.md §3.1.

use serde_json::{json, Value};

use crate::analyzer::TickResults;
use crate::config::SimConfig;
use crate::engine::{AttractorSnapshot, TickResult};

// ── Label maps ────────────────────────────────────────────────────────────────

fn phase_label(phase: u8) -> &'static str {
    match phase {
        0 => "growth",
        1 => "stability",
        2 => "crisis",
        3 => "recovery",
        _ => "stability",
    }
}

fn entropy_label(label: u8) -> &'static str {
    match label {
        0 => "settled",
        1 => "active",
        2 => "turbulent",
        _ => "settled",
    }
}

// ── Round helpers ─────────────────────────────────────────────────────────────

#[inline]
fn r4(x: f64) -> f64 {
    (x * 1e4).round() / 1e4
}

#[inline]
fn r6(x: f64) -> f64 {
    (x * 1e6).round() / 1e6
}

#[inline]
fn r3(x: f64) -> f64 {
    (x * 1e3).round() / 1e3
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Serialize the startup handshake message (type: "init") per DESIGN.md §3.1.
pub fn serialize_init(config: &SimConfig, personality: Option<&str>) -> Value {
    // Build category dict: category_name → [aspect_name, ...]
    // config.categories stores category → Vec<usize> (indices)
    let mut categories = serde_json::Map::new();
    for (cat, indices) in &config.categories {
        let members: Vec<Value> = indices
            .iter()
            .filter_map(|&i| config.aspects.get(i))
            .map(|name| Value::String(name.clone()))
            .collect();
        categories.insert(cat.clone(), Value::Array(members));
    }

    // category_colors: category_name → hex string
    let mut category_colors = serde_json::Map::new();
    for (cat, color) in &config.category_colors {
        category_colors.insert(cat.clone(), Value::String(color.clone()));
    }

    json!({
        "type": "init",
        "aspects": config.aspects,
        "categories": categories,
        "categoryColors": category_colors,
        "personality": personality,
        "aspectCount": config.n,
    })
}

/// Serialize a tick result to JSON matching DESIGN.md §3.1 "tick" message.
pub fn serialize_tick(tick_result: &TickResult, config: &SimConfig) -> Value {
    // weights: aspect_name → f64 (4dp)
    let mut weights = serde_json::Map::new();
    for (i, name) in config.aspects.iter().enumerate() {
        weights.insert(name.clone(), json!(r4(tick_result.weights[i])));
    }

    // energy object
    let e = &tick_result.energy;
    let energy = json!({
        "energy": e.energy,
        "energy_pct": r4(e.energy_pct),
        "arousal": r4(e.arousal),
        "stress": r4(e.stress),
        "attended": e.attended,
        "flow_states": e.flow_states,
        "circadian": r4(e.circadian),
    });

    // analysis
    let analysis = serialize_analysis(&tick_result.analysis, &tick_result.attractors, config);

    // envStatus, activeStimuli, valence
    let env_status = &tick_result.env_status;
    let (active_stimuli, valence) = parse_env_status(env_status);

    // behavior
    let mut scores_map = serde_json::Map::new();
    for (name, score) in &tick_result.behavior.scores {
        scores_map.insert(name.clone(), json!(*score));
    }
    let behavior = json!({
        "primary": tick_result.behavior.primary,
        "scores": scores_map,
        "intensity": tick_result.behavior.intensity,
    });

    json!({
        "type": "tick",
        "tick": tick_result.tick,
        "weights": weights,
        "energy": energy,
        "analysis": analysis,
        "envStatus": env_status,
        "activeStimuli": active_stimuli,
        "valence": r6(valence),
        "behavior": behavior,
    })
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn serialize_analysis(results: &TickResults, attractors: &[AttractorSnapshot], config: &SimConfig) -> Value {
    // phases
    let phases: Value = match &results.phases {
        Some(ps) => {
            let arr: Vec<Value> = ps.iter().map(|p| {
                let aspect_name = config.aspects.get(p.aspect_idx)
                    .map(String::as_str)
                    .unwrap_or("");
                json!({
                    "phase": phase_label(p.phase),
                    "aspect": aspect_name,
                    "confidence": r3(p.confidence),
                    "duration": p.duration,
                    "slope": r6(p.slope),
                })
            }).collect();
            Value::Array(arr)
        }
        None => Value::Array(vec![]),
    };

    // entropy
    let entropy: Value = match &results.entropy {
        Some(e) => json!({
            "shannon": r4(e.shannon),
            "normalized": r4(e.normalized),
            "delta": r6(e.delta),
            "complexity_label": entropy_label(e.label),
        }),
        None => Value::Null,
    };

    // cascades — last 5 from the tick's new cascades
    let cascades: Value = match &results.cascades {
        Some(cs) => {
            let tail = if cs.len() > 5 { &cs[cs.len() - 5..] } else { cs.as_slice() };
            let arr: Vec<Value> = tail.iter().map(|c| {
                let path: Vec<Value> = c.path.iter().map(|(aspect_idx, delta, depth)| {
                    let aspect_name = config.aspects.get(*aspect_idx)
                        .map(String::as_str)
                        .unwrap_or("");
                    json!([aspect_name, r4(*delta), depth])
                }).collect();
                json!({
                    "trigger": config.aspects.get(c.trigger_idx).map(String::as_str).unwrap_or(""),
                    "trigger_delta": r4(c.trigger_delta),
                    "path": path,
                    "magnitude": r4(c.total_magnitude),
                    "tick": c.tick,
                })
            }).collect();
            Value::Array(arr)
        }
        None => Value::Array(vec![]),
    };

    // attractors — passed in from TickResult (last 3 snapshots)
    let attractors_val: Value = {
        let arr: Vec<Value> = attractors.iter().map(|a| json!({
            "basin_radius": r4(a.basin_radius),
            "strength": r4(a.strength),
            "drift_rate": r6(a.drift_rate),
        })).collect();
        Value::Array(arr)
    };

    // resilience
    let resilience: Value = match &results.resilience {
        Some(r) => json!({
            "displacement": r4(r.displacement),
            "elapsed": r.elapsed,
            "elasticity": r4(r.elasticity),
            "recovered": r.recovered,
        }),
        None => Value::Null,
    };

    json!({
        "phases": phases,
        "entropy": entropy,
        "cascades": cascades,
        "attractors": attractors_val,
        "resilience": resilience,
    })
}

/// Parse "stimulus_a, stimulus_b | valence=+0.12" into (stimuli, valence).
fn parse_env_status(status: &str) -> (Vec<String>, f64) {
    let mut active_stimuli: Vec<String> = Vec::new();
    let mut valence = 0.0f64;

    if status.is_empty() {
        return (active_stimuli, valence);
    }

    let parts: Vec<&str> = status.splitn(2, " | ").collect();
    let stimuli_part = parts[0].trim();
    if !stimuli_part.is_empty() {
        for s in stimuli_part.split(',') {
            let s = s.trim();
            // Filter out "valence=..." in case there's no " | " separator
            if !s.is_empty() && !s.starts_with("valence=") {
                active_stimuli.push(s.to_string());
            }
        }
    }

    if let Some(rest) = parts.get(1) {
        for segment in rest.split('|') {
            if let Some(pos) = segment.find("valence=") {
                let val_str = &segment[pos + 8..];
                // Take up to the first non-numeric char
                let end = val_str.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '+' && c != '-')
                    .unwrap_or(val_str.len());
                if let Ok(v) = val_str[..end].parse::<f64>() {
                    valence = v;
                }
            }
        }
        // Also check the part directly if there's a valence= in the joined rest
        if let Some(pos) = rest.find("valence=") {
            let val_str = &rest[pos + 8..];
            let end = val_str.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '+' && c != '-')
                .unwrap_or(val_str.len());
            if let Ok(v) = val_str[..end].parse::<f64>() {
                valence = v;
            }
        }
    }

    (active_stimuli, valence)
}
