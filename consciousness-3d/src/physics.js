import * as THREE from 'three';
import * as CANNON from 'cannon-es';

// ── Physics world ────────────────────────────────────────────────────────────

const GRAVITY = -9.82;
const BREAK_IMPULSE = 3.0;    // impulse threshold to shatter an object
const FRAGMENT_COUNT = 5;      // pieces per break
const FRAGMENT_LIFETIME = 4.0; // seconds before fragments fade

export class PhysicsWorld {
  constructor(scene, onBreak) {
    this.scene = scene;
    this.onBreak = onBreak; // callback(position) when something breaks
    this.world = new CANNON.World({ gravity: new CANNON.Vec3(0, GRAVITY, 0) });
    this.world.broadphase = new CANNON.NaiveBroadphase();
    this.world.solver.iterations = 5;

    // Floor
    const floorBody = new CANNON.Body({ mass: 0 });
    floorBody.addShape(new CANNON.Plane());
    floorBody.quaternion.setFromEuler(-Math.PI / 2, 0, 0);
    this.world.addBody(floorBody);

    // Walls
    this._addWall(0, 1.75, -3, 0, 0, 0);      // back
    this._addWall(-4, 1.75, 0, 0, Math.PI/2, 0); // left
    this._addWall(4, 1.75, 0, 0, -Math.PI/2, 0); // right

    this.objects = [];    // { mesh, body, breakable, broken, hp }
    this.fragments = [];  // { mesh, body, age }
    this.characterBody = null;

    this._initCharacterBody();
  }

  _addWall(x, y, z, rx, ry, rz) {
    const body = new CANNON.Body({ mass: 0 });
    body.addShape(new CANNON.Plane());
    body.position.set(x, y, z);
    body.quaternion.setFromEuler(rx, ry, rz);
    this.world.addBody(body);
  }

  _initCharacterBody() {
    this.characterBody = new CANNON.Body({
      mass: 0, // kinematic
      type: CANNON.Body.KINEMATIC,
      shape: new CANNON.Sphere(0.3),
    });
    this.characterBody.position.set(0, 0.5, 0);
    this.world.addBody(this.characterBody);
  }

  // ── Add objects to the room ──────────────────────────────────────────────

  addBreakableBox(x, y, z, w, h, d, color, name) {
    const geo = new THREE.BoxGeometry(w, h, d);
    const mat = new THREE.MeshStandardMaterial({ color, roughness: 0.7, metalness: 0.1 });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    mesh.position.set(x, y, z);
    this.scene.add(mesh);

    const body = new CANNON.Body({
      mass: 1,
      position: new CANNON.Vec3(x, y, z),
      linearDamping: 0.4,
      angularDamping: 0.4,
    });
    body.addShape(new CANNON.Box(new CANNON.Vec3(w/2, h/2, d/2)));
    this.world.addBody(body);

    const obj = { mesh, body, breakable: true, broken: false, name, hp: 1, w, h, d, color };
    this.objects.push(obj);

    // Listen for collisions
    body.addEventListener('collide', (e) => {
      if (obj.broken) return;
      const impulse = e.contact ? Math.abs(e.contact.getImpactVelocityAlongNormal()) : 0;
      if (impulse > BREAK_IMPULSE || e.body === this.characterBody) {
        this._breakObject(obj);
      }
    });

    return obj;
  }

  addStaticBox(x, y, z, w, h, d, color) {
    const geo = new THREE.BoxGeometry(w, h, d);
    const mat = new THREE.MeshStandardMaterial({ color, roughness: 0.7, metalness: 0.1 });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.castShadow = true;
    mesh.position.set(x, y, z);
    this.scene.add(mesh);

    const body = new CANNON.Body({
      mass: 0.5,
      position: new CANNON.Vec3(x, y, z),
      linearDamping: 0.5,
      angularDamping: 0.5,
    });
    body.addShape(new CANNON.Box(new CANNON.Vec3(w/2, h/2, d/2)));
    this.world.addBody(body);

    this.objects.push({ mesh, body, breakable: false, broken: false, name: 'box' });
  }

  // ── Populate the room ──────────────────────────────────────────────────

