#!/usr/bin/env python3
"""Bridge between the ConsciousnessEngine and an Electron/Three.js frontend.

Runs the simulation headless, outputs JSON lines to stdout each tick,
reads JSON commands from stdin (non-blocking). Designed to be spawned
as a child process by the Electron app.

Usage:
    python3 bridge.py                           # default 20 aspects, 20 Hz
    python3 bridge.py --expanded                # 32-aspect model
    python3 bridge.py --personality seeker      # with personality
    python3 bridge.py --tick-rate 30            # 30 Hz
"""

from __future__ import annotations

import argparse
import json
import os
import re
import signal
import sys
import threading
import time
from collections import deque
from dataclasses import asdict, dataclass

# Ensure unbuffered stdout so the JS side gets lines immediately
os.environ['PYTHONUNBUFFERED'] = '1'

# Add project root to path so `core` is importable when run from app/
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from core import SimConfig, EXPANDED_32_CONFIG, ConsciousnessEngine
from core.config import DEFAULT_CATEGORIES, EXPANDED_CATEGORIES, CATEGORY_COLORS
from core.personality import PROFILES


# ---------------------------------------------------------------------------
# Behavior scoring (DESIGN.md Section 3.2)
# ---------------------------------------------------------------------------

BEHAVIORS = [
    'idle_stand', 'idle_fidget', 'pacing', 'sitting_think',
    'sitting_slump', 'gesture_emphatic', 'gesture_dismiss', 'startle',
]

_HYSTERESIS = 0.12


def _neg(x: float) -> float:
    return max(0.0, -x)


def _clamp01(x: float) -> float:
    return max(0.0, min(1.0, x))


def compute_behavior(weight_dict: dict[str, float],
                     energy_state: dict,
                     prev_stress: float,
                     current_behavior: str) -> tuple[dict, str]:
    """Score each behavior per DESIGN.md §3.2; apply hysteresis.

    Returns (behavior_dict, new_current_behavior).
    """
    w = weight_dict
    stress = energy_state.get('stress', 0.0)
    stress_spike = max(0.0, stress - prev_stress)

    scores: dict[str, float] = {
        'idle_stand': _clamp01(
            0.3
            + 0.2 * w.get('self_regulation', 0.0)
            + 0.15 * w.get('patience', 0.0)
            - 0.2 * abs(w.get('motivation', 0.0))
            - 0.15 * abs(w.get('agency', 0.0))
        ),
        'idle_fidget': _clamp01(
            0.1
            + 0.3 * abs(w.get('emotional_awareness', 0.0))
            + 0.2 * stress
            - 0.2 * w.get('self_regulation', 0.0)
            + 0.15 * w.get('body_awareness', 0.0)
        ),
        'pacing': _clamp01(
            0.15
            + 0.25 * w.get('motivation', 0.0)
            + 0.2 * w.get('agency', 0.0)
            + 0.15 * abs(w.get('introspection', 0.0))
            - 0.1 * w.get('self_regulation', 0.0)
            + 0.1 * stress
        ),
        'sitting_think': _clamp01(
            0.1
            + 0.3 * w.get('introspection', 0.0)
            + 0.25 * w.get('reflection', 0.0)
            + 0.2 * w.get('metacognition', 0.0)
            - 0.15 * w.get('agency', 0.0)
        ),
        'sitting_slump': _clamp01(
            0.05
            + 0.3 * _neg(w.get('self_esteem', 0.0))
            + 0.2 * _neg(w.get('motivation', 0.0))
            + 0.2 * _neg(w.get('agency', 0.0))
            + 0.15 * stress
        ),
        'gesture_emphatic': _clamp01(
            0.1
            + 0.3 * w.get('social_awareness', 0.0)
            + 0.25 * w.get('theory_of_mind', 0.0)
            + 0.2 * w.get('emotional_awareness', 0.0)
            + 0.15 * w.get('agency', 0.0)
        ),
        'gesture_dismiss': _clamp01(
            0.05
            + 0.2 * _neg(w.get('social_awareness', 0.0))
            + 0.2 * _neg(w.get('theory_of_mind', 0.0))
            + 0.15 * w.get('agency', 0.0)
            + 0.1 * w.get('self_regulation', 0.0)
        ),
        'startle': _clamp01(
            0.0
            + 0.5 * stress_spike
            + 0.3 * w.get('situational_awareness', 0.0)
            + 0.2 * w.get('body_awareness', 0.0)
        ),
    }

    # Round for wire format
    rounded = {b: round(s, 4) for b, s in scores.items()}

    # Hysteresis: only switch if new best exceeds current by >= 0.12
    best = max(scores, key=scores.__getitem__)
    if current_behavior == '' or current_behavior not in scores:
        new_behavior = best
    else:
        current_score = scores[current_behavior]
        if scores[best] >= current_score + _HYSTERESIS:
            new_behavior = best
        else:
            new_behavior = current_behavior

    return {
        'primary': new_behavior,
        'scores': rounded,
        'intensity': round(scores[new_behavior], 4),
    }, new_behavior


