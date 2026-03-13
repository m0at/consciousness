"""ConsciousnessEngine: central orchestrator that wires all subsystems
together and runs the simulation loop."""

from __future__ import annotations
import numpy as np

from .config import SimConfig
from .interrelations import InterrelationMatrix
from .dynamics import DynamicsEngine
from .environment import Environment
from .analyzer import SystemAnalyzer
from .personality import PersonalitySystem
from .memory import MemorySystem
from .energy import EnergySystem
from .visualization import ConsciousnessVisualizer


class ConsciousnessEngine:
    """Central orchestrator for the consciousness simulation.

    Each tick executes subsystems in order:
      1. Energy — compute available energy and arousal modifiers
      2. Environment — generate stimuli + poll user input
      3. Personality — compute bias forces, register conflict
      4. Interrelation — propagate inter-aspect influences
      5. Dynamics — update weights with momentum, adaptive LR, homeostasis
      6. Memory — record state, compute memory-based influences
      7. Analyzer — detect emergent patterns
      8. Visualization — render frame
    """

    def __init__(self, config: SimConfig | None = None):
        self.config = config or SimConfig()
        self.tick = 0

        # Weight state
        self.weight_dict = self.config.get_initial_weights()
        self._initial_abs_sum = sum(abs(v) for v in self.weight_dict.values())

        # Rolling history
        self.weight_history: list[list[float]] = []
        self.tick_history: list[int] = []

        # Initialize all subsystems
        aspects = self.config.aspects
        initial_weights = self.config.get_initial_weights()

        # Interrelation matrix
        self.interrelation = InterrelationMatrix()
        self.interrelation.build(aspects, self.config.interrelationships)

        # Dynamics engine
        self.dynamics = DynamicsEngine()
        self.dynamics.init_state(aspects, initial_weights, self.config)

        # Environment
        self.environment = Environment(aspects, backend=self.config.input_backend)

        # Analyzer
        self.analyzer = SystemAnalyzer(
            aspects, self.interrelation.matrix, window=50)

        # Personality (optional)
        self.personality: PersonalitySystem | None = None
        if self.config.personality_profile:
            self.personality = PersonalitySystem(
                self.config.personality_profile, aspects)

        # Memory
        self.memory = MemorySystem(
            aspects, stm_size=self.config.memory_stm_size,
            ltm_capacity=self.config.memory_ltm_capacity,
            influence_scale=self.config.memory_influence_scale)

        # Energy
        self.energy = EnergySystem(
            aspects, initial_weights,
            max_energy=self.config.energy_budget,
            attention_slots=self.config.attention_slots,
            circadian_period=self.config.circadian_period)

        # Visualization
        self.viz: ConsciousnessVisualizer | None = None
        if self.config.vis_enabled:
            self.viz = ConsciousnessVisualizer(self.config)
            self.viz.setup()

    def _scale_weights(self, weights: list[float]) -> list[float]:
        current = sum(abs(w) for w in weights)
        if current == 0:
            return weights
        factor = self._initial_abs_sum / current
        return [w * factor for w in weights]

    def step(self) -> dict:
        """Execute one simulation tick. Returns tick summary."""
        aspects = self.config.aspects
        self.tick += 1

        # 1. Energy
        energy_state = self.energy.step(self.weight_dict)
        aspect_scales = energy_state['aspect_scales']

        # 2. Environment: stimuli + user input
        weights_list = [self.weight_dict[a] for a in aspects]
        stimuli = self.environment.generate_stimuli(weights_list, self.tick)
        user_input = self.environment.get_user_input()

        # Scale stimuli by energy/attention
        for i, a in enumerate(aspects):
            stimuli[i] *= aspect_scales.get(a, 1.0)
        stimuli = [s * energy_state['noise_modifier'] for s in stimuli]

        # 3. Personality
        personality_biases = None
        if self.personality:
            self.personality.register_input(self.weight_dict, stimuli)
            stimuli = self.personality.apply_conflict_volatility(stimuli)
            personality_biases = self.personality.compute_biases(self.weight_dict)

        # 4. Interrelation matrix
        rebalanced = self.interrelation.propagate(weights_list)

        # 5. Memory influences (from previous tick's state)
        memory_influences = self.memory.compute_influences(self.weight_dict)

        # 6. Dynamics update
        energy_mods = {a: energy_state['lr_modifier'] for a in aspects}
        updated = self.dynamics.update(
            rebalanced, stimuli, aspects,
            energy_modifiers=energy_mods,
            personality_biases=personality_biases,
            memory_influences=memory_influences)

        # Scale to preserve total magnitude
        updated = self._scale_weights(updated)

        # Apply user input
        self.weight_dict = dict(zip(aspects, updated))
        if user_input['active']:
            self.weight_dict = self.environment.apply_input(
                self.weight_dict, user_input)
            self.analyzer.mark_perturbation()
            self.energy.inject_perturbation(0.3)

        # 7. Memory: record current state
        current_stimuli = stimuli
        self.memory.process_frame(
            [self.weight_dict[a] for a in aspects], current_stimuli)

        # 8. Analyzer
        analysis = self.analyzer.tick_update(self.weight_dict)

        # Update history
        snapshot = [self.weight_dict[a] for a in aspects]
        self.weight_history.append(snapshot)
        self.tick_history.append(self.tick)

        window = self.config.vis_rolling_window
        if len(self.weight_history) > window:
            self.weight_history = self.weight_history[-window:]
            self.tick_history = self.tick_history[-window:]

        return {
            'tick': self.tick,
            'weights': dict(self.weight_dict),
            'analysis': analysis,
            'energy': self.energy.get_state(),
            'user_input': user_input,
            'env_status': self.environment.get_status(),
        }

    def render(self, tick_result: dict):
        """Render one frame via the visualization subsystem."""
        if self.viz is None:
            return
        self.viz.render(
            weight_dict=tick_result['weights'],
            interrelation_matrix=self.interrelation.matrix,
            user_input=tick_result['user_input'],
            analysis=tick_result.get('analysis'),
            energy_state=tick_result.get('energy'),
            env_status=tick_result.get('env_status'),
            iteration=tick_result['tick'])

    def print_initial_state(self):
        print('Initial Weights:')
        print(f"{'Aspect':<26}{'Weight':>8}  {'LR':>6}  {'Mom':>6}  {'Category':<12}")
        for a, w in sorted(self.weight_dict.items(), key=lambda x: x[1], reverse=True):
            lr = self.config.get_learning_rate(a)
            mu = self.config.get_momentum(a)
            cat = self.config.aspect_category(a)
            print(f'{a:<26}{w:>8.4f}  {lr:>6.3f}  {mu:>6.2f}  {cat:<12}')
        print()
        if self.personality:
            print(f'Personality: {self.personality.profile.name}')
            print(f'  {self.personality.profile.description}')
            print()

    def run(self):
        """Main entry point: print state, set up animation, run."""
        from matplotlib.animation import FuncAnimation
        import matplotlib.pyplot as plt

        self.print_initial_state()

        # Seed one tick
        result = self.step()

        if self.viz is None:
            # Headless mode
            print('Running headless (no visualization)...')
            try:
                while True:
                    result = self.step()
                    if self.tick % 100 == 0:
                        summary = self.analyzer.summary()
                        print(f"Tick {self.tick}: phase={summary.get('dominant_phase')} "
                              f"entropy={summary.get('entropy', {}).get('label', '?')} "
                              f"energy={result['energy']['energy_pct']}%")
            except KeyboardInterrupt:
                print(f'\nStopped at tick {self.tick}.')
            return

        def animate(frame):
            result = self.step()
            self.render(result)

        ani = FuncAnimation(self.viz.fig, animate,
                            interval=self.config.vis_interval_ms,
                            cache_frame_data=False)
        plt.show()
