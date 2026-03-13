import * as THREE from 'three';

export const CLOTHING_MAT = new THREE.MeshStandardMaterial({
  color: 0x4a5568,
  roughness: 0.75,
  metalness: 0.05,
});

export const SKIN_MAT = new THREE.MeshStandardMaterial({
  color: 0xc8a882,
  roughness: 0.6,
  metalness: 0.0,
});

export const SHOE_MAT = new THREE.MeshStandardMaterial({
  color: 0x2d2d2d,
  roughness: 0.9,
  metalness: 0.1,
});

const CATEGORY_COLORS = {
  cognitive:   new THREE.Color(0x4A90D9),
  emotional:   new THREE.Color(0xE8613C),
  social:      new THREE.Color(0x3CB371),
  executive:   new THREE.Color(0x9B59B6),
  existential: new THREE.Color(0xDAA520),
};

export function updateDominantCategory(category, dominance) {
  const color = CATEGORY_COLORS[category];
  if (!color) return;
  CLOTHING_MAT.emissive.copy(color);
  CLOTHING_MAT.emissiveIntensity = 0.05 * dominance;
}
