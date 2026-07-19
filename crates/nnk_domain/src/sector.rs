use crate::HexCoord;

/// 19 hexes of one sector (radius-2 hex flower), index 0..18.
/// Order: center, then rings — used after D20 hex roll (1..=19).
pub fn sector_hexes() -> [HexCoord; 19] {
    [
        HexCoord::new(0, 0),
        HexCoord::new(1, 0),
        HexCoord::new(1, -1),
        HexCoord::new(0, -1),
        HexCoord::new(-1, 0),
        HexCoord::new(-1, 1),
        HexCoord::new(0, 1),
        HexCoord::new(2, 0),
        HexCoord::new(2, -1),
        HexCoord::new(2, -2),
        HexCoord::new(1, -2),
        HexCoord::new(0, -2),
        HexCoord::new(-1, -1),
        HexCoord::new(-2, 0),
        HexCoord::new(-2, 1),
        HexCoord::new(-2, 2),
        HexCoord::new(-1, 2),
        HexCoord::new(0, 2),
        HexCoord::new(1, 1),
    ]
}

/// Map D20 (1..=20) to sector hex. 20 wraps to index 18 (19th hex).
pub fn hex_from_d20(roll: u8) -> HexCoord {
    let idx = match roll {
        1..=19 => (roll - 1) as usize,
        _ => 18,
    };
    sector_hexes()[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nineteen_unique() {
        let h = sector_hexes();
        let mut v = h.to_vec();
        v.sort_by_key(|c| (c.q, c.r));
        v.dedup();
        assert_eq!(v.len(), 19);
    }

    #[test]
    fn d20_maps() {
        assert_eq!(hex_from_d20(1), HexCoord::ZERO);
        assert_eq!(hex_from_d20(20), sector_hexes()[18]);
    }
}
