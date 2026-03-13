function lerp(a, b, t) {
  return a + (b - a) * t;
}

function clamp(v, lo, hi) {
  return Math.max(lo, Math.min(hi, v));
}

export function interpolate(store, timestamp) {
  const { previousTick, latestTick, latestTickReceivedAt } = store;
  if (!latestTick) return;

  // If no previous tick yet, just use latest values directly
  const prev = previousTick || latestTick;

  const elapsed = timestamp - latestTickReceivedAt;
  const alpha = clamp(elapsed / 16, 0, 1);

  // Interpolate per-aspect weights
  const prevW = prev.weights ?? {};
  const nextW = latestTick.weights ?? {};
  for (const aspect of Object.keys(nextW)) {
    const a = prevW[aspect] ?? nextW[aspect] ?? 0;
    const b = nextW[aspect] ?? 0;
    store.interpolated.weights[aspect] = lerp(a, b, alpha);
  }

  // Interpolate energy as a sub-object (lighting.js reads store.interpolated.energy.*)
  const prevE = prev.energy ?? {};
  const nextE = latestTick.energy ?? {};

  store.interpolated.stress    = lerp(prevE.stress ?? 0, nextE.stress ?? 0, alpha);
  store.interpolated.arousal   = lerp(prevE.arousal ?? 0, nextE.arousal ?? 0, alpha);
  store.interpolated.circadian = lerp(prevE.circadian ?? 1, nextE.circadian ?? 1, alpha);
  store.interpolated.valence   = lerp(prev.valence ?? 0, latestTick.valence ?? 0, alpha);

  const energyPct = lerp(prevE.energy_pct ?? 100, nextE.energy_pct ?? 100, alpha);

  // Build the energy sub-object that lighting.js and HUD expect
  store.interpolated.energy = {
    energy_pct: energyPct,
    stress: store.interpolated.stress,
    arousal: store.interpolated.arousal,
    circadian: store.interpolated.circadian,
    flow_states: nextE.flow_states ?? [],
    attended: nextE.attended ?? [],
  };
}
