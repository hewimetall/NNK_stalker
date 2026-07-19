use crate::HexCoord;

/// 19 hexes of one sector (radius-2 hex flower), index 0..18.
/// Order: left-to-right, top-to-bottom visual row-major — used after D20
/// hex roll (1..=19).
pub fn sector_hexes() -> [HexCoord; 19] {
    [
        HexCoord::new(0, -2),
        HexCoord::new(1, -2),
        HexCoord::new(2, -2),
        HexCoord::new(-1, -1),
        HexCoord::new(0, -1),
        HexCoord::new(1, -1),
        HexCoord::new(2, -1),
        HexCoord::new(-2, 0),
        HexCoord::new(-1, 0),
        HexCoord::new(0, 0),
        HexCoord::new(1, 0),
        HexCoord::new(2, 0),
        HexCoord::new(-2, 1),
        HexCoord::new(-1, 1),
        HexCoord::new(0, 1),
        HexCoord::new(1, 1),
        HexCoord::new(-2, 2),
        HexCoord::new(-1, 2),
        HexCoord::new(0, 2),
    ]
}

/// Map D20 to sector hex. 20 means reroll and never wraps to hex 19.
pub fn hex_from_d20(roll: u8) -> Result<Option<HexCoord>, crate::DomainError> {
    match roll {
        1..=19 => Ok(Some(sector_hexes()[(roll - 1) as usize])),
        20 => Ok(None),
        _ => Err(crate::DomainError::InvalidDice(roll)),
    }
}

pub fn is_sector_hex(hex: HexCoord) -> bool {
    sector_hexes().contains(&hex)
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
        assert_eq!(hex_from_d20(1).unwrap(), Some(HexCoord::new(0, -2)));
        assert_eq!(hex_from_d20(10).unwrap(), Some(HexCoord::ZERO));
        assert_eq!(hex_from_d20(19).unwrap(), Some(HexCoord::new(0, 2)));
        assert_eq!(hex_from_d20(20).unwrap(), None);
        assert!(hex_from_d20(0).is_err());
    }

    #[test]
    fn row_major_order() {
        assert_eq!(
            sector_hexes(),
            [
                HexCoord::new(0, -2),
                HexCoord::new(1, -2),
                HexCoord::new(2, -2),
                HexCoord::new(-1, -1),
                HexCoord::new(0, -1),
                HexCoord::new(1, -1),
                HexCoord::new(2, -1),
                HexCoord::new(-2, 0),
                HexCoord::new(-1, 0),
                HexCoord::new(0, 0),
                HexCoord::new(1, 0),
                HexCoord::new(2, 0),
                HexCoord::new(-2, 1),
                HexCoord::new(-1, 1),
                HexCoord::new(0, 1),
                HexCoord::new(1, 1),
                HexCoord::new(-2, 2),
                HexCoord::new(-1, 2),
                HexCoord::new(0, 2),
            ]
        );
    }
}
