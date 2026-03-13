// detailPanel.js — Expanded HUD detail view (left side, Tab toggle)

const CATEGORY_COLORS = {
  cognitive:   '#4A90D9',
  emotional:   '#E8613C',
  social:      '#3CB371',
  executive:   '#9B59B6',
  existential: '#DAA520',
};

function makeBar(weight, chars) {
  const filled = Math.round(Math.abs(weight) * chars);
  const clamped = Math.max(0, Math.min(chars, filled));
  return '█'.repeat(clamped);
}

function formatSign(val) {
  return (val >= 0 ? '+' : '') + val.toFixed(2);
}

function makeSectionHeader(text) {
  const el = document.createElement('div');
  el.style.cssText = [
    'color: #666666',
    'margin: 10px 0 4px 0',
    'font-size: 11px',
    'letter-spacing: 0.08em',
  ].join(';');
  el.textContent = '\u2500\u2500\u2500 ' + text + ' \u2500\u2500\u2500';
  return el;
}

function makeRow(label, value) {
  const el = document.createElement('div');
  el.style.cssText = 'display:flex;justify-content:space-between;gap:8px;margin:1px 0';
  const lbl = document.createElement('span');
  lbl.style.color = '#777777';
  lbl.textContent = label;
  const val = document.createElement('span');
  val.textContent = value;
  el.appendChild(lbl);
  el.appendChild(val);
  return { el, lbl, val };
}

export class DetailPanel {
  constructor() {
    const el = document.createElement('div');
    el.style.cssText = [
      'position: fixed',
      'top: 16px',
      'left: 16px',
      'width: 380px',
      'height: calc(100vh - 32px)',
      'background: rgba(13,17,23,0.75)',
      'border: 1px solid rgba(255,255,255,0.1)',
      'border-radius: 8px',
      'font-family: "SF Mono","Fira Code",monospace',
      'font-size: 12px',
      'color: #aaaaaa',
      'overflow-y: auto',
      'pointer-events: auto',
      'z-index: 99',
      'display: none',
    ].join(';');

    const scroll = document.createElement('div');
    scroll.style.cssText = 'padding:12px 16px;max-height:calc(100vh - 64px);overflow-y:auto';

    // Header section
    const hdrTitle = document.createElement('div');
    hdrTitle.style.cssText = 'color:#cccccc;font-size:13px;font-weight:bold;margin-bottom:4px';
    hdrTitle.textContent = 'CONSCIOUSNESS SIMULATION';

    const hdrPersonality = document.createElement('div');
    const hdrMeta = document.createElement('div');
    hdrMeta.style.cssText = 'display:flex;justify-content:space-between';

    const hdrTick = document.createElement('span');
    const hdrCircadian = document.createElement('span');
    hdrMeta.appendChild(hdrTick);
    hdrMeta.appendChild(hdrCircadian);

    scroll.appendChild(hdrTitle);
    scroll.appendChild(hdrPersonality);
    scroll.appendChild(hdrMeta);

    // Aspects section
    scroll.appendChild(makeSectionHeader('ASPECT WEIGHTS'));
    const aspectsContainer = document.createElement('div');
    scroll.appendChild(aspectsContainer);

    // Energy section
    scroll.appendChild(makeSectionHeader('ENERGY'));
    const energyContainer = document.createElement('div');
    scroll.appendChild(energyContainer);

    // Environment section
    scroll.appendChild(makeSectionHeader('ENVIRONMENT'));
    const envContainer = document.createElement('div');
    scroll.appendChild(envContainer);

    // Behavior section
    scroll.appendChild(makeSectionHeader('BEHAVIOR'));
    const behaviorContainer = document.createElement('div');
    scroll.appendChild(behaviorContainer);

    // Memory section
    scroll.appendChild(makeSectionHeader('MEMORY'));
    const memoryContainer = document.createElement('div');
    scroll.appendChild(memoryContainer);

    el.appendChild(scroll);
    this.element = el;

    this._hdrPersonality = hdrPersonality;
    this._hdrTick = hdrTick;
    this._hdrCircadian = hdrCircadian;
    this._aspectsContainer = aspectsContainer;
    this._energyContainer = energyContainer;
    this._envContainer = envContainer;
    this._behaviorContainer = behaviorContainer;
    this._memoryContainer = memoryContainer;
  }

