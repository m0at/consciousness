import * as THREE from 'three';

// ── Ghost tackle directions ──────────────────────────────────────────────────
const TACKLE_DIRS = {
  w: { x: 0, z: -1, label: 'NORTH' },  // push from front (toward back wall)
  s: { x: 0, z: 1, label: 'SOUTH' },   // push from back
  a: { x: -1, z: 0, label: 'WEST' },   // push from right (toward left wall)
  d: { x: 1, z: 0, label: 'EAST' },    // push from left
};

// How hard the ghost shoves the character's position
const TACKLE_FORCE = 1.2;
// How much the tackle damages consciousness weights
const TACKLE_DAMAGE = {
  'self-esteem':     -0.3,
  'self-efficacy':   -0.2,
  'agency':          -0.2,
  'self-regulation': -0.15,
  'body_awareness':   0.4,   // heightened body awareness from impact
  'emotional_awareness': 0.3,
  'situational_awareness': 0.3,
};

// ── Drug types ───────────────────────────────────────────────────────────────
const DRUGS = {
  i: {
    name: 'Stimulant',
    color: 0x44ff44,
    desc: 'energy + motivation + agency',
    deltas: {
      'motivation': 0.4, 'agency': 0.35, 'self-efficacy': 0.3,
      'goal-setting': 0.2, 'body_awareness': 0.15,
      'self-regulation': -0.15,  // overstimulated, harder to regulate
    },
  },
  j: {
    name: 'Sedative',
    color: 0x4488ff,
    desc: 'calm + regulation + reduce stress',
    deltas: {
      'self-regulation': 0.4, 'emotional_awareness': -0.2,
      'motivation': -0.2, 'agency': -0.15,
      'introspection': 0.2, 'reflection': 0.15,
      'body_awareness': -0.1,
    },
  },
  k: {
    name: 'Empathogen',
    color: 0xff44ff,
    desc: 'social + emotional + empathy',
    deltas: {
      'social_awareness': 0.4, 'theory_of_mind': 0.35,
      'emotional_awareness': 0.35, 'moral_awareness': 0.25,
      'self-esteem': 0.2, 'introspection': 0.2,
      'self-monitoring': -0.1,
    },
  },
  l: {
    name: 'Nootropic',
    color: 0xffaa00,
    desc: 'cognition + metacognition + clarity',
    deltas: {
      'metacognition': 0.4, 'introspection': 0.3,
      'reflection': 0.3, 'self-monitoring': 0.25,
      'temporal_awareness': 0.2, 'situational_awareness': 0.2,
      'self-development': 0.15,
      'emotional_awareness': -0.1,  // slightly numbed emotions
    },
  },
};

export class InteractionSystem {
  constructor(scene) {
    this.scene = scene;
    this._ghosts = [];     // active ghost visual effects
    this._drugFx = [];     // active drug visual effects
    this._notifications = []; // floating text notifications
  }

  // ── Ghost tackle ─────────────────────────────────────────────────────────

  ghostTackle(key, store) {
    const dir = TACKLE_DIRS[key];
    if (!dir) return null;

    const pos = store.character.position;

    // Shove the character position
    pos.x += dir.x * TACKLE_FORCE;
    pos.z += dir.z * TACKLE_FORCE;
    // Clamp to room
    pos.x = Math.max(-3.5, Math.min(3.5, pos.x));
    pos.z = Math.max(-2.5, Math.min(2.5, pos.z));

    // Spawn ghost visual — a dark translucent figure rushing in from the direction
    const ghostStart = {
      x: pos.x - dir.x * 3,
      z: pos.z - dir.z * 3,
    };
    this._spawnGhost(ghostStart, pos, dir);

    // Spawn impact flash
    this._spawnImpactFlash(pos);

    // Return the weight deltas to apply
    return { deltas: TACKLE_DAMAGE, label: 'GHOST TACKLE ' + dir.label };
  }

  _spawnGhost(from, to, dir) {
    // Dark translucent rushing shape
    const geo = new THREE.ConeGeometry(0.3, 1.8, 6);
    const mat = new THREE.MeshBasicMaterial({
      color: 0x220033,
      transparent: true,
      opacity: 0.6,
      side: THREE.DoubleSide,
    });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.position.set(from.x, 0.9, from.z);
    // Point toward the character
    mesh.lookAt(to.x, 0.9, to.z);
    mesh.rotateX(Math.PI / 2);
    this.scene.add(mesh);

    // Trail particles
    const trail = new THREE.Group();
    for (let i = 0; i < 8; i++) {
      const pg = new THREE.SphereGeometry(0.06 + Math.random() * 0.06, 4, 4);
      const pm = new THREE.MeshBasicMaterial({
        color: 0x440066,
        transparent: true,
        opacity: 0.5,
      });
      const p = new THREE.Mesh(pg, pm);
      p.position.set(
        (Math.random() - 0.5) * 0.4,
        (Math.random() - 0.5) * 0.8 + 0.9,
        (Math.random() - 0.5) * 0.4,
      );
      trail.add(p);
    }
    trail.position.set(from.x, 0, from.z);
    this.scene.add(trail);

    this._ghosts.push({
      mesh, trail, mat,
      startX: from.x, startZ: from.z,
      endX: to.x, endZ: to.z,
      elapsed: 0,
      duration: 0.4,
    });
  }

