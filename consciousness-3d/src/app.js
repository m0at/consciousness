import * as THREE from 'three';
import { createRoom, CHAIR_POS } from './scene/room.js';
import { createLights } from './scene/lighting.js';
import { loadCharacter, updateCharacter, playAnimation, setCharacterPosition, setCharacterRotation, getCharacter } from './character/model.js';
import { BEHAVIORS } from './behavior/behaviors.js';
import { Locomotion } from './animation/locomotion.js';
import { StateMachine } from './behavior/stateMachine.js';
import { EventManager } from './environment/events.js';
import { Store } from './state/store.js';
import { interpolate } from './state/interpolator.js';
import { HUD } from './hud/hud.js';
import { sendInput } from './ipc.js';

// ── Renderer
const canvas = document.getElementById('viewport');
const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
renderer.setSize(window.innerWidth, window.innerHeight);
renderer.toneMapping = THREE.ACESFilmicToneMapping;
renderer.toneMappingExposure = 1.0;
renderer.shadowMap.enabled = true;
renderer.shadowMap.type = THREE.PCFSoftShadowMap;

// ── Camera
const camera = new THREE.PerspectiveCamera(50, window.innerWidth / window.innerHeight, 0.1, 50);
camera.position.set(0, 2.8, 7.5);
camera.lookAt(0, 1.0, 0);

// ── Scene
const scene = new THREE.Scene();
const room = createRoom(scene);
const lights = createLights(scene);
const store = new Store();
const locomotion = new Locomotion();
const stateMachine = new StateMachine(locomotion);
const eventManager = new EventManager();
const hud = new HUD();

// Initialize event system
eventManager.init(scene);

// ── IPC
window.api.onPythonMessage(msg => store.handleMessage(msg));

// ── Keyboard
window.addEventListener('keydown', e => {
    if (e.key === 'ArrowUp') sendInput('positive');
    else if (e.key === 'ArrowDown') sendInput('negative');
    else if (e.key === 'Tab') { e.preventDefault(); hud.toggle(); }
});
window.addEventListener('keyup', e => {
    if (e.key === 'ArrowUp' || e.key === 'ArrowDown') sendInput('none');
});

// ── Resize
window.addEventListener('resize', () => {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
});

let currentAnimName = 'idle';
let characterLoaded = false;

function updateBehaviorAnimation() {
    if (!characterLoaded) return;
    const behavior = store.behavior.current;
    const meta = BEHAVIORS[behavior] ?? BEHAVIORS['idle_calm'];
    const isWalking = stateMachine.state === 'WALKING' && locomotion.active;

    // Pick animation: walking behaviors get walk/run, others get idle
    let animName;
    if (isWalking) {
        // Fast/anxious behaviors use run, others use walk
        animName = (behavior === 'wander_anxious' || behavior === 'pacing_energized') ? 'run' : 'walk';
    } else {
        animName = 'idle';
    }

    if (animName !== currentAnimName) {
        playAnimation(animName, 0.4);
        currentAnimName = animName;
    }

    // Adjust animation speed based on behavior
    const ch = getCharacter();
    if (ch && ch.mixer) {
        ch.mixer.timeScale = meta.animSpeed;
    }
}

// ── Load character async, then start render loop
async function init() {
    try {
        await loadCharacter(scene);
        characterLoaded = true;
        console.log('Character loaded');
    } catch (err) {
        console.error('Failed to load character:', err);
        // Continue without character - scene still renders
    }

    let lastTime = performance.now();

    function animate(time) {
        requestAnimationFrame(animate);
        const dt = Math.min((time - lastTime) / 1000, 0.1);
        lastTime = time;

        // Update state
        interpolate(store, time);
        stateMachine.update(store, dt);
        store.character.walkPhase = locomotion.walkPhase;

        // Update animation based on state machine + locomotion
        updateBehaviorAnimation();

        // Update character animation mixer
        updateCharacter(dt);

        // Sync character position from locomotion
        setCharacterPosition(store.character.position.x, store.character.position.z);
        setCharacterRotation(store.character.facingAngle);

        // Update systems
        lights.update(store);
        eventManager.update(scene, store, dt);
        hud.update(store);

        renderer.render(scene, camera);
    }

    requestAnimationFrame(animate);
}

init();
