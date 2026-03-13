// miniPanel.js — Compact always-visible HUD overlay (bottom-left)

function standardDeviation(vals) {
  const n = vals.length;
  if (n === 0) return 0;
  const mean = vals.reduce((s, v) => s + v, 0) / n;
  const variance = vals.reduce((s, v) => s + (v - mean) ** 2, 0) / n;
  return Math.sqrt(variance);
}

function detectMentalState(weights, stress, avgDelta) {
  const vals = Object.values(weights);
  if (vals.length === 0) return { label: 'CALM', color: '#DAA520' };
  const std = standardDeviation(vals);
  const mean = vals.reduce((s, v) => s + Math.abs(v), 0) / vals.length;
  if (std > 0.45 && avgDelta > 0.02) return { label: 'AGITATED', color: '#E8613C' };
  if (mean > 0.3 && avgDelta > 0.005) return { label: 'GROWING', color: '#3CB371' };
  if (avgDelta < -0.005) return { label: 'RECOVERING', color: '#4A90D9' };
  if (std > 0.35) return { label: 'CONFLICTED', color: '#9B59B6' };
  return { label: 'CALM', color: '#DAA520' };
}

function makeBar(value, max, chars) {
  const filled = Math.round((value / max) * chars);
  const clamped = Math.max(0, Math.min(chars, filled));
  return '█'.repeat(clamped) + '░'.repeat(chars - clamped);
}

function formatSign(val) {
  const fixed = Math.abs(val).toFixed(2);
  return (val >= 0 ? '+' : '-') + fixed;
}

export class MiniPanel {
  constructor() {
    const el = document.createElement('div');
    el.style.cssText = [
      'position: fixed',
      'bottom: 16px',
      'left: 16px',
      'background: rgba(13,17,23,0.75)',
      'border: 1px solid rgba(255,255,255,0.1)',
      'border-radius: 8px',
      'padding: 12px 16px',
      'font-family: "SF Mono","Fira Code",monospace',
      'font-size: 12px',
      'color: #aaaaaa',
      'white-space: pre',
      'line-height: 1.6',
      'pointer-events: none',
      'z-index: 100',
    ].join(';');

    // Row: mental state indicator
    const rowState = document.createElement('div');
    rowState.style.cssText = 'display:flex;align-items:center;gap:6px;margin-bottom:2px';

    const dot = document.createElement('span');
    dot.textContent = '●';
    dot.style.fontSize = '10px';

    const stateLabel = document.createElement('span');
    const tickLabel = document.createElement('span');
    tickLabel.style.marginLeft = 'auto';

    rowState.appendChild(dot);
    rowState.appendChild(stateLabel);
    rowState.appendChild(tickLabel);

    // Row: energy bar
    const rowEnergy = document.createElement('div');
    // Row: stress bar
    const rowStress = document.createElement('div');
    // Row: valence
    const rowValence = document.createElement('div');
    // Row: behavior
    const rowBehavior = document.createElement('div');
    // Row: hint
    const rowHint = document.createElement('div');
    rowHint.style.color = '#555555';
    rowHint.textContent = '[Tab] for details';

    el.appendChild(rowState);
    el.appendChild(rowEnergy);
    el.appendChild(rowStress);
    el.appendChild(rowValence);
    el.appendChild(rowBehavior);
    el.appendChild(rowHint);

    this.element = el;
    this._dot = dot;
    this._stateLabel = stateLabel;
    this._tickLabel = tickLabel;
    this._rowEnergy = rowEnergy;
    this._rowStress = rowStress;
    this._rowValence = rowValence;
    this._rowBehavior = rowBehavior;
    this._prevWeights = null;
  }

  update(store) {
    const tick = store.latestTick;
    const interp = store.interpolated;

    const weights = interp.weights && Object.keys(interp.weights).length > 0
      ? interp.weights
      : (tick && tick.weights ? tick.weights : {});

    // Compute avgDelta from weight change between ticks
    let avgDelta = 0;
    if (this._prevWeights && Object.keys(weights).length > 0) {
      const keys = Object.keys(weights);
      const deltas = keys.map(k => (weights[k] || 0) - (this._prevWeights[k] || 0));
      avgDelta = deltas.reduce((s, v) => s + v, 0) / keys.length;
    }
    this._prevWeights = Object.assign({}, weights);

    const stress = interp.stress != null ? interp.stress
      : (interp.energy && typeof interp.energy === 'object' ? interp.energy.stress
      : (tick && tick.energy ? tick.energy.stress : 0));
    const energyPct = interp.energy != null
      ? (typeof interp.energy === 'object' ? (interp.energy.energy_pct ?? 0) : interp.energy)
      : (tick && tick.energy ? tick.energy.energy_pct : 0);
    const valence = interp.valence != null ? interp.valence : (tick ? tick.valence : 0);
    const tickNum = tick ? tick.tick : 0;
    const behavior = store.behavior ? store.behavior.current : 'idle_stand';

    const mental = detectMentalState(weights, stress, avgDelta);

    this._dot.style.color = mental.color;
    this._stateLabel.style.color = mental.color;
    this._stateLabel.textContent = mental.label;
    this._tickLabel.textContent = 'tick ' + tickNum;

    const energyBar = makeBar(energyPct, 100, 10);
    this._rowEnergy.textContent = 'energy ' + energyBar + '  ' + Math.round(energyPct) + '%';

    const stressBar = makeBar(stress, 1, 10);
    this._rowStress.textContent = 'stress ' + stressBar + '  ' + stress.toFixed(2);

    this._rowValence.textContent = 'valence         ' + formatSign(valence);
    this._rowBehavior.textContent = 'behavior: ' + behavior;
  }
}
