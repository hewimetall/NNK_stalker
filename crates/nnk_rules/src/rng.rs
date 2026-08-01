use rand::SeedableRng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

/// Deterministic RNG wrapper (engine contract: seed in state, no side-effect RNG).
#[derive(Debug, Clone)]
pub struct GameRng {
    inner: ChaCha8Rng,
    seed: u64,
}

impl GameRng {
    pub fn from_seed(seed: u64) -> Self {
        Self {
            inner: ChaCha8Rng::seed_from_u64(seed),
            seed,
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn d6(&mut self) -> u8 {
        self.inner.gen_range(1..=6)
    }

    pub fn d20(&mut self) -> u8 {
        self.inner.gen_range(1..=20)
    }

    /// Tens deck {0,10,...,90} + units {0..9}.
    pub fn ccc_parts(&mut self) -> (u8, u8) {
        let tens = self.inner.gen_range(0..=9) * 10;
        let units = self.inner.gen_range(0..=9);
        (tens, units)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_sequence() {
        let mut a = GameRng::from_seed(42);
        let mut b = GameRng::from_seed(42);
        let seq_a: Vec<_> = (0..20).map(|_| a.d20()).collect();
        let seq_b: Vec<_> = (0..20).map(|_| b.d20()).collect();
        assert_eq!(seq_a, seq_b);
    }

    #[test]
    fn dice_ranges() {
        let mut r = GameRng::from_seed(1);
        for _ in 0..100 {
            let d = r.d6();
            assert!((1..=6).contains(&d));
            let d20 = r.d20();
            assert!((1..=20).contains(&d20));
            let (t, u) = r.ccc_parts();
            assert_eq!(t % 10, 0);
            assert!(t <= 90);
            assert!(u <= 9);
        }
    }
}
