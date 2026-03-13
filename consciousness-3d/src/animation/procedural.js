import { REST_POSE } from '../character/skeleton.js';

// startleStartTime is module-level state: reset when startle behavior ends
let startleStartTime = null;
let lastBehavior = null;

/**
 * Returns a complete pose dict for the given behavior.
 * All unspecified joints default to REST_POSE values.
 *
 * @param {string} behaviorName
 * @param {number} intensity  0–1
 * @param {number} t          elapsed seconds (performance.now() / 1000)
 * @param {number} phase      walk-cycle phase in radians (only used by pacing)
 * @returns {Object}  joint → { rotX, rotY, rotZ } plus optional pelvis.posY
 */
export function getPose(behaviorName, intensity, t, phase) {
  // Deep-copy REST_POSE as the base for all unspecified joints
  const pose = {};
  for (const joint in REST_POSE) {
    pose[joint] = { ...REST_POSE[joint] };
  }
  // posY defaults — caller reads pose.pelvis.posY if present
  pose.pelvis.posY = 0.92;

  // Track startle timing
  if (behaviorName === 'startle') {
    if (lastBehavior !== 'startle') {
      startleStartTime = t;
    }
  } else {
    if (lastBehavior === 'startle') {
      startleStartTime = null;
    }
  }
  lastBehavior = behaviorName;

  switch (behaviorName) {

    case 'idle_stand': {
      pose.torso.rotX     = 0.02 * Math.sin(t * 0.4);
      pose.torso.rotZ     = 0.01 * Math.sin(t * 0.3);
      pose.head.rotX      = 0.03 * Math.sin(t * 0.25);
      pose.head.rotY      = 0.04 * Math.sin(t * 0.15);
      pose.shoulderL.rotZ =  0.05;
      pose.shoulderR.rotZ = -0.05;
      pose.elbowL.rotX    = 0.1;
      pose.elbowR.rotX    = 0.1;
      pose.hipL.rotX      = 0;
      pose.hipR.rotX      = 0;
      pose.kneeL.rotX     = 0.02;
      pose.kneeR.rotX     = 0.02;
      pose.pelvis.posY    = 0.92;
      break;
    }

    case 'idle_fidget': {
      pose.torso.rotX     = 0.03 * Math.sin(t * 1.2) * intensity;
      pose.torso.rotZ     = 0.04 * Math.sin(t * 0.9) * intensity;
      pose.head.rotX      = 0.05 * Math.sin(t * 1.5) * intensity;
      pose.head.rotY      = 0.08 * Math.sin(t * 0.7) * intensity;
      pose.shoulderL.rotZ =  0.05 + 0.10 * Math.sin(t * 1.8) * intensity;
      pose.shoulderR.rotZ = -0.05 - 0.08 * Math.sin(t * 2.1) * intensity;
      pose.elbowL.rotX    = 0.3 + 0.20 * Math.sin(t * 1.4) * intensity;
      pose.elbowR.rotX    = 0.2 + 0.15 * Math.sin(t * 1.7) * intensity;
      pose.pelvis.posY    = 0.92 + 0.005 * Math.sin(t * 2.0) * intensity;
      pose.hipL.rotX      =  0.03 * Math.sin(t * 0.8) * intensity;
      pose.hipR.rotX      = -0.03 * Math.sin(t * 0.8) * intensity;
      break;
    }

    case 'pacing': {
      // phase is provided by locomotion.js
      pose.hipL.rotX      =  0.35 * Math.sin(phase) * intensity;
      pose.hipR.rotX      =  0.35 * Math.sin(phase + Math.PI) * intensity;
      pose.kneeL.rotX     = Math.max(0, 0.5 * Math.sin(phase - 0.5)) * intensity;
      pose.kneeR.rotX     = Math.max(0, 0.5 * Math.sin(phase + Math.PI - 0.5)) * intensity;
      pose.footL.rotX     = 0.15 * Math.sin(phase + 0.3) * intensity;
      pose.footR.rotX     = 0.15 * Math.sin(phase + Math.PI + 0.3) * intensity;

      pose.shoulderL.rotX = -0.2 * Math.sin(phase) * intensity;
      pose.shoulderR.rotX = -0.2 * Math.sin(phase + Math.PI) * intensity;
      pose.elbowL.rotX    = 0.3 + 0.15 * Math.sin(phase + 0.5) * intensity;
      pose.elbowR.rotX    = 0.3 + 0.15 * Math.sin(phase + Math.PI + 0.5) * intensity;

      pose.torso.rotZ     = 0.03 * Math.sin(phase) * intensity;
      // pelvis.rotY is set externally by locomotion to face movement direction
      pose.pelvis.posY    = 0.92 + 0.02 * Math.abs(Math.sin(phase * 2)) * intensity;
      break;
    }

    case 'sitting_think': {
      pose.pelvis.posY    = 0.45;
      pose.hipL.rotX      = -1.57;
      pose.hipR.rotX      = -1.57;
      pose.kneeL.rotX     =  1.57;
      pose.kneeR.rotX     =  1.57;
      pose.torso.rotX     = 0.15 + 0.03 * Math.sin(t * 0.3);
      pose.head.rotX      = 0.10 + 0.05 * Math.sin(t * 0.2) * intensity;
      pose.head.rotY      = 0.06 * Math.sin(t * 0.12);
      pose.shoulderL.rotX = -0.3;
      pose.elbowL.rotX    = 1.8 + 0.1 * Math.sin(t * 0.25) * intensity;
      pose.shoulderR.rotX = -0.1;
      pose.elbowR.rotX    =  0.5;
      break;
    }

    case 'sitting_slump': {
      pose.pelvis.posY    = 0.42;
      pose.hipL.rotX      = -1.57;
      pose.hipR.rotX      = -1.57;
      pose.kneeL.rotX     =  1.57;
      pose.kneeR.rotX     =  1.8;
      pose.torso.rotX     = 0.35 + 0.02 * Math.sin(t * 0.15);
      pose.torso.rotZ     = 0.05;
      pose.head.rotX      = 0.3;
      pose.head.rotZ      = 0.08 * Math.sin(t * 0.1);
      pose.shoulderL.rotZ =  0.15;
      pose.shoulderR.rotZ = -0.15;
      pose.elbowL.rotX    = 0.8;
      pose.elbowR.rotX    = 0.7;
      break;
    }

    case 'gesture_emphatic': {
      pose.torso.rotX     = -0.05 + 0.05 * Math.sin(t * 1.5) * intensity;
      pose.torso.rotZ     =  0.04 * Math.sin(t * 1.2) * intensity;
      pose.head.rotY      =  0.10 * Math.sin(t * 0.8) * intensity;

      pose.shoulderR.rotX = -0.8 - 0.4 * Math.sin(t * 2.0) * intensity;
      pose.shoulderR.rotZ = -0.3 - 0.2 * Math.sin(t * 1.6) * intensity;
      pose.elbowR.rotX    =  0.8 + 0.6 * Math.sin(t * 2.5) * intensity;
      pose.handR.rotX     =  0.3 * Math.sin(t * 3.0) * intensity;

      pose.shoulderL.rotX = -0.3 - 0.2 * Math.sin(t * 1.8) * intensity;
      pose.elbowL.rotX    =  0.5 + 0.3 * Math.sin(t * 2.2) * intensity;

      pose.hipL.rotX      =  0.03 * Math.sin(t * 0.6);
      pose.hipR.rotX      = -0.03 * Math.sin(t * 0.6);
      break;
    }

    case 'gesture_dismiss': {
      pose.shoulderR.rotX = -0.6 * intensity;
      pose.shoulderR.rotZ = -0.4 * intensity;
      pose.elbowR.rotX    =  0.5;
      pose.handR.rotX     = -0.3 + 0.4 * Math.sin(t * 4.0) * intensity;
      pose.head.rotY      = -0.15 * intensity;
      pose.torso.rotZ     = -0.03 * intensity;
      break;
    }

    case 'startle': {
      const startT = startleStartTime !== null ? startleStartTime : t;
      const startleT = t - startT;
      const decay = Math.max(0, 1.0 - startleT / 0.4);

      pose.torso.rotX     = -0.15 * decay * intensity;
      pose.shoulderL.rotX = -0.5  * decay * intensity;
      pose.shoulderR.rotX = -0.5  * decay * intensity;
      pose.shoulderL.rotZ =  0.3  * decay * intensity;
      pose.shoulderR.rotZ = -0.3  * decay * intensity;
      pose.elbowL.rotX    =  1.2  * decay * intensity;
      pose.elbowR.rotX    =  1.2  * decay * intensity;
      pose.head.rotX      = -0.1  * decay * intensity;
      pose.kneeL.rotX     =  0.15 * decay * intensity;
      pose.kneeR.rotX     =  0.15 * decay * intensity;
      pose.pelvis.posY    = 0.92 - 0.03 * decay * intensity;
      break;
    }

    default:
      // Unknown behavior — return REST_POSE as-is
      break;
  }

  return pose;
}
