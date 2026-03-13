import * as THREE from 'three';

export const CHAIR_POS = { x: 2.5, y: 0, z: -1.5 };

export function createRoom(scene) {
  const floorMat = new THREE.MeshStandardMaterial({ color: 0x6b5e52, roughness: 0.85, metalness: 0.05 });
  const wallMat  = new THREE.MeshStandardMaterial({ color: 0x8a8078, roughness: 0.9,  metalness: 0.0  });
  const ceilMat  = new THREE.MeshStandardMaterial({ color: 0x5a5550, roughness: 0.95, metalness: 0.0  });
  const furnMat  = new THREE.MeshStandardMaterial({ color: 0x7a6b5a, roughness: 0.8,  metalness: 0.05 });

  // ── Floor ────────────────────────────────────────────────────────────────
  const floorGeo = new THREE.PlaneGeometry(8, 6);
  const floor = new THREE.Mesh(floorGeo, floorMat);
  floor.rotation.x = -Math.PI / 2;
  floor.position.set(0, 0, 0);
  floor.receiveShadow = true;
  scene.add(floor);

  // ── Walls ────────────────────────────────────────────────────────────────
  const walls = new THREE.Group();

  // Back wall  Z = -3
  const backGeo = new THREE.PlaneGeometry(8, 3.5);
  const backWall = new THREE.Mesh(backGeo, wallMat);
  backWall.position.set(0, 1.75, -3);
  backWall.receiveShadow = true;
  walls.add(backWall);

  // Left wall  X = -4  (rotated +90° on Y so normal faces +X)
  const leftGeo = new THREE.PlaneGeometry(6, 3.5);
  const leftWall = new THREE.Mesh(leftGeo, wallMat);
  leftWall.rotation.y = Math.PI / 2;
  leftWall.position.set(-4, 1.75, 0);
  leftWall.receiveShadow = true;
  walls.add(leftWall);

  // Right wall  X = +4  (rotated -90° on Y so normal faces -X)
  const rightGeo = new THREE.PlaneGeometry(6, 3.5);
  const rightWall = new THREE.Mesh(rightGeo, wallMat);
  rightWall.rotation.y = -Math.PI / 2;
  rightWall.position.set(4, 1.75, 0);
  rightWall.receiveShadow = true;
  walls.add(rightWall);

  scene.add(walls);

  // ── Ceiling ───────────────────────────────────────────────────────────────
  const ceilGeo = new THREE.PlaneGeometry(8, 6);
  const ceiling = new THREE.Mesh(ceilGeo, ceilMat);
  ceiling.rotation.x = Math.PI / 2;
  ceiling.position.set(0, 3.5, 0);
  ceiling.receiveShadow = true;
  scene.add(ceiling);

  // ── Chair ─────────────────────────────────────────────────────────────────
  // Origin at (2.5, 0, -1.5); seat centre at Y=0.45
  const chair = new THREE.Group();
  chair.position.set(CHAIR_POS.x, CHAIR_POS.y, CHAIR_POS.z);

  const seatMesh = new THREE.Mesh(new THREE.BoxGeometry(0.45, 0.05, 0.45), furnMat);
  seatMesh.position.set(0, 0.45, 0);
  seatMesh.receiveShadow = true;
  seatMesh.castShadow = true;
  chair.add(seatMesh);

  // Backrest: sits on top of seat back edge, centred vertically at seat+0.05/2+0.55/2
  const backrestMesh = new THREE.Mesh(new THREE.BoxGeometry(0.45, 0.55, 0.05), furnMat);
  backrestMesh.position.set(0, 0.45 + 0.025 + 0.275, -0.20);
  backrestMesh.receiveShadow = true;
  backrestMesh.castShadow = true;
  chair.add(backrestMesh);

  // Four legs  (seat underside at Y=0.45-0.025=0.425; legs go from 0→0.425)
  const legH = 0.425;
  const legGeo = new THREE.BoxGeometry(0.04, legH, 0.04);
  const legOffsets = [
    [ 0.185,  0.185],
    [-0.185,  0.185],
    [ 0.185, -0.185],
    [-0.185, -0.185],
  ];
  for (const [lx, lz] of legOffsets) {
    const leg = new THREE.Mesh(legGeo, furnMat);
    leg.position.set(lx, legH / 2, lz);
    leg.receiveShadow = true;
    leg.castShadow = true;
    chair.add(leg);
  }

  scene.add(chair);

  // ── Table ─────────────────────────────────────────────────────────────────
  // Design spec: position (-2.0, 0, -2.0), top 1.0×0.04×0.6 at Y=0.75
  const table = new THREE.Group();
  table.position.set(-2.0, 0, -2.0);

  const topMesh = new THREE.Mesh(new THREE.BoxGeometry(1.0, 0.04, 0.6), furnMat);
  topMesh.position.set(0, 0.75, 0);
  topMesh.receiveShadow = true;
  topMesh.castShadow = true;
  table.add(topMesh);

  // Four legs from floor to underside of top (Y=0 → Y=0.73)
  const tLegH = 0.73;
  const tLegGeo = new THREE.BoxGeometry(0.05, tLegH, 0.05);
  const tLegOffsets = [
    [ 0.45,  0.25],
    [-0.45,  0.25],
    [ 0.45, -0.25],
    [-0.45, -0.25],
  ];
  for (const [lx, lz] of tLegOffsets) {
    const leg = new THREE.Mesh(tLegGeo, furnMat);
    leg.position.set(lx, tLegH / 2, lz);
    leg.receiveShadow = true;
    leg.castShadow = true;
    table.add(leg);
  }

  scene.add(table);

  return { floor, walls, ceiling, chair, table };
}
