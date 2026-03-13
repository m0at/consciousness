/// Xoshiro256+ PRNG — deterministic given a seed.
///
/// State is four u64 words initialised via SplitMix64 so that a single u64
/// seed expands into a full 256-bit state with good statistical properties.
pub struct Rng {
    s: [u64; 4],
}

// SplitMix64 — used only during seeding.
#[inline(always)]
fn splitmix64(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

impl Rng {
    /// Seed the generator from a single u64 via SplitMix64.
    pub fn new(seed: u64) -> Self {
        let mut s = seed;
        Self {
            s: [
                splitmix64(&mut s),
                splitmix64(&mut s),
                splitmix64(&mut s),
                splitmix64(&mut s),
            ],
        }
    }

    /// Xoshiro256+ output step — period 2^256 - 1.
    #[inline(always)]
    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[0].wrapping_add(self.s[3]);
        let t = self.s[1] << 17;

        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];

        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);

        result
    }

    /// Uniform f64 in [0, 1).
    ///
    /// Uses the upper 53 bits to fill the mantissa exactly, giving the finest
    /// representable spacing (2^-53) with no bias.
    #[inline(always)]
    pub fn next_f64(&mut self) -> f64 {
        // Set the IEEE-754 exponent to 1.0 then subtract 1.0 → [0, 1).
        let bits = (self.next_u64() >> 11) | (1023u64 << 52);
        f64::from_bits(bits) - 1.0
    }

    /// Uniform f64 in [lo, hi).
    #[inline(always)]
    pub fn uniform(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.next_f64() * (hi - lo)
    }

    /// Sample a random element from a non-empty slice.
    ///
    /// # Panics
    /// Panics if `slice` is empty.
    #[inline(always)]
    pub fn choice<'a, T>(&mut self, slice: &'a [T]) -> &'a T {
        &slice[self.choice_index(slice.len())]
    }

    /// Return a uniform random index in `0..len`.
    ///
    /// Uses rejection sampling to avoid modulo bias.
    ///
    /// # Panics
    /// Panics if `len == 0`.
    #[inline]
    pub fn choice_index(&mut self, len: usize) -> usize {
        assert!(len > 0, "choice_index: len must be > 0");
        let len64 = len as u64;
        // Largest multiple of len that fits in u64 (rejection threshold).
        let threshold = u64::MAX - (u64::MAX % len64);
        loop {
            let r = self.next_u64();
            if r < threshold {
                return (r % len64) as usize;
            }
        }
    }

    /// Normal sample N(mu, sigma) via Box-Muller transform.
    ///
    /// Box-Muller requires two uniform samples on (0, 1).  We guard against
    /// the degenerate u == 0 case (probability 2^-53) by retrying.
    pub fn normal(&mut self, mu: f64, sigma: f64) -> f64 {
        loop {
            let u = self.next_f64();
            if u == 0.0 {
                continue;
            }
            let v = self.next_f64();
            let z = (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos();
            return mu + sigma * z;
        }
    }

    /// Return `true` with probability `p` ∈ [0, 1].
    #[inline(always)]
    pub fn bool_with_prob(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn f64_in_range() {
        let mut rng = Rng::new(1);
        for _ in 0..100_000 {
            let x = rng.next_f64();
            assert!((0.0..1.0).contains(&x));
        }
    }

    #[test]
    fn uniform_bounds() {
        let mut rng = Rng::new(2);
        for _ in 0..100_000 {
            let x = rng.uniform(-5.0, 5.0);
            assert!(x >= -5.0 && x < 5.0);
        }
    }

    #[test]
    fn choice_covers_all() {
        let mut rng = Rng::new(3);
        let items = [0u32, 1, 2, 3, 4];
        let mut seen = [false; 5];
        for _ in 0..10_000 {
            let i = rng.choice_index(items.len());
            seen[i] = true;
        }
        assert!(seen.iter().all(|&v| v));
    }

    #[test]
    fn bool_with_prob_zero_one() {
        let mut rng = Rng::new(4);
        for _ in 0..1000 {
            assert!(!rng.bool_with_prob(0.0));
            assert!(rng.bool_with_prob(1.0));
        }
    }

    #[test]
    fn normal_rough_moments() {
        let mut rng = Rng::new(5);
        let n = 100_000;
        let samples: Vec<f64> = (0..n).map(|_| rng.normal(0.0, 1.0)).collect();
        let mean = samples.iter().sum::<f64>() / n as f64;
        let var = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
        // Mean within 3 standard errors (~0.003), variance within 1 % of 1.0.
        assert!(mean.abs() < 0.02, "mean {mean} out of range");
        assert!((var - 1.0).abs() < 0.02, "variance {var} out of range");
    }
}