  _spawnImpactFlash(pos) {
    // White/purple flash ring expanding from impact point
    const ringGeo = new THREE.TorusGeometry(0.1, 0.05, 8, 24);
    const ringMat = new THREE.MeshBasicMaterial({
      color: 0x8844cc,
      transparent: true,
      opacity: 0.8,
    });
    const ring = new THREE.Mesh(ringGeo, ringMat);
    ring.position.set(pos.x, 0.9, pos.z);
    ring.rotation.x = Math.PI / 2;
    this.scene.add(ring);

    // Impact light
    const light = new THREE.PointLight(0x8844cc, 3, 5);
    light.position.set(pos.x, 1.5, pos.z);
    this.scene.add(light);

    this._ghosts.push({
      mesh: ring, trail: null, mat: ringMat,
      light,
      isRing: true,
      elapsed: 0,
      duration: 0.6,
    });
  }

  // ── Drug administration ──────────────────────────────────────────────────

  administerDrug(key, store) {
    const drug = DRUGS[key];
    if (!drug) return null;

    const pos = store.character.position;
    this._spawnDrugEffect(pos, drug.color);

    return { deltas: drug.deltas, label: drug.name.toUpperCase() + ': ' + drug.desc };
  }

  _spawnDrugEffect(pos, color) {
    // Rising healing particles spiraling around character
    const group = new THREE.Group();
    group.position.set(pos.x, 0, pos.z);

    for (let i = 0; i < 20; i++) {
      const angle = (i / 20) * Math.PI * 4;
      const radius = 0.3 + (i / 20) * 0.4;
      const y = (i / 20) * 2.0;
      const pg = new THREE.SphereGeometry(0.025, 4, 4);
      const pm = new THREE.MeshBasicMaterial({
        color,
        transparent: true,
        opacity: 0.8,
      });
      const p = new THREE.Mesh(pg, pm);
      p.position.set(Math.cos(angle) * radius, y, Math.sin(angle) * radius);
      p.userData.baseAngle = angle;
      p.userData.radius = radius;
      p.userData.baseY = y;
      group.add(p);
    }
    this.scene.add(group);

    // Healing glow
    const light = new THREE.PointLight(color, 0, 4);
    light.position.set(pos.x, 1.2, pos.z);
    this.scene.add(light);

    this._drugFx.push({
      group, light, color,
      elapsed: 0,
      duration: 2.0,
    });
  }

  // ── Notification text (floating label) ─────────────────────────────────

  showNotification(label, color) {
    // We'll track these and the HUD can read them
    this._notifications.push({ label, color, elapsed: 0, duration: 2.0 });
  }

  getActiveNotification() {
    if (this._notifications.length === 0) return null;
    return this._notifications[0];
  }

  // ── Update every frame ─────────────────────────────────────────────────

  update(dt) {
    // Ghosts
    for (let i = this._ghosts.length - 1; i >= 0; i--) {
      const g = this._ghosts[i];
      g.elapsed += dt;
      const t = Math.min(g.elapsed / g.duration, 1);

      if (g.isRing) {
        // Expanding ring
        const scale = 1 + t * 8;
        g.mesh.scale.set(scale, scale, scale);
        g.mat.opacity = 0.8 * (1 - t);
        if (g.light) g.light.intensity = 3 * (1 - t);
      } else {
        // Rush toward character
        const ease = t * t; // accelerating
        g.mesh.position.x = g.startX + (g.endX - g.startX) * ease;
        g.mesh.position.z = g.startZ + (g.endZ - g.startZ) * ease;
        g.mat.opacity = 0.6 * (1 - t * 0.5);
        if (g.trail) {
          g.trail.position.x = g.mesh.position.x;
          g.trail.position.z = g.mesh.position.z;
          g.trail.children.forEach(p => {
            p.material.opacity = 0.5 * (1 - t);
          });
        }
      }

      if (g.elapsed >= g.duration) {
        this.scene.remove(g.mesh);
        g.mesh.geometry?.dispose();
        g.mat?.dispose();
        if (g.trail) {
          g.trail.children.forEach(p => { p.geometry?.dispose(); p.material?.dispose(); });
          this.scene.remove(g.trail);
        }
        if (g.light) { this.scene.remove(g.light); g.light.dispose(); }
        this._ghosts.splice(i, 1);
      }
    }

    // Drug effects
    for (let i = this._drugFx.length - 1; i >= 0; i--) {
      const fx = this._drugFx[i];
      fx.elapsed += dt;
      const t = fx.elapsed / fx.duration;

      // Rotate and rise
      fx.group.rotation.y += dt * 2;
      fx.group.children.forEach(p => {
        p.position.y = p.userData.baseY + t * 0.5;
        p.material.opacity = 0.8 * Math.max(0, 1 - t);
      });

      // Glow pulse
      fx.light.intensity = 1.5 * Math.sin(t * Math.PI);

      if (fx.elapsed >= fx.duration) {
        fx.group.children.forEach(p => { p.geometry?.dispose(); p.material?.dispose(); });
        this.scene.remove(fx.group);
        this.scene.remove(fx.light);
        fx.light.dispose();
        this._drugFx.splice(i, 1);
      }
    }

    // Notifications
    for (let i = this._notifications.length - 1; i >= 0; i--) {
      this._notifications[i].elapsed += dt;
      if (this._notifications[i].elapsed >= this._notifications[i].duration) {
        this._notifications.splice(i, 1);
      }
    }
  }
}
