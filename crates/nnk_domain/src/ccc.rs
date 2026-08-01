use serde::{Deserialize, Serialize};

use crate::{CccDraw, DomainError};

const TENS_CARDS: [u8; 10] = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90];
const UNITS_CARDS: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CccDeck {
    pub tens: Vec<u8>,
    pub units: Vec<u8>,
    pub tens_discard: Vec<u8>,
    pub units_discard: Vec<u8>,
    #[serde(default)]
    shuffle_seed: u64,
}

impl CccDeck {
    pub fn shuffled(seed: u64) -> Self {
        let mut deck = Self {
            tens: TENS_CARDS.to_vec(),
            units: UNITS_CARDS.to_vec(),
            tens_discard: Vec::new(),
            units_discard: Vec::new(),
            shuffle_seed: seed,
        };
        deck.shuffle_tens();
        deck.shuffle_units();
        deck
    }

    pub fn draw_tens(&mut self) -> u8 {
        if self.tens.is_empty() {
            self.reshuffle_tens_from_discard();
        }
        let card = self.tens.pop().unwrap_or_else(|| {
            self.tens = TENS_CARDS.to_vec();
            self.shuffle_tens();
            self.tens.pop().expect("tens deck reset must contain cards")
        });
        self.tens_discard.push(card);
        card
    }

    pub fn draw_units(&mut self) -> u8 {
        if self.units.is_empty() {
            self.reshuffle_units_from_discard();
        }
        let card = self.units.pop().unwrap_or_else(|| {
            self.units = UNITS_CARDS.to_vec();
            self.shuffle_units();
            self.units
                .pop()
                .expect("units deck reset must contain cards")
        });
        self.units_discard.push(card);
        card
    }

    pub fn draw_ccc(&mut self) -> Result<CccDraw, DomainError> {
        let tens = self.draw_tens();
        let units = self.draw_units();
        CccDraw::combine(tens, units)
    }

    fn reshuffle_tens_from_discard(&mut self) {
        if self.tens_discard.is_empty() {
            return;
        }
        self.tens.append(&mut self.tens_discard);
        self.shuffle_tens();
    }

    fn reshuffle_units_from_discard(&mut self) {
        if self.units_discard.is_empty() {
            return;
        }
        self.units.append(&mut self.units_discard);
        self.shuffle_units();
    }

    fn shuffle_tens(&mut self) {
        let seed = self.next_shuffle_seed(0x4343_435f_5445_4e53);
        shuffle(&mut self.tens, seed);
    }

    fn shuffle_units(&mut self) {
        let seed = self.next_shuffle_seed(0x4343_435f_554e_4954);
        shuffle(&mut self.units, seed);
    }

    fn next_shuffle_seed(&mut self, salt: u64) -> u64 {
        self.shuffle_seed = splitmix64(self.shuffle_seed ^ salt);
        self.shuffle_seed
    }
}

impl Default for CccDeck {
    fn default() -> Self {
        Self::shuffled(0)
    }
}

fn shuffle(cards: &mut [u8], seed: u64) {
    let mut state = seed;
    for i in (1..cards.len()).rev() {
        state = splitmix64(state);
        let j = (state as usize) % (i + 1);
        cards.swap(i, j);
    }
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = value;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_contains_real_ccc_cards() {
        let deck = CccDeck::shuffled(42);
        let mut tens = deck.tens.clone();
        let mut units = deck.units.clone();
        tens.sort_unstable();
        units.sort_unstable();
        assert_eq!(tens, TENS_CARDS);
        assert_eq!(units, UNITS_CARDS);
    }

    #[test]
    fn draws_consume_and_discard_cards() {
        let mut deck = CccDeck::shuffled(1);
        let tens_len = deck.tens.len();
        let units_len = deck.units.len();
        let draw = deck.draw_ccc().unwrap();
        assert_eq!(deck.tens.len(), tens_len - 1);
        assert_eq!(deck.units.len(), units_len - 1);
        assert_eq!(deck.tens_discard, vec![draw.tens]);
        assert_eq!(deck.units_discard, vec![draw.units]);
        assert!((1..=100).contains(&draw.sector));
    }

    #[test]
    fn reshuffles_discard_when_empty() {
        let mut deck = CccDeck::shuffled(5);
        for _ in 0..10 {
            let _ = deck.draw_ccc().unwrap();
        }
        assert!(deck.tens.is_empty());
        assert!(deck.units.is_empty());
        assert_eq!(deck.tens_discard.len(), 10);
        assert_eq!(deck.units_discard.len(), 10);

        let _ = deck.draw_ccc().unwrap();

        assert_eq!(deck.tens.len(), 9);
        assert_eq!(deck.units.len(), 9);
        assert_eq!(deck.tens_discard.len(), 1);
        assert_eq!(deck.units_discard.len(), 1);
    }
}