# ---------------------------------------------------------------------------
# State serialization helpers
# ---------------------------------------------------------------------------

def serialize_analysis(analysis: dict) -> dict:
    """Convert analyzer dataclasses to the JSON schema expected by DESIGN.md §3.1."""
    out: dict = {}

    # phases: list of per-aspect phase objects
    if 'phases' in analysis:
        phases_list = []
        for ps in analysis['phases']:
            phases_list.append({
                'phase': ps.phase,
                'aspect': ps.aspect,
                'confidence': round(ps.confidence, 3),
                'duration': ps.duration,
                'slope': round(ps.slope, 6),
            })
        out['phases'] = phases_list
    else:
        out['phases'] = []

    # entropy: match exact schema keys including complexity_label
    if 'entropy' in analysis:
        e = analysis['entropy']
        out['entropy'] = {
            'shannon': round(e.shannon, 4),
            'normalized': round(e.normalized, 4),
            'delta': round(e.delta, 6),
            'complexity_label': e.complexity_label,
        }
    else:
        out['entropy'] = None

    # cascades
    if 'cascades' in analysis:
        out['cascades'] = [
            {
                'trigger': c.trigger_aspect,
                'trigger_delta': round(c.trigger_delta, 4),
                'path': [(a, round(d, 4), depth) for a, d, depth in c.path],
                'magnitude': round(c.total_magnitude, 4),
                'tick': c.timestamp,
            }
            for c in analysis['cascades'][-5:]
        ]
    else:
        out['cascades'] = []

    # attractors: match schema (basin_radius, strength, drift_rate — no age)
    if 'attractors' in analysis:
        out['attractors'] = [
            {
                'basin_radius': round(a.basin_radius, 4),
                'strength': round(a.strength, 4),
                'drift_rate': round(a.drift_rate, 6),
            }
            for a in analysis['attractors'][-3:]
        ]
    else:
        out['attractors'] = []

    # resilience
    if 'resilience' in analysis and analysis['resilience'] is not None:
        r = analysis['resilience']
        out['resilience'] = {
            'displacement': round(r['displacement'], 4),
            'elapsed': r['elapsed'],
            'elasticity': round(r['elasticity'], 4),
            'recovered': r['recovered'],
        }
    else:
        out['resilience'] = None

    return out


def parse_env_status(status: str) -> tuple[list[str], float]:
    """Parse envStatus string into (activeStimuli, valence).

    Expected format: "stimulus_a, stimulus_b | valence=+0.12"
    Stimuli part is everything before the first " | ", split by ", ".
    Valence is extracted from "valence=<number>".
    """
    active_stimuli: list[str] = []
    valence = 0.0

    if not status:
        return active_stimuli, valence

    parts = status.split(' | ')
    # First part: comma-separated stimulus names
    stimuli_part = parts[0].strip()
    if stimuli_part:
        active_stimuli = [s.strip() for s in stimuli_part.split(',') if s.strip()]

    # Remaining parts: look for valence=<number>
    for part in parts[1:]:
        m = re.search(r'valence=([+-]?\d+\.?\d*)', part)
        if m:
            try:
                valence = float(m.group(1))
            except ValueError:
                pass

    return active_stimuli, valence


def serialize_memory(memory_system) -> dict:
    """Extract memory state for JSON output."""
    strongest = memory_system.get_strongest_memories(k=5)
    recent_memories = []
    for mem, strength in strongest:
        recent_memories.append({
            'emotion': mem.emotion,
            'valence': round(mem.valence, 3),
            'intensity': round(mem.intensity, 3),
            'strength': round(strength, 4),
            'frame': mem.frame,
            'reinforced': mem.reinforcement_count,
        })

    if memory_system.stm:
        state = memory_system.stm[-1]
        dominant_emotion = memory_system._classify_emotion(state)
    else:
        dominant_emotion = 'neutral'

    return {
        'count': len(memory_system.memories),
        'dominant_emotion': dominant_emotion,
        'recent_memories': recent_memories,
        'detected_cycles': memory_system.get_detected_cycles(),
    }


def serialize_personality(personality_system, weight_dict: dict) -> dict | None:
    """Extract personality state for JSON output."""
    if personality_system is None:
        return None

    viz = personality_system.get_visualization_data(weight_dict)

    conflict_summary = {
        a: round(v, 4)
        for a, v in viz['conflict_level'].items()
        if v > 0.01
    }

    drift_summary = {
        a: round(v, 4)
        for a, v in viz['bias_drift'].items()
        if abs(v) > 0.001
    }

    return {
        'name': viz['personality_name'],
        'conflict': conflict_summary,
        'drift': drift_summary,
    }