  show() { this.element.style.display = 'block'; }
  hide() { this.element.style.display = 'none'; }

  update(store) {
    const tick = store.latestTick;
    const interp = store.interpolated;
    const init = store.init;

    const personality = init ? init.personality : 'unknown';
    const tickNum = tick ? tick.tick : 0;
    const circadian = interp.circadian != null ? interp.circadian
      : (tick && tick.energy ? tick.energy.circadian : 1.0);

    this._hdrPersonality.textContent = 'personality: ' + personality;
    this._hdrTick.textContent = 'tick: ' + tickNum;
    this._hdrCircadian.textContent = 'circadian: ' + circadian.toFixed(2);

    // Build category-to-color and aspect-to-category maps from init
    const categoryMap = {}; // aspect -> category name
    const catColors = (init && init.categoryColors) ? init.categoryColors : CATEGORY_COLORS;
    if (init && init.categories) {
      for (const [cat, aspects] of Object.entries(init.categories)) {
        for (const a of aspects) categoryMap[a] = cat;
      }
    }

    // Aspect weights
    const weights = interp.weights && Object.keys(interp.weights).length > 0
      ? interp.weights
      : (tick && tick.weights ? tick.weights : {});

    const sortedAspects = Object.entries(weights)
      .sort((a, b) => Math.abs(b[1]) - Math.abs(a[1]));

    this._aspectsContainer.textContent = '';
    for (const [aspect, value] of sortedAspects) {
      const cat = categoryMap[aspect] || 'cognitive';
      const color = catColors[cat] || '#aaaaaa';
      const row = document.createElement('div');
      row.style.cssText = 'display:flex;align-items:center;gap:4px;margin:1px 0';

      const dot = document.createElement('span');
      dot.textContent = '●';
      dot.style.cssText = 'color:' + color + ';font-size:9px;flex-shrink:0';

      const name = document.createElement('span');
      name.style.cssText = 'flex:0 0 22ch;overflow:hidden;text-overflow:ellipsis;color:#888888';
      name.textContent = aspect;

      const val = document.createElement('span');
      val.style.cssText = 'flex:0 0 6ch;text-align:right;' + (value >= 0 ? 'color:#aaaaaa' : 'color:#888888');
      val.textContent = formatSign(value);

      const bar = document.createElement('span');
      bar.style.cssText = 'color:' + color + ';opacity:0.8';
      bar.textContent = '  ' + makeBar(value, 10);

      row.appendChild(dot);
      row.appendChild(name);
      row.appendChild(val);
      row.appendChild(bar);
      this._aspectsContainer.appendChild(row);
    }

    // Energy
    const energy = tick ? tick.energy : null;
    const energyPct = interp.energy != null
      ? (typeof interp.energy === 'object' ? (interp.energy.energy_pct ?? 0) : interp.energy)
      : (energy ? energy.energy_pct : 0);
    const arousal = interp.arousal != null ? interp.arousal
      : (interp.energy && typeof interp.energy === 'object' ? interp.energy.arousal
      : (energy ? energy.arousal : 0));
    const stress = interp.stress != null ? interp.stress
      : (interp.energy && typeof interp.energy === 'object' ? interp.energy.stress
      : (energy ? energy.stress : 0));
    const flowStates = energy && energy.flow_states ? energy.flow_states : [];
    const attended = energy && energy.attended ? energy.attended : [];

    this._energyContainer.textContent = '';
    const addEnergyRow = (lbl, txt) => {
      const d = document.createElement('div');
      d.style.margin = '1px 0';
      const s = document.createElement('span');
      s.style.color = '#666666';
      s.textContent = lbl + ' ';
      const v = document.createElement('span');
      v.textContent = txt;
      d.appendChild(s);
      d.appendChild(v);
      this._energyContainer.appendChild(d);
    };

    addEnergyRow('energy   ', Math.round(energyPct) + '%');
    addEnergyRow('arousal  ', arousal.toFixed(2));
    addEnergyRow('stress   ', stress.toFixed(2));
    addEnergyRow('circadian', circadian.toFixed(2));
    addEnergyRow('flow:    ', flowStates.length > 0 ? flowStates.join(', ') : 'none');
    addEnergyRow('attended:', attended.length > 0 ? attended.join(', ') : 'none');

    // Environment
    const valence = interp.valence != null ? interp.valence : (tick ? tick.valence : 0);
    const activeStimuli = tick ? (tick.activeStimuli || []) : [];
    const entropy = tick && tick.analysis && tick.analysis.entropy
      ? tick.analysis.entropy : null;

    this._envContainer.textContent = '';
    const addEnvRow = (lbl, txt) => {
      const d = document.createElement('div');
      d.style.margin = '1px 0';
      const s = document.createElement('span');
      s.style.color = '#666666';
      s.textContent = lbl + ' ';
      const v = document.createElement('span');
      v.textContent = txt;
      d.appendChild(s);
      d.appendChild(v);
      this._envContainer.appendChild(d);
    };

    addEnvRow('valence:', formatSign(valence));
    addEnvRow('active: ', activeStimuli.length > 0 ? activeStimuli.join(', ') : 'none');
    if (entropy) {
      addEnvRow('entropy:', entropy.shannon.toFixed(2) + ' (' + (entropy.complexity_label || '') + ')');
    } else {
      addEnvRow('entropy:', 'N/A');
    }

    // Behavior
    const behavior = store.behavior || { current: 'idle_stand', intensity: 0, scores: {} };
    const scores = behavior.scores || {};
    const sortedScores = Object.entries(scores).sort((a, b) => b[1] - a[1]);

    this._behaviorContainer.textContent = '';

    const primaryRow = document.createElement('div');
    primaryRow.style.cssText = 'margin:2px 0;color:#cccccc';
    const pLbl = document.createElement('span');
    pLbl.style.color = '#666666';
    pLbl.textContent = 'primary: ';
    const pVal = document.createElement('span');
    pVal.textContent = behavior.current + ' (' + (behavior.intensity || 0).toFixed(2) + ')';
    primaryRow.appendChild(pLbl);
    primaryRow.appendChild(pVal);
    this._behaviorContainer.appendChild(primaryRow);

    for (const [name, score] of sortedScores) {
      if (name === behavior.current) continue;
      const row = document.createElement('div');
      row.style.margin = '1px 0';
      const n = document.createElement('span');
      n.style.cssText = 'display:inline-block;width:20ch;color:#777777';
      n.textContent = name;
      const s = document.createElement('span');
      s.textContent = score.toFixed(2) + '  ' + makeBar(score, 6);
      row.appendChild(n);
      row.appendChild(s);
      this._behaviorContainer.appendChild(row);
    }

    // Memory (from tick if available)
    this._memoryContainer.textContent = '';
    const memory = tick && tick.memory ? tick.memory : null;

    const addMemRow = (lbl, txt) => {
      const d = document.createElement('div');
      d.style.margin = '1px 0';
      const s = document.createElement('span');
      s.style.color = '#666666';
      s.textContent = lbl + ' ';
      const v = document.createElement('span');
      v.textContent = txt;
      d.appendChild(s);
      d.appendChild(v);
      this._memoryContainer.appendChild(d);
    };

    if (memory) {
      addMemRow('STM buffer:', (memory.stm_size || 'N/A') + '/' + (memory.stm_capacity || 50));
      addMemRow('LTM count: ', (memory.ltm_count || 'N/A') + '/' + (memory.ltm_capacity || 500));
      if (memory.strongest) {
        addMemRow('strongest: ', memory.strongest.label + ' (str=' + memory.strongest.strength.toFixed(1) + ')');
      }
    } else {
      addMemRow('STM buffer:', 'N/A');
      addMemRow('LTM count: ', 'N/A');
    }
  }
}
