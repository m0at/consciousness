"""Interrelation matrix: manages and applies inter-aspect influence propagation."""

import numpy as np


class InterrelationMatrix:
    """Aspect coupling matrix supporting symmetric pairs, asymmetric weighted
    edges, and inhibitory connections."""

    def __init__(self):
        self._matrix: np.ndarray | None = None
        self._aspects: list[str] = []

    def build(self, aspects: list[str], relationships: list, strength: float = 0.005):
        """Construct the matrix from relationship definitions.

        Relationships can be:
          - (a, b)       — symmetric pair at `strength`
          - (a, b, w)    — directed edge a→b with weight w
        """
        n = len(aspects)
        self._aspects = list(aspects)
        self._matrix = np.identity(n)
        idx = {a: i for i, a in enumerate(aspects)}

        for rel in relationships:
            if len(rel) == 2:
                a1, a2 = rel
                w = strength
                if a1 in idx and a2 in idx:
                    self._matrix[idx[a1], idx[a2]] = w
                    self._matrix[idx[a2], idx[a1]] = w
            elif len(rel) == 3:
                a1, a2, w = rel
                if a1 in idx and a2 in idx:
                    self._matrix[idx[a1], idx[a2]] = w

    def propagate(self, weights: list[float]) -> list[float]:
        """Apply the interrelation matrix to a weight vector."""
        if self._matrix is None:
            return list(weights)
        w = np.array(weights)
        return (self._matrix @ w).tolist()

    @property
    def matrix(self) -> np.ndarray:
        return self._matrix if self._matrix is not None else np.array([])