# ---------------------------------------------------------------------------
# Non-blocking stdin reader
# ---------------------------------------------------------------------------

class StdinReader:
    """Reads JSON lines from stdin in a background thread."""

    def __init__(self):
        self._queue: deque = deque(maxlen=64)
        self._running = False
        self._thread: threading.Thread | None = None

    def start(self):
        self._running = True
        self._thread = threading.Thread(target=self._read_loop, daemon=True)
        self._thread.start()

    def stop(self):
        self._running = False

    def drain(self) -> list[dict]:
        """Return all queued commands and clear the queue."""
        cmds = list(self._queue)
        self._queue.clear()
        return cmds

    def _read_loop(self):
        while self._running:
            try:
                line = sys.stdin.readline()
                if not line:
                    self._running = False
                    break
                line = line.strip()
                if not line:
                    continue
                try:
                    cmd = json.loads(line)
                    self._queue.append(cmd)
                except json.JSONDecodeError:
                    _emit_stderr(f'invalid JSON on stdin: {line[:200]}')
            except Exception:
                break


# ---------------------------------------------------------------------------
# Output helpers
# ---------------------------------------------------------------------------

def _emit(obj: dict):
    """Write a single JSON line to stdout."""
    try:
        sys.stdout.write(json.dumps(obj, separators=(',', ':')) + '\n')
        sys.stdout.flush()
    except BrokenPipeError:
        _shutdown()


def _emit_stderr(msg: str):
    """Write a diagnostic message to stderr."""
    try:
        sys.stderr.write(f'[bridge] {msg}\n')
        sys.stderr.flush()
    except Exception:
        pass


# ---------------------------------------------------------------------------
# Graceful shutdown
# ---------------------------------------------------------------------------

_shutdown_flag = threading.Event()


def _shutdown(*_args):
    _shutdown_flag.set()


def _install_signal_handlers():
    signal.signal(signal.SIGTERM, _shutdown)
    signal.signal(signal.SIGINT, _shutdown)
    if hasattr(signal, 'SIGPIPE'):
        signal.signal(signal.SIGPIPE, signal.SIG_DFL)


def _parent_alive() -> bool:
    """Check if the parent process is still alive (Unix only)."""
    try:
        ppid = os.getppid()
        return ppid != 1
    except Exception:
        return True


# ---------------------------------------------------------------------------
# Command handlers
# ---------------------------------------------------------------------------

# Mutable flag for active input injection direction (None = no injection)
_active_input_direction: list[str | None] = [None]


def handle_commands(engine: ConsciousnessEngine, commands: list[dict],
                    expanded: bool, tick_rate_ref: list[float]):
    """Process incoming commands from the JS frontend."""
    for cmd in commands:
        cmd_type = cmd.get('type', '')

        if cmd_type == 'input':
            direction = cmd.get('direction', 'positive')
            if direction == 'none':
                _active_input_direction[0] = None
            else:
                _active_input_direction[0] = direction
                user_input = {
                    'active': True,
                    'direction': direction,
                    'text': f'{direction} input',
                }
                engine.weight_dict = engine.environment.apply_input(
                    engine.weight_dict, user_input)
                engine.analyzer.mark_perturbation()
                engine.energy.inject_perturbation(0.3)

        elif cmd_type == 'config':
            if 'personality' in cmd:
                pname = cmd['personality']
                if pname in PROFILES:
                    from core.personality import PersonalitySystem
                    engine.personality = PersonalitySystem(
                        pname, engine.config.aspects)
                    _emit_stderr(f'personality changed to {pname}')
                elif pname is None or pname == 'none':
                    engine.personality = None
                    _emit_stderr('personality disabled')
                else:
                    _emit_stderr(f'unknown personality: {pname}')

            if 'tick_rate' in cmd:
                rate = max(1, min(120, int(cmd['tick_rate'])))
                tick_rate_ref[0] = rate
                _emit_stderr(f'tick rate changed to {rate} Hz')

            if 'expanded' in cmd:
                _emit({'type': 'error',
                       'message': 'hot-swap to expanded model not supported; restart with --expanded'})

        elif cmd_type == 'ping':
            _emit({'type': 'pong', 'tick': engine.tick})

        elif cmd_type == 'reset':
            engine.weight_dict = engine.config.get_initial_weights()
            engine.tick = 0
            _active_input_direction[0] = None
            _emit_stderr('engine reset to initial state')

        else:
            _emit_stderr(f'unknown command type: {cmd_type}')


# ---------------------------------------------------------------------------
# Main loop
# ---------------------------------------------------------------------------

