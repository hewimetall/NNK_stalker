use serde::{Deserialize, Serialize};

use crate::DomainError;

/// Locations from form.md.md (D20; 19–20 = reroll).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum LocationId {
    Cordon = 1,
    Garbage = 2,
    Swamps = 3,
    Agroprom = 4,
    AgropromUnderground = 5,
    DarkValley = 6,
    LabX18 = 7,
    BarRostok = 8,
    WildTerritory = 9,
    Yantar = 10,
    LabX16 = 11,
    MilitaryWarehouses = 12,
    Radar = 13,
    Pripyat = 14,
    Zaton = 15,
    Jupiter = 16,
    EastPripyat = 17,
    Cnpp = 18,
}

impl LocationId {
    pub fn from_d20(roll: u8) -> Result<Option<Self>, DomainError> {
        match roll {
            0 | 21..=u8::MAX => Err(DomainError::InvalidDice(roll)),
            19 | 20 => Ok(None),
            1 => Ok(Some(Self::Cordon)),
            2 => Ok(Some(Self::Garbage)),
            3 => Ok(Some(Self::Swamps)),
            4 => Ok(Some(Self::Agroprom)),
            5 => Ok(Some(Self::AgropromUnderground)),
            6 => Ok(Some(Self::DarkValley)),
            7 => Ok(Some(Self::LabX18)),
            8 => Ok(Some(Self::BarRostok)),
            9 => Ok(Some(Self::WildTerritory)),
            10 => Ok(Some(Self::Yantar)),
            11 => Ok(Some(Self::LabX16)),
            12 => Ok(Some(Self::MilitaryWarehouses)),
            13 => Ok(Some(Self::Radar)),
            14 => Ok(Some(Self::Pripyat)),
            15 => Ok(Some(Self::Zaton)),
            16 => Ok(Some(Self::Jupiter)),
            17 => Ok(Some(Self::EastPripyat)),
            18 => Ok(Some(Self::Cnpp)),
        }
    }

    pub fn name_ru(self) -> &'static str {
        match self {
            Self::Cordon => "Кордон",
            Self::Garbage => "Свалка",
            Self::Swamps => "Болота",
            Self::Agroprom => "НИИ Агропром",
            Self::AgropromUnderground => "Подземелье Агропрома",
            Self::DarkValley => "Темная долина",
            Self::LabX18 => "Лаборатория Х-18",
            Self::BarRostok => "Бар (Завод Росток)",
            Self::WildTerritory => "Дикая территория",
            Self::Yantar => "Янтарь",
            Self::LabX16 => "Лаб. Х-16",
            Self::MilitaryWarehouses => "Военные склады",
            Self::Radar => "Радар",
            Self::Pripyat => "Припять",
            Self::Zaton => "Затон",
            Self::Jupiter => "Юпитер",
            Self::EastPripyat => "Восточная Припять",
            Self::Cnpp => "ЧАЭС",
        }
    }
}

/// Coordination cards → sector 1..=100 (form.md.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CccDraw {
    pub tens: u8,
    pub units: u8,
    pub sector: u8,
}

impl CccDraw {
    pub fn combine(tens: u8, units: u8) -> Result<Self, DomainError> {
        if tens > 90 || tens % 10 != 0 || units > 9 {
            return Err(DomainError::InvalidCcc);
        }
        let sector = if tens == 0 && units == 0 {
            100
        } else {
            tens.saturating_add(units)
        };
        Ok(Self {
            tens,
            units,
            sector,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ccc_form_examples() {
        assert_eq!(CccDraw::combine(30, 6).unwrap().sector, 36);
        assert_eq!(CccDraw::combine(0, 2).unwrap().sector, 2);
        assert_eq!(CccDraw::combine(0, 0).unwrap().sector, 100);
    }

    #[test]
    fn ccc_rejects_bad() {
        assert!(CccDraw::combine(15, 1).is_err());
        assert!(CccDraw::combine(10, 10).is_err());
    }

    #[test]
    fn d20_mapping() {
        assert_eq!(LocationId::from_d20(1).unwrap(), Some(LocationId::Cordon));
        assert_eq!(LocationId::from_d20(19).unwrap(), None);
        assert_eq!(LocationId::from_d20(20).unwrap(), None);
        assert!(LocationId::from_d20(0).is_err());
        assert!(LocationId::from_d20(21).is_err());
    }

    #[test]
    fn all_named() {
        for roll in 1..=18u8 {
            let loc = LocationId::from_d20(roll).unwrap().unwrap();
            assert!(!loc.name_ru().is_empty());
        }
    }
}
