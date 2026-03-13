import * as THREE from 'three';

// HSL colour stops for key light valence shift
// valence = -1  →  H=220, S=50%, L=45%   (cool blue)
// valence =  0  →  H=38,  S=40%, L=55%   (neutral warm)
// valence = +1  →  H=35,  S=60%, L=55%   (warm amber)
const KEY_NEG = { h: 220 / 360, s: 0.50, l: 0.45 };
const KEY_NEU = { h:  38 / 360, s: 0.40, l: 0.55 };
const KEY_POS = { h:  35 / 360, s: 0.60, l: 0.55 };

function lerpHSL(a, b, t) {
  return {
    h: a.h + (b.h - a.h) * t,
    s: a.s + (b.s - a.s) * t,
    l: a.l + (b.l - a.l) * t,
  };
}

export function createLights(scene) {
  // ── Static lights ─────────────────────────────────────────────────────────

  const ambient = new THREE.AmbientLight(0x9090b0, 0.8);
  scene.add(ambient);

  const keyLight = new THREE.DirectionalLight(0xffeedd, 1.8);
  keyLight.position.set(3, 4, 2);
  keyLight.castShadow = true;
  keyLight.shadow.mapSize.width  = 1024;
  keyLight.shadow.mapSize.height = 1024;
  scene.add(keyLight);

  const fillLight = new THREE.PointLight(0x8090c0, 0.8);
  fillLight.position.set(-3, 3, -1);
  scene.add(fillLight);

  const hemiLight = new THREE.HemisphereLight(0x8888bb, 0x444444, 0.6);
  scene.add(hemiLight);

  // ── Dynamic lights (start at intensity 0) ────────────────────────────────

  // Stress pulse: dim red overhead point light
  const stressPulse = new THREE.PointLight(0xff2020, 0);
  stressPulse.position.set(0, 3, 0);
  scene.add(stressPulse);

  // Flow spotlight: subtle overhead cone
  const flowSpot = new THREE.SpotLight(0x6080c0, 0);
  flowSpot.position.set(0, 3.4, 0);
  flowSpot.target.position.set(0, 0, 0);
  flowSpot.angle = Math.PI / 4;        // 45°
  flowSpot.penumbra = 0.2;
  scene.add(flowSpot);
  scene.add(flowSpot.target);

  // Base intensities (before energy/circadian scale)
  const BASE_AMBIENT = 0.8;
  const BASE_HEMI    = 0.6;
  const BASE_KEY     = 1.8;
  const BASE_FILL    = 0.8;

  // Working THREE.Color for HSL manipulation
  const keyColor = new THREE.Color();

  // ── update(store) — called every frame ───────────────────────────────────
  function update(store) {
    const state = store.interpolated;
    if (!state) return;

    const valence   = state.valence       ?? 0;
    const stress    = state.energy?.stress    ?? 0;
    const energyPct = state.energy?.energy_pct ?? 100;
    const flowStates = state.energy?.flow_states ?? [];
    const circadian  = state.energy?.circadian   ?? 1.0;
    const time       = performance.now() / 1000;

    // ── Energy scale (0.5 – 1.0) ───────────────────────────────────────────
    const energyScale = 0.7 + 0.3 * (energyPct / 100);

    // ── Circadian scale ────────────────────────────────────────────────────
    const circScale = circadian;           // already [0.5, 1.0] per spec

    // ── Ambient intensity: base × energy × circadian + stress pulse ────────
    // Section 4.3: 0.3 + 0.15 * stress * sin(time * 3.0)
    ambient.intensity = (BASE_AMBIENT + 0.15 * stress * Math.sin(time * 3.0))
                        * energyScale * circScale;

    // ── Hemisphere ─────────────────────────────────────────────────────────
    hemiLight.intensity = BASE_HEMI * energyScale * circScale;

    // ── Key light colour (valence HSL shift) ───────────────────────────────
    let hsl;
    if (valence >= 0) {
      hsl = lerpHSL(KEY_NEU, KEY_POS, valence);
    } else {
      hsl = lerpHSL(KEY_NEU, KEY_NEG, -valence);
    }
    keyColor.setHSL(hsl.h, hsl.s, hsl.l);
    keyLight.color.copy(keyColor);
    keyLight.intensity = BASE_KEY * energyScale;

    // ── Fill light ─────────────────────────────────────────────────────────
    fillLight.intensity = BASE_FILL * energyScale;

    // ── Stress pulse point light ───────────────────────────────────────────
    // Fades in only above stress > 0.6
    if (stress > 0.6) {
      stressPulse.intensity = 0.2 * stress;
    } else {
      stressPulse.intensity = 0;
    }

    // ── Flow spotlight ─────────────────────────────────────────────────────
    if (flowStates.length > 0) {
      flowSpot.intensity = 0.2;
    } else {
      flowSpot.intensity = 0;
    }
  }

  return { ambient, keyLight, fillLight, hemiLight, stressPulse, flowSpot, update };
}