def build_engine(args) -> tuple[ConsciousnessEngine, bool]:
    """Create the engine from CLI args."""
    expanded = args.expanded
    if expanded:
        config = EXPANDED_32_CONFIG(
            personality_profile=args.personality,
            vis_enabled=False,
            input_backend='none',
            energy_budget=args.energy,
            circadian_period=args.circadian,
        )
    else:
        config = SimConfig(
            personality_profile=args.personality,
            vis_enabled=False,
            input_backend='none',
            energy_budget=args.energy,
            circadian_period=args.circadian,
        )
    return ConsciousnessEngine(config), expanded


def emit_init(engine: ConsciousnessEngine, expanded: bool, tick_rate: int):
    """Send the startup handshake message (type: 'init') per DESIGN.md §3.1."""
    # categories as dict of category → [aspects]  (not aspect → category)
    categories = EXPANDED_CATEGORIES if expanded else DEFAULT_CATEGORIES
    cat_dict = {cat: list(members) for cat, members in categories.items()}

    _emit({
        'type': 'init',
        'aspects': list(engine.config.aspects),
        'categories': cat_dict,
        'categoryColors': dict(CATEGORY_COLORS),
        'personality': engine.config.personality_profile,
        'aspectCount': len(engine.config.aspects),
    })


def run_loop(engine: ConsciousnessEngine, expanded: bool, tick_rate: int):
    """Main tick loop: step engine, serialize state, emit JSON line."""
    tick_interval = 1.0 / tick_rate
    tick_rate_ref = [float(tick_rate)]

    reader = StdinReader()
    reader.start()

    PARENT_CHECK_INTERVAL = 20

    # Behavior hysteresis state
    current_behavior: str = ''
    prev_stress: float = 0.0

    try:
        while not _shutdown_flag.is_set():
            t0 = time.monotonic()

            # Process incoming commands
            commands = reader.drain()
            if commands:
                handle_commands(engine, commands, expanded, tick_rate_ref)
                tick_interval = 1.0 / tick_rate_ref[0]

            # Apply continuous input injection if active
            if _active_input_direction[0] is not None:
                direction = _active_input_direction[0]
                user_input = {
                    'active': True,
                    'direction': direction,
                    'text': f'{direction} input',
                }
                engine.weight_dict = engine.environment.apply_input(
                    engine.weight_dict, user_input)

            # Step simulation
            result = engine.step()

            weights = result['weights']
            energy_state = result['energy']
            analysis = result.get('analysis', {})

            # Round weights
            rounded_weights = {a: round(w, 6) for a, w in weights.items()}

            # Current stress for next tick's spike detection
            current_stress = energy_state.get('stress', 0.0)

            # Compute behavior with hysteresis
            behavior_dict, current_behavior = compute_behavior(
                weights, energy_state, prev_stress, current_behavior)

            prev_stress = current_stress

            # Environment status, activeStimuli, valence
            env_status: str = engine.environment.get_status()
            active_stimuli, valence = parse_env_status(env_status)

            output = {
                'type': 'tick',
                'tick': result['tick'],
                'weights': rounded_weights,
                'energy': energy_state,
                'analysis': serialize_analysis(analysis),
                'envStatus': env_status,
                'activeStimuli': active_stimuli,
                'valence': round(valence, 6),
                'behavior': behavior_dict,
            }

            _emit(output)

            if result['tick'] % PARENT_CHECK_INTERVAL == 0:
                if not _parent_alive():
                    _emit_stderr('parent process died, shutting down')
                    break

            elapsed = time.monotonic() - t0
            sleep_time = tick_interval - elapsed
            if sleep_time > 0.001:
                _shutdown_flag.wait(timeout=sleep_time)

    except BrokenPipeError:
        _emit_stderr('broken pipe, shutting down')
    except KeyboardInterrupt:
        pass
    finally:
        reader.stop()
        _emit_stderr(f'bridge stopped at tick {engine.tick}')


def main():
    _install_signal_handlers()

    parser = argparse.ArgumentParser(
        description='Consciousness Engine JSON Bridge')
    parser.add_argument('--expanded', action='store_true',
                        help='Use 32-aspect expanded model')
    parser.add_argument('--personality', type=str, default=None,
                        choices=list(PROFILES.keys()),
                        help='Initial personality profile')
    parser.add_argument('--tick-rate', type=int, default=20,
                        help='Simulation ticks per second (default: 20)')
    parser.add_argument('--energy', type=float, default=100.0,
                        help='Energy budget (default: 100)')
    parser.add_argument('--circadian', type=int, default=2000,
                        help='Circadian period in ticks (0=disabled)')
    args = parser.parse_args()

    engine, expanded = build_engine(args)

    emit_init(engine, expanded, args.tick_rate)

    run_loop(engine, expanded, args.tick_rate)


if __name__ == '__main__':
    main()
