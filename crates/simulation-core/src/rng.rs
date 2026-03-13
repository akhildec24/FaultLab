//! Deterministic pseudo-random number generator.
//!
//! Uses splitmix64 — a simple, fast, well-distributed PRNG with a single
//! 64-bit state. Given the same seed, it always produces the same sequence,
//! which is essential for deterministic replay and comparison mode.
//!
//! Reference: https://prng.di.unimi.it/splitmix64.c

/// A deterministic PRNG based on splitmix64.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Create a new RNG with the given seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Generate the next random u64.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Generate a random float in [0.0, 1.0).
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Generate a random integer in [0, bound).
    pub fn below(&mut self, bound: u64) -> u64 {
        if bound == 0 {
            return 0;
        }
        self.next_u64() % bound
    }

    /// Generate a random float in [min, max).
    pub fn range_f64(&mut self, min: f64, max: f64) -> f64 {
        min + self.next_f64() * (max - min)
    }

    /// Generate a random integer in [min, max].
    pub fn range_u64(&mut self, min: u64, max: u64) -> u64 {
        if min >= max {
            return min;
        }
        min + self.below(max - min + 1)
    }

    /// Return true with probability `p` (0.0 = never, 1.0 = always).
    pub fn chance(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }

    /// Pick a random element from a slice.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            None
        } else {
            Some(&items[self.below(items.len() as u64) as usize])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_produce_different_sequences() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(43);
        let mut diffs = 0;
        for _ in 0..100 {
            if a.next_u64() != b.next_u64() {
                diffs += 1;
            }
        }
        assert!(
            diffs > 90,
            "expected mostly different values, got {diffs} diffs"
        );
    }

    #[test]
    fn next_f64_in_range() {
        let mut rng = Rng::new(42);
        for _ in 0..1000 {
            let v = rng.next_f64();
            assert!(v >= 0.0 && v < 1.0, "value {v} out of range");
        }
    }

    #[test]
    fn below_never_exceeds_bound() {
        let mut rng = Rng::new(42);
        for _ in 0..1000 {
            let v = rng.below(10);
            assert!(v < 10, "value {v} >= bound 10");
        }
    }

    #[test]
    fn range_f64_within_bounds() {
        let mut rng = Rng::new(42);
        for _ in 0..1000 {
            let v = rng.range_f64(5.0, 15.0);
            assert!(v >= 5.0 && v < 15.0, "value {v} out of range");
        }
    }

    #[test]
    fn chance_zero_always_false() {
        let mut rng = Rng::new(42);
        for _ in 0..100 {
            assert!(!rng.chance(0.0));
        }
    }

    #[test]
    fn chance_one_always_true() {
        let mut rng = Rng::new(42);
        for _ in 0..100 {
            assert!(rng.chance(1.0));
        }
    }

    #[test]
    fn pick_from_empty_returns_none() {
        let mut rng = Rng::new(42);
        let items: Vec<u32> = vec![];
        assert!(rng.pick(&items).is_none());
    }

    #[test]
    fn pick_returns_valid_element() {
        let mut rng = Rng::new(42);
        let items = vec![10, 20, 30, 40, 50];
        let picked = rng.pick(&items).unwrap();
        assert!(items.contains(picked));
    }
}
