export class Store {
  constructor() {
    this.latestTick = null;
    this.previousTick = null;
    this.latestTickReceivedAt = 0;

    this.interpolated = {
      weights: {},
      energy: 0,
      stress: 0,
      valence: 0,
      arousal: 0,
      circadian: 1.0,
    };

    this.behavior = {
      current: 'idle_stand',
      intensity: 0.0,
      scores: {},
    };

    this.activeEvents = [];

    this.character = {
      position: { x: 0, y: 0.92, z: 0 },
      targetPosition: { x: 0, y: 0, z: 0 },
      facingAngle: 0,
    };

    this.init = null;
    this.hudExpanded = false;
  }

  handleMessage(msg) {
    if (!msg || !msg.type) return;

    switch (msg.type) {
      case 'init':
      case 'ready':
        this.init = msg;
        break;

      case 'tick': {
        this.previousTick = this.latestTick;
        this.latestTick = msg;
        this.latestTickReceivedAt = performance.now();

        const b = msg.behavior;
        if (b) {
          this.behavior.current = b.primary ?? this.behavior.current;
          this.behavior.intensity = b.intensity ?? 0.0;
          this.behavior.scores = b.scores ?? {};
        }
        break;
      }

      default:
        break;
    }
  }

  getState() {
    return {
      latestTick: this.latestTick,
      previousTick: this.previousTick,
      interpolated: this.interpolated,
      behavior: this.behavior,
      activeEvents: this.activeEvents,
      character: this.character,
      init: this.init,
      hudExpanded: this.hudExpanded,
    };
  }
}
