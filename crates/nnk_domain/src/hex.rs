use serde::{Deserialize, Serialize};

/// Axial hex coordinates (Red Blob Games). Presentation maps this to `hexx` later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub const ZERO: Self = Self { q: 0, r: 0 };

    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn distance(self, other: Self) -> u32 {
        let dq = self.q - other.q;
        let dr = self.r - other.r;
        let ds = (-self.q - self.r) - (-other.q - other.r);
        ((dq.abs() + dr.abs() + ds.abs()) / 2) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_distance() {
        assert_eq!(HexCoord::ZERO.distance(HexCoord::ZERO), 0);
    }

    #[test]
    fn neighbors_distance_one() {
        assert_eq!(HexCoord::new(1, 0).distance(HexCoord::ZERO), 1);
        assert_eq!(HexCoord::new(0, -1).distance(HexCoord::ZERO), 1);
        assert_eq!(HexCoord::new(-1, 1).distance(HexCoord::ZERO), 1);
    }

    #[test]
    fn longer_path() {
        assert_eq!(HexCoord::new(2, -1).distance(HexCoord::ZERO), 2);
        assert_eq!(HexCoord::new(3, -3).distance(HexCoord::ZERO), 3);
    }
}
