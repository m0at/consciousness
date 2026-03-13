import { BEHAVIORS } from './behaviors.js';

const KNOWN_BEHAVIORS = new Set(Object.keys(BEHAVIORS));

const DEFAULT = { primary: 'idle_stand', scores: {}, intensity: 0.0 };

export function readBehavior(tickData) {
  if (!tickData || !tickData.behavior) return DEFAULT;

  const { primary, scores, intensity } = tickData.behavior;

  if (!primary || !KNOWN_BEHAVIORS.has(primary)) return DEFAULT;

  return {
    primary,
    scores: scores ?? {},
    intensity: typeof intensity === 'number' ? intensity : 0.0,
  };
}
