#!/usr/bin/env python3
"""Consciousness Simulation — Entry Point

Usage:
    python3 main.py                          # default 20 aspects, rich dynamics
    python3 main.py --expanded               # 32-aspect model
    python3 main.py --personality seeker      # with personality bias
    python3 main.py --headless               # no visualization
    python3 main.py --expanded --personality empathic

Controls:
    UP arrow   — inject positive environmental input
    DOWN arrow — inject negative environmental input
    Hold the key for sustained influence
"""

import argparse
from core import SimConfig, EXPANDED_32_CONFIG, ConsciousnessEngine


def main():
    parser = argparse.ArgumentParser(description='Consciousness Simulation')
    parser.add_argument('--expanded', action='store_true',
                        help='Use 32-aspect expanded model')
    parser.add_argument('--personality', type=str, default=None,
                        choices=['contemplative', 'action-oriented', 'empathic',
                                 'analytical', 'resilient', 'seeker'],
                        help='Personality profile to apply')
    parser.add_argument('--headless', action='store_true',
                        help='Run without visualization')
    parser.add_argument('--no-input', action='store_true',
                        help='Disable keyboard input')
    parser.add_argument('--interval', type=int, default=50,
                        help='Animation interval in ms (default: 50)')
    parser.add_argument('--energy', type=float, default=100.0,
                        help='Energy budget (default: 100)')
    parser.add_argument('--circadian', type=int, default=2000,
                        help='Circadian period in ticks (0=disabled)')
    args = parser.parse_args()

    if args.expanded:
        config = EXPANDED_32_CONFIG(
            personality_profile=args.personality,
            vis_enabled=not args.headless,
            vis_interval_ms=args.interval,
            input_backend='none' if args.no_input else 'pynput',
            energy_budget=args.energy,
            circadian_period=args.circadian,
        )
    else:
        config = SimConfig(
            personality_profile=args.personality,
            vis_enabled=not args.headless,
            vis_interval_ms=args.interval,
            input_backend='none' if args.no_input else 'pynput',
            energy_budget=args.energy,
            circadian_period=args.circadian,
        )

    engine = ConsciousnessEngine(config)
    engine.run()


if __name__ == '__main__':
    main()
