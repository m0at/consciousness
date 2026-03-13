import * as THREE from 'three';

export class EventManager {
    constructor() {
        this._activeEffects = [];
        this._stressLight = null;
        this._rewardParticles = null;
        this._prevStimuli = new Set();
    }

    // Create persistent scene objects (call once after scene is built)
    init(scene) {
        // A subtle colored point light that represents emotional state
        // Positioned near the character
        this._moodLight = new THREE.PointLight(0xffffff, 0, 5);
        this._moodLight.position.set(0, 2, 0);
        scene.add(this._moodLight);

        // A subtle ground glow plane under the character
        const glowGeo = new THREE.CircleGeometry(1.5, 32);
        const glowMat = new THREE.MeshBasicMaterial({
            color: 0x4A90D9, transparent: true, opacity: 0,
            side: THREE.DoubleSide
        });
        this._groundGlow = new THREE.Mesh(glowGeo, glowMat);
        this._groundGlow.rotation.x = -Math.PI / 2;
        this._groundGlow.position.y = 0.01;
        scene.add(this._groundGlow);
    }

    update(scene, store, dt) {
        if (!store.latestTick) return;

        const stimuli = new Set(store.latestTick.activeStimuli || []);
        const valence = store.interpolated?.valence ?? 0;
        const stress = store.interpolated?.stress ?? 0;
        const energy = store.interpolated?.energy?.energy_pct ?? 100;

        // 1. MOOD LIGHT — color shifts with dominant emotional state
        //    Warm gold for positive valence, cool blue for negative,
        //    red pulse for stress
        this._updateMoodLight(valence, stress, dt);

        // 2. GROUND GLOW — subtle circle under character
        //    Color = category of dominant weight cluster
        //    Opacity = how "active" the mind is (from entropy/arousal)
        this._updateGroundGlow(store, dt);

        // 3. STIMULUS FLASH — brief, subtle lighting pulse when
        //    a NEW stimulus arrives (not every frame)
        this._checkNewStimuli(stimuli, scene, store, dt);

        // 4. Tick existing transient effects
        this._tickEffects(dt, scene);

        this._prevStimuli = stimuli;
    }

    _updateMoodLight(valence, stress, dt) {
        if (!this._moodLight) return;

        // Target color based on valence
        let r, g, b;
        if (stress > 0.4) {
            // Stress overrides — pulsing red
            const pulse = 0.5 + 0.5 * Math.sin(performance.now() / 1000 * 3);
            r = 0.8; g = 0.1; b = 0.1;
            this._moodLight.intensity += ((stress * 0.5 * pulse) - this._moodLight.intensity) * 3 * dt;
        } else if (valence > 0.1) {
            // Positive — warm gold
            r = 1.0; g = 0.85; b = 0.4;
            this._moodLight.intensity += (valence * 0.3 - this._moodLight.intensity) * 2 * dt;
        } else if (valence < -0.1) {
            // Negative — cool blue
            r = 0.3; g = 0.4; b = 0.8;
            this._moodLight.intensity += (Math.abs(valence) * 0.3 - this._moodLight.intensity) * 2 * dt;
        } else {
            // Neutral — fade out
            r = 1.0; g = 1.0; b = 1.0;
            this._moodLight.intensity += (0 - this._moodLight.intensity) * 2 * dt;
        }
        this._moodLight.color.setRGB(r, g, b);
    }

    _updateGroundGlow(store, dt) {
        if (!this._groundGlow) return;
        const arousal = store.interpolated?.arousal ?? 0.5;
        // Glow opacity based on arousal (more active mind = brighter glow)
        const targetOpacity = Math.max(0, Math.min(0.15, arousal * 0.15));
        const mat = this._groundGlow.material;
        mat.opacity += (targetOpacity - mat.opacity) * 2 * dt;

        // Position follows character
        const pos = store.character?.position;
        if (pos) {
            this._groundGlow.position.x = pos.x;
            this._groundGlow.position.z = pos.z;
        }
    }

