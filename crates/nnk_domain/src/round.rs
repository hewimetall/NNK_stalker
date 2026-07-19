//! Round phases from Пилигрим v3.0 general rules («Ход игры»).

use serde::{Deserialize, Serialize};

/// In-match round phase while `RoomPhase::Playing`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RoundPhase {
    /// Phase I — «Выход в Зону»: roll D20 for event-token count.
    #[default]
    ExitZone,
    /// Phase II — «Розыгрыш Карт Событий»: D6 move + draw events.
    Explore,
    /// Phase III — «Возвращение на базу»: trade/repair abstract, then next round.
    ReturnBase,
}

impl RoundPhase {
    pub fn wire_id(self) -> &'static str {
        match self {
            Self::ExitZone => "exit_zone",
            Self::Explore => "explore",
            Self::ReturnBase => "return_base",
        }
    }

    pub fn label_ru(self) -> &'static str {
        match self {
            Self::ExitZone => "Выход в Зону",
            Self::Explore => "Розыгрыш событий",
            Self::ReturnBase => "База",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_ids_stable() {
        assert_eq!(RoundPhase::ExitZone.wire_id(), "exit_zone");
        assert_eq!(RoundPhase::Explore.label_ru(), "Розыгрыш событий");
    }
}
