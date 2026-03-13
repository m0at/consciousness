import * as THREE from 'three';
import { createRoom, CHAIR_POS } from './scene/room.js';
import { createLights } from './scene/lighting.js';
import { loadCharacter, updateCharacter, playAnimation, setCharacterPosition, setCharacterRotation, getCharacter } from './character/model.js';
import { BEHAVIORS } from './behavior/behaviors.js';
import { Locomotion } from './animation/locomotion.js';
import { StateMachine } from './behavior/stateMachine.js';
import { EventManager } from './environment/events.js';
import { PhysicsWorld } from './physics.js';
import { Store } from './state/store.js';
import { interpolate } from './state/interpolator.js';
import { HUD } from './hud/hud.js';
import { sendInput } from './ipc.js';
import { InteractionSystem } from './interactions.js';

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

// ── Physics with breakable objects
// When something breaks, inject fear/stress into the consciousness
let breakCooldown = 0;
const physics = new PhysicsWorld(scene, (event) => {
    // Something broke! Scare the character.
    console.log('BREAK:', event.name, 'at', event.x.toFixed(1), event.z.toFixed(1));

    // Inject negative stimulus (fear/threat) — multiple rapid injections for big scare
    if (breakCooldown <= 0) {
        for (let i = 0; i < 5; i++) {
            window.api.sendToPython({ type: 'input', direction: 'negative' });
        }
        breakCooldown = 0.5; // don't stack too many break-scares
    }

    // Flash the room red briefly via a temporary light
    const flash = new THREE.PointLight(0xff2200, 2, 6);
    flash.position.set(event.x, 1.5, event.z);
    scene.add(flash);
    setTimeout(() => {
        scene.remove(flash);
        flash.dispose();
    }, 300);
});
physics.populateRoom();

// Initialize event system
eventManager.init(scene);

// ── Interaction system (ghost tackles + drugs)
const interactions = new InteractionSystem(scene);

// ── IPC
window.api.onPythonMessage(msg => store.handleMessage(msg));

// ── Keyboard
window.addEventListener('keydown', e => {
    if (e.key === 'ArrowUp') sendInput('positive');
    else if (e.key === 'ArrowDown') sendInput('negative');
    else if (e.key === 'Tab') { e.preventDefault(); hud.toggle(); }
    else if (e.key === 'r' || e.key === 'R') {
        window.api.sendToPython({ type: 'randomize' });
    }
    // WASD — ghost tackle from different directions
    else if ('wasd'.includes(e.key)) {
        const result = interactions.ghostTackle(e.key, store);
        if (result) {
            window.api.sendToPython({ type: 'nudge', deltas: result.deltas });
            // Also inject negative for the fear response
            for (let i = 0; i < 3; i++) window.api.sendToPython({ type: 'input', direction: 'negative' });
            interactions.showNotification(result.label, '#8844cc');
        }
    }
    // IJKL — drugs/medicine
    else if ('ijkl'.includes(e.key)) {
        const result = interactions.administerDrug(e.key, store);
        if (result) {
            window.api.sendToPython({ type: 'nudge', deltas: result.deltas });
            interactions.showNotification(result.label, '#44ff88');
        }
    }
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

    let animName;
    if (isWalking) {
        animName = (behavior === 'wander_anxious' || behavior === 'pacing_energized') ? 'run' : 'walk';
    } else {
        animName = 'idle';
    }

    if (animName !== currentAnimName) {
        playAnimation(animName, 0.4);
        currentAnimName = animName;
    }

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
    }

    let lastTime = performance.now();

    function animate(time) {
        requestAnimationFrame(animate);
        const dt = Math.min((time - lastTime) / 1000, 0.1);
        lastTime = time;

        // Cooldown for break-scare
        if (breakCooldown > 0) breakCooldown -= dt;

        // Update state
        interpolate(store, time);
        stateMachine.update(store, dt);
        store.character.walkPhase = locomotion.walkPhase;

        // Update animation
        updateBehaviorAnimation();
        updateCharacter(dt);

        // Sync character position
        const charX = store.character.position.x;
        const charZ = store.character.position.z;
        setCharacterPosition(charX, charZ);
        setCharacterRotation(store.character.facingAngle);

        // Step physics — character collision sphere follows the character
        physics.update(dt, charX, charZ);

        // Update systems
        lights.update(store);
        eventManager.update(scene, store, dt);
        interactions.update(dt);
        hud.update(store);

        // Show interaction notifications in mini panel
        const notif = interactions.getActiveNotification();
        if (notif) {
            store._lastNotification = notif;
        }

        renderer.render(scene, camera);
    }

    requestAnimationFrame(animate);
}

init();
