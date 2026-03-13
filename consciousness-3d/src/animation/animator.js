import { playAnimation } from '../character/model.js';

const BEHAVIOR_TO_ANIM = {
    idle_stand: 'idle',
    idle_fidget: 'idle',
    pacing: 'walk',
    sitting_think: 'idle',
    sitting_slump: 'idle',
    gesture_emphatic: 'idle',
    gesture_dismiss: 'idle',
    startle: 'idle',
};

let currentAnim = null;

export function updateAnimFromBehavior(behavior, intensity) {
    const animName = BEHAVIOR_TO_ANIM[behavior] || 'idle';
    if (animName !== currentAnim) {
        playAnimation(animName, 0.4);
        currentAnim = animName;
    }
}
