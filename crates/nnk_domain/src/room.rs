use serde::{Deserialize, Serialize};

use crate::{
    CccDeck, GameClock, Npc, PlayerToken, RoomId, RoomPhase, UserId, LOBBY_MAX_PLAYERS,
    LOBBY_MIN_PLAYERS,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomState {
    pub room_id: RoomId,
    pub code: String,
    pub gm_user_id: UserId,
    pub phase: RoomPhase,
    pub mission_id: u8,
    pub tokens: Vec<PlayerToken>,
    pub active_user_id: Option<UserId>,
    pub history: Vec<String>,
    /// Deterministic RNG seed for replay (turn-based engine contract).
    pub rng_seed: u64,
    pub min_players: u8,
    pub max_players: u8,
    #[serde(default = "default_game_day")]
    pub game_day: u32,
    #[serde(default)]
    pub game_hour: u8,
    #[serde(default)]
    pub play_started_unix: Option<i64>,
    #[serde(default)]
    pub ccc_deck: CccDeck,
    #[serde(default)]
    pub npcs: Vec<Npc>,
}

impl RoomState {
    pub fn new(room_id: RoomId, code: String, gm: PlayerToken, rng_seed: u64) -> Self {
        let gm_user_id = gm.user_id;
        Self {
            room_id,
            code,
            gm_user_id,
            phase: RoomPhase::Lobby,
            mission_id: 1,
            tokens: vec![gm],
            active_user_id: Some(gm_user_id),
            history: Vec::new(),
            rng_seed,
            min_players: LOBBY_MIN_PLAYERS,
            max_players: LOBBY_MAX_PLAYERS,
            game_day: GameClock::START.game_day,
            game_hour: GameClock::START.game_hour,
            play_started_unix: GameClock::START.play_started_unix,
            ccc_deck: CccDeck::shuffled(rng_seed),
            npcs: Vec::new(),
        }
    }

    pub fn player_count(&self) -> u8 {
        self.tokens.len() as u8
    }

    pub fn is_full(&self) -> bool {
        self.player_count() >= self.max_players
    }

    pub fn all_ready(&self) -> bool {
        !self.tokens.is_empty() && self.tokens.iter().all(|t| t.ready)
    }

    pub fn can_start(&self) -> bool {
        self.phase == RoomPhase::Lobby
            && self.player_count() >= self.min_players
            && self.player_count() <= self.max_players
            && self.all_ready()
    }

    pub fn find_token(&self, user_id: UserId) -> Option<&PlayerToken> {
        self.tokens.iter().find(|t| t.user_id == user_id)
    }

    pub fn find_token_mut(&mut self, user_id: UserId) -> Option<&mut PlayerToken> {
        self.tokens.iter_mut().find(|t| t.user_id == user_id)
    }

    pub fn game_clock(&self) -> GameClock {
        GameClock {
            game_day: self.game_day,
            game_hour: self.game_hour,
            play_started_unix: self.play_started_unix,
        }
    }

    /// Until real wall-clock mapping is wired, one completed travel resolution
    /// advances campaign time by two game hours.
    pub fn advance_clock_after_travel_resolution(&mut self) {
        let mut clock = self.game_clock();
        clock.advance_after_travel_resolution();
        self.game_day = clock.game_day;
        self.game_hour = clock.game_hour;
        self.play_started_unix = clock.play_started_unix;
    }
}

fn default_game_day() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PlayerToken;

    #[test]
    fn room_starts_in_lobby() {
        let gm_id = UserId::new();
        let room = RoomState::new(
            RoomId::new(),
            "ABCD12".into(),
            PlayerToken::new(gm_id, "GM"),
            7,
        );
        assert_eq!(room.phase, RoomPhase::Lobby);
        assert_eq!(room.min_players, 4);
        assert_eq!(room.max_players, 5);
        assert_eq!(room.game_day, 1);
        assert_eq!(room.game_hour, 0);
        assert!(room.play_started_unix.is_none());
        assert!(room.npcs.is_empty());
        assert_eq!(room.ccc_deck.tens.len(), 10);
        assert_eq!(room.ccc_deck.units.len(), 10);
        assert!(!room.can_start());
        assert!(!room.is_full());
    }

    #[test]
    fn can_start_requires_four_ready() {
        let gm = UserId::new();
        let mut room = RoomState::new(RoomId::new(), "X".into(), PlayerToken::new(gm, "gm"), 1);
        for i in 0..3 {
            let mut t = PlayerToken::new(UserId::new(), format!("p{i}"));
            t.ready = true;
            room.tokens.push(t);
        }
        room.tokens[0].ready = true;
        assert_eq!(room.player_count(), 4);
        assert!(room.can_start());
    }

    #[test]
    fn travel_resolution_advances_clock_by_two_hours() {
        let gm = UserId::new();
        let mut room = RoomState::new(RoomId::new(), "X".into(), PlayerToken::new(gm, "gm"), 1);
        room.game_hour = 23;
        room.advance_clock_after_travel_resolution();
        assert_eq!(room.game_day, 2);
        assert_eq!(room.game_hour, 1);
    }
}