    _checkNewStimuli(currentStimuli, scene, store, dt) {
        // Find stimuli that just appeared
        for (const name of currentStimuli) {
            if (this._prevStimuli.has(name)) continue;
            // New stimulus! Create a brief, subtle effect
            this._spawnStimulusEffect(name, scene, store);
        }
    }

    _spawnStimulusEffect(name, scene, store) {
        const charPos = store.character?.position ?? {x:0, z:0};

        // Brief light flash at character position
        // Color depends on stimulus type
        const colors = {
            social_interaction: 0x3CB371,
            challenge: 0xE8613C,
            threat: 0xcc3333,
            reward: 0xDAA520,
            loss: 0x2244aa,
            novelty: 0x4A90D9,
            moral_dilemma: 0x9B59B6,
            flow_state: 0x6080c0,
            social_rejection: 0xcc3333,
            accomplishment: 0xDAA520,
            // Internal events
            mind_wandering: 0x4A90D9,
            sudden_insight: 0xffffff,
            intrusive_thought: 0xcc3333,
            self_doubt: 0x666666,
            nostalgia: 0xd4a050,
            creative_impulse: 0x44bbff,
        };

        const color = colors[name] || 0xffffff;

        // Create a brief point light pulse
        const light = new THREE.PointLight(color, 0, 4);
        light.position.set(charPos.x, 1.5, charPos.z);
        scene.add(light);

        this._activeEffects.push({
            type: 'pulse',
            object: light,
            elapsed: 0,
            duration: 1.5,
            peakIntensity: name === 'threat' || name === 'reward' || name === 'accomplishment' ? 1.0 : 0.5,
        });

        // For reward/accomplishment: also spawn a few subtle rising particles
        if (name === 'reward' || name === 'accomplishment') {
            this._spawnSubtleParticles(scene, charPos, color, name === 'accomplishment' ? 12 : 6);
        }
    }

    _spawnSubtleParticles(scene, charPos, color, count) {
        const group = new THREE.Group();
        group.position.set(charPos.x, 0.1, charPos.z);
        const particles = [];
        for (let i = 0; i < count; i++) {
            const geo = new THREE.SphereGeometry(0.015, 4, 4);
            const mat = new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.8 });
            const mesh = new THREE.Mesh(geo, mat);
            mesh.position.set(
                (Math.random() - 0.5) * 0.5,
                Math.random() * 0.3,
                (Math.random() - 0.5) * 0.5
            );
            mesh.userData.vy = 0.3 + Math.random() * 0.5;
            group.add(mesh);
            particles.push(mesh);
        }
        scene.add(group);
        this._activeEffects.push({
            type: 'particles',
            object: group,
            particles,
            elapsed: 0,
            duration: 2.0,
        });
    }

    _tickEffects(dt, scene) {
        for (let i = this._activeEffects.length - 1; i >= 0; i--) {
            const fx = this._activeEffects[i];
            fx.elapsed += dt;

            if (fx.elapsed >= fx.duration) {
                // Remove
                scene.remove(fx.object);
                if (fx.object.traverse) fx.object.traverse(c => { if (c.geometry) c.geometry.dispose(); if (c.material) c.material.dispose(); });
                this._activeEffects.splice(i, 1);
                continue;
            }

            const t = fx.elapsed / fx.duration;

            if (fx.type === 'pulse') {
                // Quick rise, slow fade
                const envelope = t < 0.2 ? t / 0.2 : 1.0 - (t - 0.2) / 0.8;
                fx.object.intensity = envelope * fx.peakIntensity;
            }

            if (fx.type === 'particles') {
                for (const p of fx.particles) {
                    p.position.y += p.userData.vy * dt;
                    p.material.opacity = Math.max(0, 0.8 * (1 - t));
                }
            }
        }
    }
}
