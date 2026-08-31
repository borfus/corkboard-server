//! A tiny deterministic PRNG.
//!
//! Battles must be reproducible from their stored seed, so the simulator can
//! never reach for thread-local randomness. SplitMix64 is used here because it
//! is a dozen lines, has no dependencies, and is more than good enough to decide
//! whether a Pidgey crit.

pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `0..n`. Returns 0 when `n` is 0 rather than dividing by zero.
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        self.next_u64() % n
    }

    /// Uniform in `[0, 1)`, taking the top 53 bits so the mantissa is filled.
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in `[lo, hi)`.
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.unit() * (hi - lo)
    }

    /// True with probability `p`.
    pub fn chance(&mut self, p: f64) -> bool {
        self.unit() < p
    }

    /// In-place Fisher-Yates.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        if slice.len() < 2 {
            return;
        }
        for i in (1..slice.len()).rev() {
            let j = self.below((i + 1) as u64) as usize;
            slice.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_deterministic() {
        let a: Vec<u64> = (0..8).map(|_| Rng::new(42).next_u64()).collect();
        assert!(a.iter().all(|v| *v == a[0]), "same seed, same first draw");

        let mut r1 = Rng::new(7);
        let mut r2 = Rng::new(7);
        for _ in 0..64 {
            assert_eq!(r1.next_u64(), r2.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut r1 = Rng::new(1);
        let mut r2 = Rng::new(2);
        assert_ne!(r1.next_u64(), r2.next_u64());
    }

    #[test]
    fn unit_stays_in_range() {
        let mut r = Rng::new(99);
        for _ in 0..10_000 {
            let u = r.unit();
            assert!(u >= 0.0 && u < 1.0, "unit() out of range: {}", u);
        }
    }

    #[test]
    fn below_stays_in_range() {
        let mut r = Rng::new(5);
        for _ in 0..10_000 {
            assert!(r.below(18) < 18);
        }
        assert_eq!(r.below(0), 0);
    }

    #[test]
    fn shuffle_is_a_permutation() {
        let mut r = Rng::new(2024);
        let mut v: Vec<usize> = (0..18).collect();
        r.shuffle(&mut v);
        v.sort();
        assert_eq!(v, (0..18).collect::<Vec<usize>>());
    }
}
