use serde::{Deserialize, Serialize};

/// In-table campaign time. Term dump says one human play hour equals one game day;
/// until wall-clock play tracking is wired, each completed travel resolution advances
/// the Zone clock by two game hours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameClock {
    pub game_day: u32,
    pub game_hour: u8,
    pub play_started_unix: Option<i64>,
}

impl GameClock {
    pub const START: Self = Self {
        game_day: 1,
        game_hour: 0,
        play_started_unix: None,
    };

    pub fn advance_hours(&mut self, hours: u8) {
        let total = u32::from(self.game_hour) + u32::from(hours);
        self.game_day += total / 24;
        self.game_hour = (total % 24) as u8;
    }

    pub fn advance_after_travel_resolution(&mut self) {
        self.advance_hours(2);
    }
}

impl Default for GameClock {
    fn default() -> Self {
        Self::START
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_day_one_midnight() {
        let clock = GameClock::default();
        assert_eq!(clock.game_day, 1);
        assert_eq!(clock.game_hour, 0);
        assert_eq!(clock.play_started_unix, None);
    }

    #[test]
    fn advances_across_midnight() {
        let mut clock = GameClock {
            game_day: 2,
            game_hour: 23,
            play_started_unix: Some(10),
        };
        clock.advance_after_travel_resolution();
        assert_eq!(clock.game_day, 3);
        assert_eq!(clock.game_hour, 1);
        assert_eq!(clock.play_started_unix, Some(10));
    }
}
