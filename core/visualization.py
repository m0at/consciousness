"""Visualization dashboard: radar chart, heatmap, phase space, time series,
mental state indicator, and input pulse effects."""

from __future__ import annotations
import numpy as np
from collections import deque
from matplotlib.collections import LineCollection


def detect_mental_state(weights, weight_history):
    """Heuristic mental state detection from weight statistics."""
    w = np.array(weights)
    std = np.std(w)
    mean = np.mean(np.abs(w))

    if len(weight_history) >= 5:
        recent = np.array(weight_history[-5:])
        deltas = np.diff(recent, axis=0)
        avg_delta = np.mean(np.abs(deltas))
        trend = np.mean(deltas)
    else:
        avg_delta = 0.0
        trend = 0.0

    if std > 0.45 and avg_delta > 0.02:
        return 'agitated', '#E8613C'
    if trend > 0.005 and mean > 0.3:
        return 'growing', '#3CB371'
    if trend < -0.005:
        return 'recovering', '#4A90D9'
    if std > 0.35:
        return 'conflicted', '#9B59B6'
    return 'calm', '#DAA520'


class ConsciousnessVisualizer:
    """Rich dashboard visualization for the consciousness simulation.

    Renders: radar chart, dynamic coupling heatmap, PCA phase space,
    weight time series, mental state indicator, input pulse effects.
    """

    HISTORY = 120
    TRAIL_LEN = 80
    PULSE_FRAMES = 12

    def __init__(self, config):
        from .config import CATEGORY_COLORS
        self.config = config
        self.aspects = config.aspects
        self.num = len(self.aspects)
        self.colors = [config.aspect_color(a) for a in self.aspects]
        self.category_colors = CATEGORY_COLORS

        # History buffers
        self.weight_history: deque = deque(maxlen=self.HISTORY)
        self.phase_trail_x: deque = deque(maxlen=self.TRAIL_LEN)
        self.phase_trail_y: deque = deque(maxlen=self.TRAIL_LEN)

        # Input pulse state
        self.pulse_remaining = 0
        self.pulse_kind = None
        self.pulse_aspect = None

        self.fig = None
        self.ax_radar = None
        self.ax_state = None
        self.ax_heat = None
        self.ax_ts = None
        self.ax_phase = None
        self.ax_pulse = None

    def setup(self):
        import matplotlib
        matplotlib.use('macosx')
        import matplotlib.pyplot as plt

        self.fig = plt.figure(figsize=(16, 9.5), facecolor='#0D1117')
        self.fig.subplots_adjust(left=0.06, right=0.94, top=0.93, bottom=0.06,
                                 wspace=0.35, hspace=0.38)
        self.fig.suptitle('Consciousness Simulation Dashboard',
                          color='white', fontsize=16, fontweight='bold', y=0.98)

        gs = self.fig.add_gridspec(3, 3, height_ratios=[1.1, 1.0, 0.15])

        self.ax_radar = self.fig.add_subplot(gs[0, 0], polar=True, facecolor='#0D1117')
        self.ax_state = self.fig.add_subplot(gs[0, 1], facecolor='#0D1117')
        self.ax_heat = self.fig.add_subplot(gs[0, 2], facecolor='#0D1117')
        self.ax_ts = self.fig.add_subplot(gs[1, 0:2], facecolor='#0D1117')
        self.ax_phase = self.fig.add_subplot(gs[1, 2], facecolor='#0D1117')
        self.ax_pulse = self.fig.add_subplot(gs[2, :], facecolor='#0D1117')

        for ax in [self.ax_state, self.ax_heat, self.ax_ts, self.ax_phase, self.ax_pulse]:
            ax.tick_params(colors='#888888', labelsize=7)
            for spine in ax.spines.values():
                spine.set_color('#333333')
        self.ax_radar.tick_params(colors='#888888', labelsize=6)

    def render(self, weight_dict: dict, interrelation_matrix: np.ndarray,
               user_input: dict | None = None,
               analysis: dict | None = None,
               energy_state: dict | None = None,
               env_status: str | None = None,
               iteration: int = 0):
        """Render one frame of the dashboard."""
        import matplotlib.pyplot as plt

        weights = [weight_dict[a] for a in self.aspects]
        self.weight_history.append(weights)

        # Track input pulse
        if user_input and user_input.get('active'):
            self.pulse_remaining = self.PULSE_FRAMES
            self.pulse_kind = 'up' if user_input['direction'] == 'positive' else 'down'

        # Update PCA trail
        if len(self.weight_history) > 2:
            mat = np.array(self.weight_history)
            centered = mat - mat.mean(axis=0)
            cov = np.cov(centered.T) if centered.shape[0] > 2 else np.eye(self.num)
            try:
                eigvals, eigvecs = np.linalg.eigh(cov)
                pc1, pc2 = eigvecs[:, -1], eigvecs[:, -2]
            except Exception:
                pc1 = np.zeros(self.num); pc1[0] = 1
                pc2 = np.zeros(self.num); pc2[1] = 1
            snap = centered[-1]
            self.phase_trail_x.append(snap @ pc1)
            self.phase_trail_y.append(snap @ pc2)

        self._draw_radar(weights)
        self._draw_state(weights, analysis, energy_state, iteration)
        self._draw_heatmap(weights, interrelation_matrix)
        self._draw_timeseries()
        self._draw_phase()
        self._draw_pulse()

    def _draw_radar(self, weights):
        ax = self.ax_radar
        ax.clear()
        ax.set_facecolor('#0D1117')

        w = np.array(weights)
        wmin, wmax = w.min(), w.max()
        span = wmax - wmin if wmax != wmin else 1.0
        normed = (w - wmin) / span

        angles = np.linspace(0, 2 * np.pi, self.num, endpoint=False).tolist()
        normed_closed = np.append(normed, normed[0])
        angles_closed = angles + [angles[0]]

        ax.fill(angles_closed, normed_closed, alpha=0.18, color='#4A90D9')
        ax.plot(angles_closed, normed_closed, linewidth=1.4, color='#4A90D9', alpha=0.8)

        for i, (a, n) in enumerate(zip(angles, normed)):
            ax.plot(a, n, 'o', color=self.colors[i], markersize=5, zorder=5)

        short = [a.replace('_', '\n').replace('-', '\n') for a in self.aspects]
        ax.set_thetagrids(np.degrees(angles), short, fontsize=5, color='#AAAAAA')
        ax.set_ylim(0, 1.05)
        ax.set_rticks([0.25, 0.5, 0.75, 1.0])
        ax.set_yticklabels(['', '', '', ''], color='#444444')
        ax.set_title('Shape of Consciousness', fontsize=9, color='white', pad=14)
        ax.grid(color='#333333', linewidth=0.4)

    def _draw_state(self, weights, analysis, energy_state, iteration):
        ax = self.ax_state
        ax.clear()
        ax.set_facecolor('#0D1117')
        ax.set_xlim(0, 10)
        ax.set_ylim(0, 10)
        ax.axis('off')

        state, scolor = detect_mental_state(weights, list(self.weight_history))

        ax.text(5, 8.0, state.upper(), fontsize=22, fontweight='bold',
                ha='center', va='center', color=scolor)
        ax.text(5, 6.6, 'mental state', fontsize=8, ha='center', va='center',
                color='#666666')

        w = np.array(weights)
        abs_w = np.abs(w)
        total = np.sum(abs_w) + 1e-12
        probs = np.clip(abs_w / total, 1e-12, None)
        entropy = float(-np.sum(probs * np.log2(probs)))
        resilience = 1.0 / (np.std(w) + 0.01)

        metrics = [
            ('Entropy', f'{entropy:.2f}', '#4A90D9'),
            ('Resilience', f'{resilience:.2f}', '#3CB371'),
            ('Iteration', f'{iteration}', '#DAA520'),
        ]

        if energy_state:
            metrics.insert(2, ('Energy', f'{energy_state.get("energy_pct", 100):.0f}%', '#E8613C'))
            metrics.insert(3, ('Stress', f'{energy_state.get("stress", 0):.2f}', '#FF6B6B'))

        for i, (label, val, c) in enumerate(metrics[:5]):
            y = 4.8 - i * 1.0
            ax.text(2.0, y, label, fontsize=9, color='#888888', va='center')
            ax.text(8.0, y, val, fontsize=11, fontweight='bold', color=c,
                    ha='right', va='center')

        # Category legend
        ax.text(1.0, 0.5, 'Categories:', fontsize=7, color='#555555', va='center')
        x = 3.2
        for cat, c in self.category_colors.items():
            ax.plot(x, 0.5, 's', color=c, markersize=6)
            ax.text(x + 0.35, 0.5, cat[:4], fontsize=6, color=c, va='center')
            x += 1.5

    def _draw_heatmap(self, weights, interrelation_matrix):
        ax = self.ax_heat
        ax.clear()
        ax.set_facecolor('#0D1117')

        w = np.array(weights)
        coupling = np.abs(np.outer(w, w) * interrelation_matrix)
        np.fill_diagonal(coupling, 0)

        ax.imshow(coupling, cmap='inferno', aspect='auto', interpolation='nearest')
        ax.set_title('Dynamic Coupling', fontsize=9, color='white', pad=6)

        abbr = [a[:6] for a in self.aspects]
        ax.set_xticks(range(self.num))
        ax.set_xticklabels(abbr, fontsize=4, rotation=90, color='#AAAAAA')
        ax.set_yticks(range(self.num))
        ax.set_yticklabels(abbr, fontsize=4, color='#AAAAAA')

    def _draw_timeseries(self):
        ax = self.ax_ts
        ax.clear()
        ax.set_facecolor('#0D1117')

        if len(self.weight_history) < 2:
            return

        hist = np.array(self.weight_history)
        x = np.arange(len(hist))

        for i in range(self.num):
            ax.plot(x, hist[:, i], color=self.colors[i], linewidth=0.9, alpha=0.75)

        cur = hist[-1]
        for i in range(self.num):
            ax.annotate(f'{cur[i]:.2f}', xy=(len(hist) - 1, cur[i]),
                        fontsize=5, color=self.colors[i], va='center',
                        xytext=(4, 0), textcoords='offset points')

        ax.set_title('Weight Time Series', fontsize=9, color='white', pad=6)
        ax.set_xlabel('frame', fontsize=7, color='#888888')
        ax.set_ylabel('weight', fontsize=7, color='#888888')
        ax.grid(color='#222222', linewidth=0.3)

    def _draw_phase(self):
        ax = self.ax_phase
        ax.clear()
        ax.set_facecolor('#0D1117')

        if len(self.phase_trail_x) < 3:
            return

        xs = np.array(self.phase_trail_x)
        ys = np.array(self.phase_trail_y)

        n = len(xs)
        points = np.column_stack([xs, ys]).reshape(-1, 1, 2)
        segments = np.concatenate([points[:-1], points[1:]], axis=1)
        alphas = np.linspace(0.1, 1.0, n - 1)
        colors = np.zeros((n - 1, 4))
        colors[:, 0] = 0.29
        colors[:, 1] = 0.56
        colors[:, 2] = 0.85
        colors[:, 3] = alphas
        lc = LineCollection(segments, colors=colors,
                            linewidths=np.linspace(0.5, 2.0, n - 1))
        ax.add_collection(lc)
        ax.plot(xs[-1], ys[-1], 'o', color='#4A90D9', markersize=6, zorder=5)

        pad = 0.3
        ax.set_xlim(xs.min() - pad, xs.max() + pad)
        ax.set_ylim(ys.min() - pad, ys.max() + pad)
        ax.set_title('Phase Space (PCA)', fontsize=9, color='white', pad=6)
        ax.set_xlabel('PC1', fontsize=7, color='#888888')
        ax.set_ylabel('PC2', fontsize=7, color='#888888')
        ax.grid(color='#222222', linewidth=0.3)

    def _draw_pulse(self):
        import matplotlib.pyplot as plt

        ax = self.ax_pulse
        ax.clear()
        ax.set_facecolor('#0D1117')
        ax.set_xlim(0, self.num)
        ax.set_ylim(0, 1)
        ax.axis('off')

        if self.pulse_remaining > 0:
            self.pulse_remaining -= 1
            t = 1.0 - self.pulse_remaining / self.PULSE_FRAMES
            base_color = '#3CB371' if self.pulse_kind == 'up' else '#E8613C'

            cx = self.num / 2
            for ring in range(3):
                r = 0.15 + t * (0.4 + ring * 0.25)
                alpha = max(0, 0.6 - t * 0.6 - ring * 0.15)
                circle = plt.Circle((cx, 0.5), r, fill=False, edgecolor=base_color,
                                    linewidth=2.5 - ring * 0.7, alpha=alpha)
                ax.add_patch(circle)

            direction = 'POSITIVE' if self.pulse_kind == 'up' else 'NEGATIVE'
            ax.text(self.num / 2, 0.5, f'{direction} INPUT',
                    fontsize=11, fontweight='bold', ha='center', va='center',
                    color=base_color, alpha=max(0, 1.0 - t * 0.8))
