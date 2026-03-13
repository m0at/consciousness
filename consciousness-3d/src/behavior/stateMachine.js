import { BEHAVIORS } from './behaviors.js';
import { CHAIR_POS } from '../scene/room.js';

export const STATE = {
  IDLE:       'IDLE',
  WALKING:    'WALKING',
  PERFORMING: 'PERFORMING',
};

const CORNERS = [
  { x: -3.0, z: -2.0 },
  { x:  3.0, z: -2.0 },
  { x: -3.0, z:  2.0 },
  { x:  3.0, z:  2.0 },
];

const CENTER = { x: 0, z: 0 };

function dist2D(ax, az, bx, bz) {
  const dx = bx - ax, dz = bz - az;
  return Math.sqrt(dx * dx + dz * dz);
}

function randomBoundaryPoint() {
  const side = Math.floor(Math.random() * 4);
  switch (side) {
    case 0: return { x: -3.5 + Math.random() * 7, z:  2.5 };
    case 1: return { x: -3.5 + Math.random() * 7, z: -2.5 };
    case 2: return { x: -3.5, z: -2.5 + Math.random() * 5 };
    case 3: return { x:  3.5, z: -2.5 + Math.random() * 5 };
  }
}

export class StateMachine {
  constructor(locomotion) {
    this._locomotion = locomotion;
    this._state = STATE.IDLE;
    this._currentBehavior = 'idle_calm';
    this._walkDwell = 0;
    this._walkDwelling = false;
  }

  get state() { return this._state; }

  update(store, dt) {
    const behavior = store.behavior.current;
    const meta = BEHAVIORS[behavior] ?? BEHAVIORS['idle_calm'];
    const pos = store.character.position;

    // Behavior changed?
    const changed = behavior !== this._currentBehavior;
    if (changed) this._currentBehavior = behavior;

    switch (this._state) {

      case STATE.IDLE: {
        if (meta.isWalking) {
          this._startWalking(store, behavior, pos);
          break;
        }
        if (meta.requiresChair) {
          const d = dist2D(pos.x, pos.z, CHAIR_POS.x, CHAIR_POS.z);
          if (d > 0.4) {
            this._locomotion.setTarget(CHAIR_POS.x, CHAIR_POS.z);
            this._state = STATE.WALKING;
          } else {
            this._state = STATE.PERFORMING;
          }
          break;
        }
        // Standing behaviors — just perform in place
        this._state = STATE.PERFORMING;
        break;
      }

      case STATE.WALKING: {
        // Update locomotion
        if (this._walkDwelling) {
          this._walkDwell -= dt;
          if (this._walkDwell <= 0) {
            this._walkDwelling = false;
            this._pickNextWalkTarget(store, behavior, pos);
          }
          break;
        }

        if (this._locomotion.active) {
          this._locomotion.update(store, dt);
        }

        // Behavior changed to something non-walking?
        if (changed && !meta.isWalking) {
          this._locomotion.stop();
          this._state = STATE.IDLE;
          break;
        }

        // Arrived at target?
        if (this._locomotion.arrived || !this._locomotion.active) {
          if (meta.isWalking) {
            // Walking behavior — dwell briefly then pick new target
            this._walkDwelling = true;
            this._walkDwell = 0.5 + Math.random() * 2.0;
          } else {
            // Was walking to chair or specific spot
            this._state = STATE.PERFORMING;
          }
        }
        break;
      }

      case STATE.PERFORMING: {
        if (changed) {
          if (meta.isWalking) {
            this._startWalking(store, behavior, pos);
          } else if (meta.requiresChair) {
            const d = dist2D(pos.x, pos.z, CHAIR_POS.x, CHAIR_POS.z);
            if (d > 0.4) {
              this._locomotion.setTarget(CHAIR_POS.x, CHAIR_POS.z);
              this._state = STATE.WALKING;
            }
            // else already at chair, stay performing
          } else {
            // Standing behavior — stay in PERFORMING, just change what we're doing
            // (animation change handled by app.js)
          }
        }
        break;
      }
    }
  }

  _startWalking(store, behavior, pos) {
    this._state = STATE.WALKING;
    this._walkDwelling = false;
    this._pickNextWalkTarget(store, behavior, pos);
  }

  _pickNextWalkTarget(store, behavior, pos) {
    if (behavior === 'retreat_corner') {
      // Pick the farthest corner from current position
      let best = CORNERS[0], bestD = 0;
      for (const c of CORNERS) {
        const d = dist2D(pos.x, pos.z, c.x, c.z);
        if (d > bestD) { bestD = d; best = c; }
      }
      this._locomotion.setTarget(best.x, best.z);
    } else if (behavior === 'wander_curious') {
      // Pick a random point in the room, biased toward objects
      const targets = [
        { x: -2.0, z: -2.0 },  // table
        { x:  2.5, z: -1.5 },  // chair
        { x:  0, z:  2.0 },    // near camera wall
        { x: -3.0, z:  0 },    // left wall
        { x:  3.0, z:  0 },    // right wall
        randomBoundaryPoint(),
      ];
      const t = targets[Math.floor(Math.random() * targets.length)];
      this._locomotion.setTarget(t.x, t.z);
    } else if (behavior === 'wander_anxious') {
      // Rapid direction changes — pick random nearby point
      const t = {
        x: pos.x + (Math.random() - 0.5) * 4,
        z: pos.z + (Math.random() - 0.5) * 3,
      };
      t.x = Math.max(-3.5, Math.min(3.5, t.x));
      t.z = Math.max(-2.5, Math.min(2.5, t.z));
      this._locomotion.setTarget(t.x, t.z);
    } else {
      // pacing_ruminate, pacing_energized — boundary-to-boundary
      const t = randomBoundaryPoint();
      this._locomotion.setTarget(t.x, t.z);
    }
  }
}