  populateRoom() {
    // Tall floor lamp (near right wall)
    this.addBreakableBox(3.2, 0.6, -1.0, 0.08, 1.2, 0.08, 0x998866, 'floor lamp');
    // Lamp shade on top
    this.addBreakableBox(3.2, 1.25, -1.0, 0.25, 0.15, 0.25, 0xddcc99, 'lamp shade');

    // Vase on table
    this.addBreakableBox(-2.0, 0.85, -2.0, 0.12, 0.2, 0.12, 0x6688aa, 'vase');

    // Books stacked on table
    this.addBreakableBox(-1.7, 0.82, -1.85, 0.18, 0.08, 0.12, 0x884433, 'book');
    this.addBreakableBox(-1.7, 0.90, -1.85, 0.18, 0.08, 0.12, 0x336644, 'book');
    this.addBreakableBox(-1.7, 0.98, -1.85, 0.17, 0.08, 0.11, 0x443366, 'book');

    // Mug on table
    this.addBreakableBox(-2.3, 0.82, -2.1, 0.08, 0.1, 0.08, 0xeeeeee, 'mug');

    // Picture frame leaning against back wall
    this.addBreakableBox(-1.0, 0.25, -2.85, 0.35, 0.5, 0.03, 0x554433, 'picture frame');

    // Small side table near left wall
    this.addBreakableBox(-3.3, 0.3, 0.5, 0.4, 0.6, 0.4, 0x665544, 'side table');

    // Bottle on side table
    this.addBreakableBox(-3.3, 0.65, 0.5, 0.06, 0.2, 0.06, 0x225533, 'bottle');

    // Potted plant near right wall
    this.addBreakableBox(3.5, 0.2, 1.0, 0.2, 0.4, 0.2, 0x775533, 'plant pot');
    // Plant top
    this.addBreakableBox(3.5, 0.5, 1.0, 0.3, 0.2, 0.3, 0x336633, 'plant');

    // Stack of boxes in corner
    this.addBreakableBox(-3.2, 0.15, -2.2, 0.3, 0.3, 0.3, 0x887766, 'box');
    this.addBreakableBox(-3.2, 0.45, -2.2, 0.28, 0.28, 0.28, 0x776655, 'box');
    this.addBreakableBox(-3.15, 0.73, -2.25, 0.25, 0.25, 0.25, 0x665544, 'small box');

    // Candle holder near center
    this.addBreakableBox(0.5, 0.15, 1.0, 0.06, 0.3, 0.06, 0xaa8844, 'candle');

    // Shoe rack / small shelf near door area
    this.addBreakableBox(1.5, 0.2, 2.0, 0.5, 0.4, 0.2, 0x554433, 'shoe rack');

    // Glasses on shoe rack
    this.addBreakableBox(1.4, 0.45, 2.0, 0.1, 0.06, 0.06, 0xaaaacc, 'glasses');
  }

  // ── Break an object into fragments ────────────────────────────────────

  _breakObject(obj) {
    if (obj.broken) return;
    obj.broken = true;

    // Remove original mesh and body
    this.scene.remove(obj.mesh);
    obj.mesh.geometry.dispose();
    obj.mesh.material.dispose();
    this.world.removeBody(obj.body);

    // Spawn fragments
    const pos = obj.body.position;
    const fragSize = Math.max(obj.w || 0.1, obj.h || 0.1, obj.d || 0.1) * 0.3;

    for (let i = 0; i < FRAGMENT_COUNT; i++) {
      const fw = fragSize * (0.3 + Math.random() * 0.7);
      const fh = fragSize * (0.3 + Math.random() * 0.7);
      const fd = fragSize * (0.3 + Math.random() * 0.7);

      const geo = new THREE.BoxGeometry(fw, fh, fd);
      const mat = new THREE.MeshStandardMaterial({
        color: obj.color || 0x888888,
        roughness: 0.8,
        transparent: true,
        opacity: 1.0,
      });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.castShadow = true;
      this.scene.add(mesh);

      const body = new CANNON.Body({
        mass: 0.1,
        position: new CANNON.Vec3(
          pos.x + (Math.random() - 0.5) * 0.2,
          pos.y + (Math.random() - 0.5) * 0.2,
          pos.z + (Math.random() - 0.5) * 0.2,
        ),
        linearDamping: 0.3,
        angularDamping: 0.3,
      });
      body.addShape(new CANNON.Box(new CANNON.Vec3(fw/2, fh/2, fd/2)));

      // Explosion impulse
      body.velocity.set(
        (Math.random() - 0.5) * 4,
        2 + Math.random() * 3,
        (Math.random() - 0.5) * 4,
      );
      body.angularVelocity.set(
        (Math.random() - 0.5) * 10,
        (Math.random() - 0.5) * 10,
        (Math.random() - 0.5) * 10,
      );

      this.world.addBody(body);
      this.fragments.push({ mesh, body, age: 0, geo, mat });
    }

    // Callback — triggers fear in the consciousness
    if (this.onBreak) {
      this.onBreak({ x: pos.x, y: pos.y, z: pos.z, name: obj.name });
    }
  }

  // ── Update each frame ──────────────────────────────────────────────────

  update(dt, characterX, characterZ) {
    // Move kinematic character body
    this.characterBody.position.set(characterX, 0.5, characterZ);

    // Step physics
    this.world.step(1/60, dt, 3);

    // Sync meshes to physics bodies
    for (const obj of this.objects) {
      if (obj.broken) continue;
      obj.mesh.position.copy(obj.body.position);
      obj.mesh.quaternion.copy(obj.body.quaternion);
    }

    // Sync and age fragments
    for (let i = this.fragments.length - 1; i >= 0; i--) {
      const frag = this.fragments[i];
      frag.age += dt;
      frag.mesh.position.copy(frag.body.position);
      frag.mesh.quaternion.copy(frag.body.quaternion);

      // Fade out
      if (frag.age > FRAGMENT_LIFETIME * 0.6) {
        const fade = 1.0 - (frag.age - FRAGMENT_LIFETIME * 0.6) / (FRAGMENT_LIFETIME * 0.4);
        frag.mat.opacity = Math.max(0, fade);
      }

      // Remove expired
      if (frag.age > FRAGMENT_LIFETIME) {
        this.scene.remove(frag.mesh);
        frag.geo.dispose();
        frag.mat.dispose();
        this.world.removeBody(frag.body);
        this.fragments.splice(i, 1);
      }
    }
  }
}
