import * as THREE from 'three';

export function createParticleBurst(options = {}) {
  const {
    count = 20,
    color = 0xffffff,
    size = 0.02,
    position = { x: 0, y: 0, z: 0 },
    velocity = { x: 0, y: 1, z: 0 },
    gravity = -1.5,
    lifetime = 2.0,
    spread = 1.0,
  } = options;

  const group = new THREE.Group();
  group.position.set(position.x, position.y, position.z);

  const particles = [];

  for (let i = 0; i < count; i++) {
    const geo = new THREE.SphereGeometry(size, 4, 4);
    const mat = new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 1.0 });
    const mesh = new THREE.Mesh(geo, mat);

    const vx = velocity.x + (Math.random() - 0.5) * spread;
    const vy = velocity.y + (Math.random() - 0.5) * spread;
    const vz = velocity.z + (Math.random() - 0.5) * spread;

    mesh.position.set(
      (Math.random() - 0.5) * spread * 0.25,
      (Math.random() - 0.5) * spread * 0.25,
      (Math.random() - 0.5) * spread * 0.25
    );

    particles.push({ mesh, vx, vy, vz, age: 0, alive: true });
    group.add(mesh);
  }

  let doneCount = 0;

  function update(dt) {
    for (const p of particles) {
      if (!p.alive) continue;

      p.age += dt;
      p.vy += gravity * dt;
      p.mesh.position.x += p.vx * dt;
      p.mesh.position.y += p.vy * dt;
      p.mesh.position.z += p.vz * dt;

      const t = p.age / lifetime;
      p.mesh.material.opacity = Math.max(0, 1.0 - t);

      if (p.mesh.material.opacity <= 0) {
        p.alive = false;
        group.remove(p.mesh);
        p.mesh.geometry.dispose();
        p.mesh.material.dispose();
        doneCount++;
      }
    }
  }

  return {
    group,
    update,
    get isDone() {
      return doneCount >= count;
    },
  };
}
