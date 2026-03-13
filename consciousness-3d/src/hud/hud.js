// hud.js — Root HUD controller

import { MiniPanel } from './miniPanel.js';
import { DetailPanel } from './detailPanel.js';

export class HUD {
  constructor() {
    this._miniPanel = new MiniPanel();
    this._detailPanel = new DetailPanel();

    const container = document.getElementById('hud');
    if (container) {
      container.appendChild(this._detailPanel.element);
      container.appendChild(this._miniPanel.element);
    }

    this._lastUpdateTime = 0;
    this._throttleMs = 100; // 10 Hz
  }

  update(store) {
    const now = performance.now();
    if (now - this._lastUpdateTime < this._throttleMs) return;
    this._lastUpdateTime = now;

    this._miniPanel.update(store);
    if (store.hudExpanded) {
      this._detailPanel.update(store);
    }
  }

  toggle(store) {
    store.hudExpanded = !store.hudExpanded;
    if (store.hudExpanded) {
      this._detailPanel.show();
      this._detailPanel.update(store);
    } else {
      this._detailPanel.hide();
    }
  }
}
