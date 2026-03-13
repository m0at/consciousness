import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';

let character = null;
let mixer = null;
let actions = {};

export async function loadCharacter(scene) {
    const loader = new GLTFLoader();
    const gltf = await loader.loadAsync('./assets/Soldier.glb');

    const model = gltf.scene;
    model.scale.set(1, 1, 1);
    model.position.set(0, 0, 0);
    model.traverse(child => {
        if (child.isMesh) {
            child.castShadow = true;
            child.receiveShadow = true;
        }
    });
    scene.add(model);

    mixer = new THREE.AnimationMixer(model);

    for (const clip of gltf.animations) {
        const action = mixer.clipAction(clip);
        actions[clip.name.toLowerCase()] = action;
        console.log('Animation:', clip.name, 'duration:', clip.duration);
    }

    if (actions.idle) {
        actions.idle.play();
    }

    character = { model, mixer, actions };
    return character;
}

export function updateCharacter(dt) {
    if (mixer) mixer.update(dt);
}

export function playAnimation(name, fadeDuration = 0.5) {
    if (!character) return;
    const target = actions[name];
    if (!target) return;

    const current = Object.values(actions).find(a => a.isRunning() && a.getEffectiveWeight() > 0);
    if (current === target) return;

    target.reset();
    target.setEffectiveTimeScale(1);
    target.setEffectiveWeight(1);

    if (current) {
        current.crossFadeTo(target, fadeDuration, true);
    }
    target.play();
}

export function setCharacterPosition(x, z) {
    if (character) {
        character.model.position.x = x;
        character.model.position.z = z;
    }
}

export function setCharacterRotation(y) {
    if (character) {
        character.model.rotation.y = y;
    }
}

export function getCharacter() { return character; }
