import { CHAIR_POS } from '../scene/room.js';

const WALKABLE = { minX: -3.5, maxX: 3.5, minZ: -2.5, maxZ: 2.5 };
const CHAIR_EXCL_RADIUS = 0.5;
const CHAIR_WAYPOINT_OFFSET = 0.7;
const ARRIVAL_DIST = 0.2;
const FACING_RATE = 4.0;

function clamp(v, lo, hi) {
  return v < lo ? lo : v > hi ? hi : v;
}

function dist2D(ax, az, bx, bz) {
  const dx = bx - ax, dz = bz - az;
  return Math.sqrt(dx * dx + dz * dz);
}

// Returns the closest point on segment (ax,az)-(bx,bz) to point (px,pz).
function closestPointOnSegment(ax, az, bx, bz, px, pz) {
  const dx = bx - ax, dz = bz - az;
  const lenSq = dx * dx + dz * dz;
  if (lenSq < 1e-10) return { x: ax, z: az };
  const t = clamp(((px - ax) * dx + (pz - az) * dz) / lenSq, 0, 1);
  return { x: ax + t * dx, z: az + t * dz };
}

function linePassesNearChair(ax, az, bx, bz) {
  const cp = closestPointOnSegment(ax, az, bx, bz, CHAIR_POS.x, CHAIR_POS.z);
  return dist2D(cp.x, cp.z, CHAIR_POS.x, CHAIR_POS.z) < CHAIR_EXCL_RADIUS;
}

function computeWaypoints(fromX, fromZ, toX, toZ) {
  if (!linePassesNearChair(fromX, fromZ, toX, toZ)) {
    return [{ x: toX, z: toZ }];
  }
  // Perpendicular to (to - from), offset from chair center
  const dx = toX - fromX, dz = toZ - fromZ;
  const len = Math.sqrt(dx * dx + dz * dz) || 1;
  const perpX = -dz / len;
  const perpZ =  dx / len;
  const wpX = CHAIR_POS.x + perpX * CHAIR_WAYPOINT_OFFSET;
  const wpZ = CHAIR_POS.z + perpZ * CHAIR_WAYPOINT_OFFSET;
  return [
    { x: clamp(wpX, WALKABLE.minX, WALKABLE.maxX), z: clamp(wpZ, WALKABLE.minZ, WALKABLE.maxZ) },
    { x: toX, z: toZ },
  ];
}

function randomBoundaryPoint(exclude) {
  // Pick a random point on one of the four boundary edges, excluding the edge we came from.
  const edges = ['north', 'south', 'east', 'west'].filter(e => e !== exclude);
  const edge = edges[Math.floor(Math.random() * edges.length)];
  let x, z;
  switch (edge) {
    case 'north': x = WALKABLE.minX + Math.random() * (WALKABLE.maxX - WALKABLE.minX); z = WALKABLE.maxZ; break;
    case 'south': x = WALKABLE.minX + Math.random() * (WALKABLE.maxX - WALKABLE.minX); z = WALKABLE.minZ; break;
    case 'east':  x = WALKABLE.maxX; z = WALKABLE.minZ + Math.random() * (WALKABLE.maxZ - WALKABLE.minZ); break;
    case 'west':  x = WALKABLE.minX; z = WALKABLE.minZ + Math.random() * (WALKABLE.maxZ - WALKABLE.minZ); break;
  }
  return { x, z, edge };
}

export class Locomotion {
  constructor() {
    this._walkPhase = 0;
    this._waypoints = [];      // Array of {x, z}; last element is final target
    this._arrived = false;
    this._active = false;

    // Pacing state
    this._pacingDwell = 0;
    this._pacingLastEdge = null;
    this._pacingWaiting = false;
  }

  get walkPhase() {
    return this._walkPhase;
  }

  get arrived() {
    return this._arrived;
  }

  get active() {
    return this._active;
  }

  setTarget(x, z) {
    // Called by the state machine with a world-space (x, z) destination.
    // Recomputes waypoints from the current character position stored later in update().
    this._pendingTarget = { x, z };
    this._arrived = false;
    this._active = true;
  }

  stop() {
    this._active = false;
    this._waypoints = [];
    this._pendingTarget = null;
  }

  // Called by behavior state machine to begin pacing mode.
  startPacing(fromX, fromZ) {
    const pt = randomBoundaryPoint(this._pacingLastEdge);
    this._pacingLastEdge = pt.edge;
    this._pacingWaiting = false;
    this._pacingDwell = 0;
    this.setTarget(pt.x, pt.z);
    this._waypointsFrom = { x: fromX, z: fromZ };
    this._waypoints = computeWaypoints(fromX, fromZ, pt.x, pt.z);
    this._pendingTarget = null;
  }

  update(store, dt) {
    const pos = store.character.position;
    const intensity = store.behavior.intensity ?? 0;

    // Resolve a pending target now that we know the current position.
    if (this._pendingTarget) {
      this._waypoints = computeWaypoints(pos.x, pos.z, this._pendingTarget.x, this._pendingTarget.z);
      this._pendingTarget = null;
    }

    if (!this._active || this._waypoints.length === 0) return;

    const target = this._waypoints[0];
    const dx = target.x - pos.x;
    const dz = target.z - pos.z;
    const d = Math.sqrt(dx * dx + dz * dz);

    const isFinalWaypoint = this._waypoints.length === 1;
    const arrivalThreshold = isFinalWaypoint ? ARRIVAL_DIST : 0.1;

    if (d < arrivalThreshold) {
      // Snap to waypoint and advance
      pos.x = target.x;
      pos.z = target.z;
      this._waypoints.shift();

      if (this._waypoints.length === 0) {
        this._arrived = true;
        this._active = false;
      }
      return;
    }

    // Move toward waypoint
    const speed = 0.8 + 0.6 * intensity;
    const step = Math.min(speed * dt, d);
    pos.x += (dx / d) * step;
    pos.z += (dz / d) * step;

    // Smooth facing rotation
    const targetAngle = Math.atan2(dx, dz);
    let facingAngle = store.character.facingAngle ?? 0;
    // Shortest-path lerp
    let diff = targetAngle - facingAngle;
    while (diff >  Math.PI) diff -= 2 * Math.PI;
    while (diff < -Math.PI) diff += 2 * Math.PI;
    facingAngle += diff * Math.min(FACING_RATE * dt, 1.0);
    store.character.facingAngle = facingAngle;

    // Increment walk phase
    const freq = 1.5 + intensity * 1.0;
    this._walkPhase = (this._walkPhase + freq * 2 * Math.PI * dt) % (2 * Math.PI);

    this._arrived = false;
  }

  // Tick pacing dwell timer. Call from state machine when behavior is 'pacing' and locomotion is idle.
  tickPacingDwell(store, dt) {
    if (!this._pacingWaiting) return;
    this._pacingDwell -= dt;
    if (this._pacingDwell <= 0) {
      this._pacingWaiting = false;
      const pos = store.character.position;
      this.startPacing(pos.x, pos.z);
    }
  }

  onPacingArrived(store) {
    this._pacingWaiting = true;
    this._pacingDwell = 1.0 + Math.random() * 2.0;
  }
}
