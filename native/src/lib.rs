#![deny(clippy::all)]
#![allow(clippy::needless_pass_by_value)]

use napi_derive::napi;
use std::sync::Mutex;

pub mod analyzer;
pub mod behavior;
pub mod config;
pub mod dynamics;
pub mod energy;
pub mod engine;
pub mod environment;
pub mod interrelations;
pub mod memory;
pub mod personality;
pub mod rng;
pub mod serialize;

use config::SimConfig;
use engine::ConsciousnessEngine;

static ENGINE: Mutex<Option<ConsciousnessEngine>> = Mutex::new(None);

#[napi]
pub fn create_engine(expanded: bool, personality: Option<String>) -> String {
    let config = if expanded {
        SimConfig::expanded_32()
    } else {
        SimConfig::default_20()
    };
    let mut engine = ConsciousnessEngine::new(config);
    if let Some(name) = &personality {
        engine.set_personality(Some(name.as_str()));
    }
    let init = serialize::serialize_init(
        &engine.config,
        engine.personality.as_ref().map(|p| p.profile_name.as_str()),
    );
    *ENGINE.lock().unwrap() = Some(engine);
    serde_json::to_string(&init).unwrap_or_default()
}

#[napi]
pub fn tick() -> String {
    let mut guard = ENGINE.lock().unwrap();
    let engine = guard.as_mut().expect("Engine not created — call createEngine first");
    let result = engine.step();
    let val = serialize::serialize_tick(&result, &engine.config);
    serde_json::to_string(&val).unwrap_or_default()
}

#[napi]
pub fn inject_input(direction: String) {
    if direction == "none" { return; }
    let mut guard = ENGINE.lock().unwrap();
    if let Some(engine) = guard.as_mut() {
        engine.inject_input(&direction);
    }
}

#[napi]
pub fn set_personality(name: Option<String>) {
    let mut guard = ENGINE.lock().unwrap();
    if let Some(engine) = guard.as_mut() {
        engine.set_personality(name.as_deref());
    }
}

#[napi]
pub fn get_traits() -> String {
    let guard = ENGINE.lock().unwrap();
    let engine = guard.as_ref().expect("Engine not created");
    let mut traits = serde_json::Map::new();
    for (i, name) in engine.config.aspects.iter().enumerate() {
        let lr = engine.dynamics().lr_mut_ref().get(i).copied().unwrap_or(0.02);
        let mu = engine.dynamics().mu_mut_ref().get(i).copied().unwrap_or(0.8);
        let w = engine.weights_ref().get(i).copied().unwrap_or(0.0);
        traits.insert(name.clone(), serde_json::json!({
            "weight": (w * 1e4).round() / 1e4,
            "lr": (lr * 1e4).round() / 1e4,
            "momentum": (mu * 1e4).round() / 1e4,
        }));
    }
    serde_json::to_string(&traits).unwrap_or_default()
}

#[napi]
pub fn randomize_weights(seed: Option<f64>) {
    let mut guard = ENGINE.lock().unwrap();
    if let Some(engine) = guard.as_mut() {
        let s = seed.unwrap_or(0.0) as u64;
        let actual_seed = if s == 0 {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
        } else {
            s
        };
        let mut rng = rng::Rng::new(actual_seed);
        engine.randomize_weights(&mut rng);
    }
}
