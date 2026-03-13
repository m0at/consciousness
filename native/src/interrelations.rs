//! Interrelation matrix: manages and applies inter-aspect influence propagation.
//!
//! Mirrors the Python `InterrelationMatrix` in core/interrelations.py exactly.
//! Triples `(src, dst, weight)` set directed edges; the matrix is initialised
//! as the identity so unspecified pairs leave weights unchanged.

pub struct InterrelationMatrix {
    data: Vec<f64>, // row-major n×n
    n: usize,
}

impl InterrelationMatrix {
    /// Create an n×n identity matrix (no coupling yet).
    pub fn new(n: usize) -> Self {
        let mut data = vec![0.0_f64; n * n];
        for i in 0..n {
            data[i * n + i] = 1.0;
        }
        Self { data, n }
    }

    /// Rebuild from (src, dst, weight) triples.
    ///
    /// Resets to identity first, then sets `matrix[src][dst] = weight` for
    /// each triple.  Symmetric pairs must be passed as two separate triples
    /// (the Python layer handles that before calling here).
    pub fn build(&mut self, triples: &[(usize, usize, f64)]) {
        // Reset to identity.
        self.data.iter_mut().for_each(|x| *x = 0.0);
        for i in 0..self.n {
            self.data[i * self.n + i] = 1.0;
        }
        // Apply edges.
        for &(src, dst, w) in triples {
            self.data[src * self.n + dst] = w;
        }
    }

    /// Matrix-vector multiply: `out = self.data @ weights`.
    ///
    /// `out[i] = Σ_j data[i*n + j] * weights[j]`
    ///
    /// Manual loop — for the 20-32 aspect sizes used here the BLAS call
    /// overhead exceeds the compute cost.
    pub fn propagate(&self, weights: &[f64], out: &mut [f64]) {
        let n = self.n;
        for (i, slot) in out.iter_mut().enumerate().take(n) {
            let row = &self.data[i * n..(i + 1) * n];
            let mut acc = 0.0_f64;
            for j in 0..n {
                acc += row[j] * weights[j];
            }
            *slot = acc;
        }
    }

    pub fn matrix(&self) -> &[f64] {
        &self.data
    }

    pub fn n(&self) -> usize {
        self.n
    }
}
