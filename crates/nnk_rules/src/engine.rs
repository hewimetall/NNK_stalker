use nnk_domain::{
    CccDraw, DomainError, HexCoord, LocationId, MemberRole, PlayerToken, RoomId, RoomPhase,
    RoomState, TurnStage, UserId,
};

use crate::{Action, GameRng};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepOutcome {
    pub state: RoomState,
    pub events: Vec<String>,
}

/// Create a new room state (engine `init`) — starts in Lobby.
pub fn init(room_id: RoomId, code: String, gm: PlayerToken, seed: u64) -> RoomState {
    let mut state = RoomState::new(room_id, code, gm, seed);
    state.history.push("room_created".into());
    state
}

/// Join lobby; enforces max seats (5). Idempotent if already seated
/// (including reconnect after the game has started).
pub fn try_add_player(state: RoomState, token: PlayerToken) -> Result<RoomState, DomainError> {
    // Allow seated players to re-enter mid-game without "already started".
    if state.find_token(token.user_id).is_some() {
        return Ok(state);
    }
    if state.phase != RoomPhase::Lobby {
        return Err(DomainError::AlreadyPlaying);
    }
    if state.is_full() {
        return Err(DomainError::LobbyFull {
            max: state.max_players,
        });
    }
    let mut next = state;
    next.history
        .push(format!("player_joined:{}", token.display_name));
    next.tokens.push(token);
    Ok(next)
}

/// Backward-compatible helper used by older call sites (panics on full — prefer `try_add_player`).
pub fn add_player(state: RoomState, token: PlayerToken) -> RoomState {
    try_add_player(state, token).expect("lobby must not be full")
}

pub fn legal_actions(state: &RoomState, user_id: UserId, role: MemberRole) -> Vec<Action> {
    let mut actions = Vec::new();
    match state.phase {
        RoomPhase::Lobby => {
            if state.find_token(user_id).is_some() {
                let ready = state.find_token(user_id).map(|t| t.ready).unwrap_or(false);
                actions.push(Action::SetReady { ready: !ready });
                actions.push(Action::Chat {
                    text: String::new(),
                });
            }
            if role == MemberRole::Gm && state.can_start() {
                actions.push(Action::StartGame);
            }
        }
        RoomPhase::Playing => {
            if let Some(token) = state.find_token(user_id) {
                if state.active_user_id == Some(user_id) {
                    match token.travel_stage {
                        TurnStage::NeedLocation => actions.push(Action::RollD20Location),
                        TurnStage::NeedSector => actions.push(Action::DrawCcc),
                        TurnStage::NeedHex => actions.push(Action::RollD20Hex),
                        TurnStage::NeedD6 => actions.push(Action::RollD6Move),
                        TurnStage::NeedMove => {
                            if token.move_points > 0 {
                                for to in nnk_domain::sector_hexes() {
                                    if can_move(token.hex, to, token.move_points) {
                                        actions.push(Action::MoveToken { to });
                                    }
                                }
                            }
                        }
                    }
                }
                actions.push(Action::Chat {
                    text: String::new(),
                });
            }
        }
    }
    actions
}

/// Apply action. On illegal input returns `Err` and does not mutate (policy).
pub fn step(
    state: RoomState,
    user_id: UserId,
    role: MemberRole,
    action: Action,
) -> Result<StepOutcome, DomainError> {
    let mut rng = GameRng::from_seed(state.rng_seed);
    for _ in 0..state.history.len() {
        let _ = rng.d20();
    }

    let mut next = state;
    let mut events = Vec::new();

    match action {
        Action::SetReady { ready } => {
            if next.phase != RoomPhase::Lobby {
                return Err(DomainError::AlreadyPlaying);
            }
            ensure_player(&next, user_id)?;
            let token = next.find_token_mut(user_id).unwrap();
            token.ready = ready;
            let ev = format!("ready:{}:{ready}", user_id.0);
            next.history.push(ev.clone());
            events.push(ev);
        }
        Action::StartGame => {
            if role != MemberRole::Gm {
                return Err(DomainError::Forbidden("only GM can start the game"));
            }
            if next.phase != RoomPhase::Lobby {
                return Err(DomainError::AlreadyPlaying);
            }
            let have = next.player_count();
            if have < next.min_players || have > next.max_players {
                return Err(DomainError::LobbySize {
                    min: next.min_players,
                    max: next.max_players,
                    have,
                });
            }
            if !next.all_ready() {
                return Err(DomainError::LobbyNotReady);
            }
            next.phase = RoomPhase::Playing;
            next.active_user_id = next.tokens.first().map(|t| t.user_id);
            for token in &mut next.tokens {
                token.location = None;
                token.sector = None;
                token.hex = HexCoord::ZERO;
                token.move_points = 0;
                token.travel_stage = TurnStage::NeedLocation;
            }
            let ev = format!("game_started:{have}");
            next.history.push(ev.clone());
            events.push(ev);
        }
        Action::StartMission { mission_id } => {
            require_playing(&next)?;
            if role != MemberRole::Gm {
                return Err(DomainError::Forbidden("only GM can start missions"));
            }
            if !(1..=10).contains(&mission_id) {
                return Err(DomainError::InvalidMissionId(mission_id));
            }
            next.mission_id = mission_id;
            let ev = format!("mission_started:{mission_id}");
            next.history.push(ev.clone());
            events.push(ev);
        }
        Action::RollD20Location => {
            require_playing(&next)?;
            ensure_player(&next, user_id)?;
            ensure_active_player(&next, user_id)?;
            ensure_stage(&next, user_id, TurnStage::NeedLocation, "roll_d20_location")?;
            let (rerolls, value, location) = roll_location(&mut rng);
            for reroll in rerolls {
                let ev = format!("d20_location_reroll:{reroll}");
                next.history.push(ev.clone());
                events.push(ev);
            }
            let token = next.find_token_mut(user_id).unwrap();
            token.location = Some(location);
            token.sector = None;
            token.move_points = 0;
            token.travel_stage = TurnStage::NeedSector;
            let ev = format!(
                "d20_location:{}:{}:{}",
                value,
                location.name_ru(),
                user_id.0
            );
            next.history.push(ev.clone());
            events.push(ev);
        }
        Action::DrawCcc => {
            require_playing(&next)?;
            ensure_player(&next, user_id)?;
            ensure_active_player(&next, user_id)?;
            ensure_stage(&next, user_id, TurnStage::NeedSector, "draw_ccc")?;
            let (tens, units) = rng.ccc_parts();
            let draw = CccDraw::combine(tens, units)?;
            let token = next.find_token_mut(user_id).unwrap();
            token.sector = Some(draw.sector);
            token.travel_stage = TurnStage::NeedHex;
            let ev = format!("ccc:{tens}:{units}:{}:{}", draw.sector, user_id.0);
            next.history.push(ev.clone());
            events.push(ev);
        }
        Action::RollD20Hex => {
            require_playing(&next)?;
            ensure_player(&next, user_id)?;
            ensure_active_player(&next, user_id)?;
            ensure_stage(&next, user_id, TurnStage::NeedHex, "roll_d20_hex")?;
            let (value, hex) = roll_sector_hex(&mut rng)?;
            let token = next.find_token_mut(user_id).unwrap();
            token.hex = hex;
            token.move_points = 0;
            token.travel_stage = TurnStage::NeedD6;
            let ev = format!("d20_hex:{value}:{}:{},{}", user_id.0, hex.q, hex.r);
            next.history.push(ev.clone());
            events.push(ev);
        }
        Action::RollD6Move => {
            require_playing(&next)?;
            ensure_player(&next, user_id)?;
            ensure_active_player(&next, user_id)?;
            ensure_stage(&next, user_id, TurnStage::NeedD6, "roll_d6_move")?;
            let value = rng.d6();
            let has_legal_move = {
                let token = next.find_token_mut(user_id).unwrap();
                token.move_points = value;
                token.travel_stage = TurnStage::NeedMove;
                nnk_domain::sector_hexes()
                    .into_iter()
                    .any(|to| can_move(token.hex, to, token.move_points))
            };
            let ev = format!("d6_move:{value}:{}", user_id.0);
            next.history.push(ev.clone());
            events.push(ev);
            if !has_legal_move {
                finish_turn(&mut next, user_id);
            }
        }
        Action::MoveToken { to } => {
            require_playing(&next)?;
            ensure_active_player(&next, user_id)?;
            ensure_stage(&next, user_id, TurnStage::NeedMove, "move_token")?;
            if !nnk_domain::is_sector_hex(to) {
                return Err(DomainError::IllegalMove {
                    distance: u32::MAX,
                    points: 0,
                });
            }
            let token = next.find_token(user_id).ok_or(DomainError::UnknownPlayer)?;
            let from = token.hex;
            let points = token.move_points;
            let distance = from.distance(to);
            if distance == 0 || distance > u32::from(points) {
                return Err(DomainError::IllegalMove { distance, points });
            }
            let token = next.find_token_mut(user_id).unwrap();
            token.hex = to;
            token.move_points = 0;
            token.travel_stage = TurnStage::NeedLocation;
            let ev = format!("moved:{}:{from:?}->{to:?}", user_id.0);
            next.history.push(ev.clone());
            events.push(ev);
            finish_turn(&mut next, user_id);
        }
        Action::Chat { text } => {
            ensure_player(&next, user_id)?;
            let text: String = text.chars().take(280).collect();
            let ev = format!("chat:{}:{text}", user_id.0);
            next.history.push(ev.clone());
            events.push(ev);
        }
    }

    next.rng_seed = next.rng_seed.wrapping_add(next.history.len() as u64);

    Ok(StepOutcome {
        state: next,
        events,
    })
}

fn require_playing(state: &RoomState) -> Result<(), DomainError> {
    if state.phase != RoomPhase::Playing {
        Err(DomainError::StillInLobby)
    } else {
        Ok(())
    }
}

fn ensure_player(state: &RoomState, user_id: UserId) -> Result<(), DomainError> {
    if state.find_token(user_id).is_none() {
        Err(DomainError::UnknownPlayer)
    } else {
        Ok(())
    }
}

fn ensure_active_player(state: &RoomState, user_id: UserId) -> Result<(), DomainError> {
    match state.active_user_id {
        Some(active) if active == user_id => Ok(()),
        _ => Err(DomainError::NotActivePlayer),
    }
}

fn ensure_stage(
    state: &RoomState,
    user_id: UserId,
    expected: TurnStage,
    got: &'static str,
) -> Result<(), DomainError> {
    let token = state
        .find_token(user_id)
        .ok_or(DomainError::UnknownPlayer)?;
    if token.travel_stage == expected {
        Ok(())
    } else {
        Err(DomainError::IllegalStage {
            expected: stage_action_name(token.travel_stage),
            got,
        })
    }
}

fn stage_action_name(stage: TurnStage) -> &'static str {
    match stage {
        TurnStage::NeedLocation => "roll_d20_location",
        TurnStage::NeedSector => "draw_ccc",
        TurnStage::NeedHex => "roll_d20_hex",
        TurnStage::NeedD6 => "roll_d6_move",
        TurnStage::NeedMove => "move_token",
    }
}

fn finish_turn(state: &mut RoomState, user_id: UserId) {
    let Some(idx) = state.tokens.iter().position(|t| t.user_id == user_id) else {
        state.active_user_id = None;
        return;
    };
    if let Some(token) = state.tokens.get_mut(idx) {
        token.move_points = 0;
        token.travel_stage = TurnStage::NeedLocation;
    }
    if state.tokens.is_empty() {
        state.active_user_id = None;
        return;
    }
    let next_idx = (idx + 1) % state.tokens.len();
    state.active_user_id = Some(state.tokens[next_idx].user_id);
}

fn roll_location(rng: &mut GameRng) -> (Vec<u8>, u8, LocationId) {
    let mut rerolls = Vec::new();
    loop {
        let v = rng.d20();
        match LocationId::from_d20(v) {
            Ok(Some(loc)) => return (rerolls, v, loc),
            Ok(None) => rerolls.push(v),
            Err(_) => unreachable!("d20 generated an out-of-range value"),
        }
    }
}

fn roll_sector_hex(rng: &mut GameRng) -> Result<(u8, HexCoord), DomainError> {
    loop {
        let v = rng.d20();
        if let Some(hex) = nnk_domain::hex_from_d20(v)? {
            return Ok((v, hex));
        }
    }
}

pub fn can_move(from: HexCoord, to: HexCoord, move_points: u8) -> bool {
    let d = from.distance(to);
    d > 0 && d <= u32::from(move_points) && nnk_domain::is_sector_hex(to)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (RoomState, UserId) {
        let gm = UserId::new();
        let state = init(
            RoomId::new(),
            "TEST01".into(),
            PlayerToken::new(gm, "GM"),
            99,
        );
        (state, gm)
    }

    fn fill_lobby_ready(mut state: RoomState, gm: UserId) -> RoomState {
        while state.player_count() < 4 {
            let p = UserId::new();
            let name = format!("p{}", state.player_count());
            state = try_add_player(state, PlayerToken::new(p, name)).unwrap();
        }
        for t in &mut state.tokens {
            t.ready = true;
        }
        assert!(state.find_token(gm).unwrap().ready);
        assert!(state.can_start());
        state
    }

    fn start_playing(state: RoomState, gm: UserId) -> RoomState {
        let state = fill_lobby_ready(state, gm);
        step(state, gm, MemberRole::Gm, Action::StartGame)
            .unwrap()
            .state
    }

    fn advance_to_hex_stage(state: RoomState, user_id: UserId) -> RoomState {
        let state = step(state, user_id, MemberRole::Player, Action::RollD20Location)
            .unwrap()
            .state;
        step(state, user_id, MemberRole::Player, Action::DrawCcc)
            .unwrap()
            .state
    }

    fn advance_to_d6_stage(state: RoomState, user_id: UserId) -> RoomState {
        let state = advance_to_hex_stage(state, user_id);
        step(state, user_id, MemberRole::Player, Action::RollD20Hex)
            .unwrap()
            .state
    }

    #[test]
    fn init_history() {
        let (s, _) = setup();
        assert_eq!(s.history, vec!["room_created".to_string()]);
        assert_eq!(s.phase, RoomPhase::Lobby);
    }

    #[test]
    fn lobby_rejects_sixth() {
        let (mut s, _) = setup();
        for i in 0..4 {
            s = try_add_player(s, PlayerToken::new(UserId::new(), format!("p{i}"))).unwrap();
        }
        assert!(s.is_full());
        let err = try_add_player(s, PlayerToken::new(UserId::new(), "overflow")).unwrap_err();
        assert_eq!(err, DomainError::LobbyFull { max: 5 });
    }

    #[test]
    fn start_requires_four_ready() {
        let (s, gm) = setup();
        let err = step(s.clone(), gm, MemberRole::Gm, Action::StartGame).unwrap_err();
        assert!(matches!(err, DomainError::LobbySize { .. }));

        let mut s = try_add_player(s, PlayerToken::new(UserId::new(), "a")).unwrap();
        s = try_add_player(s, PlayerToken::new(UserId::new(), "b")).unwrap();
        s = try_add_player(s, PlayerToken::new(UserId::new(), "c")).unwrap();
        let err = step(s.clone(), gm, MemberRole::Gm, Action::StartGame).unwrap_err();
        assert_eq!(err, DomainError::LobbyNotReady);

        let s = fill_lobby_ready(s, gm);
        let out = step(s, gm, MemberRole::Gm, Action::StartGame).unwrap();
        assert_eq!(out.state.phase, RoomPhase::Playing);
    }

    #[test]
    fn gameplay_blocked_in_lobby() {
        let (s, gm) = setup();
        let err = step(s, gm, MemberRole::Player, Action::RollD6Move).unwrap_err();
        assert_eq!(err, DomainError::StillInLobby);
    }

    #[test]
    fn cannot_draw_ccc_before_location() {
        let (s, gm) = setup();
        let s = start_playing(s, gm);
        let err = step(s, gm, MemberRole::Player, Action::DrawCcc).unwrap_err();
        assert!(matches!(
            err,
            DomainError::IllegalStage {
                expected: "roll_d20_location",
                got: "draw_ccc"
            }
        ));
    }

    #[test]
    fn cannot_roll_d20_hex_before_sector() {
        let (s, gm) = setup();
        let s = start_playing(s, gm);
        let s = step(s, gm, MemberRole::Player, Action::RollD20Location)
            .unwrap()
            .state;
        let err = step(s, gm, MemberRole::Player, Action::RollD20Hex).unwrap_err();
        assert!(matches!(
            err,
            DomainError::IllegalStage {
                expected: "draw_ccc",
                got: "roll_d20_hex"
            }
        ));
    }

    #[test]
    fn cannot_roll_d6_before_hex() {
        let (s, gm) = setup();
        let s = start_playing(s, gm);
        let s = advance_to_hex_stage(s, gm);
        let err = step(s, gm, MemberRole::Player, Action::RollD6Move).unwrap_err();
        assert!(matches!(
            err,
            DomainError::IllegalStage {
                expected: "roll_d20_hex",
                got: "roll_d6_move"
            }
        ));
    }

    #[test]
    fn set_ready_and_start_five() {
        let (mut s, gm) = setup();
        for i in 0..4 {
            s = try_add_player(s, PlayerToken::new(UserId::new(), format!("p{i}"))).unwrap();
        }
        for t in s.tokens.clone() {
            s = step(
                s,
                t.user_id,
                MemberRole::Player,
                Action::SetReady { ready: true },
            )
            .unwrap()
            .state;
        }
        assert!(s.can_start());
        assert_eq!(s.player_count(), 5);
        let out = step(s, gm, MemberRole::Gm, Action::StartGame).unwrap();
        assert_eq!(out.state.phase, RoomPhase::Playing);
    }

    #[test]
    fn gm_starts_mission_only_when_playing() {
        let (s, gm) = setup();
        let s = start_playing(s, gm);
        let out = step(
            s.clone(),
            gm,
            MemberRole::Gm,
            Action::StartMission { mission_id: 3 },
        )
        .unwrap();
        assert_eq!(out.state.mission_id, 3);
        let err = step(
            s.clone(),
            gm,
            MemberRole::Gm,
            Action::StartMission { mission_id: 0 },
        )
        .unwrap_err();
        assert_eq!(err, DomainError::InvalidMissionId(0));
        let err = step(
            s,
            gm,
            MemberRole::Gm,
            Action::StartMission { mission_id: 11 },
        )
        .unwrap_err();
        assert_eq!(err, DomainError::InvalidMissionId(11));
    }

    #[test]
    fn player_cannot_start_game() {
        let (s, gm) = setup();
        let s = fill_lobby_ready(s, gm);
        let err = step(s, gm, MemberRole::Player, Action::StartGame).unwrap_err();
        assert_eq!(err, DomainError::Forbidden("only GM can start the game"));
    }

    #[test]
    fn illegal_move_keeps_state() {
        let (s, gm) = setup();
        let mut s = start_playing(s, gm);
        let token = s.find_token_mut(gm).unwrap();
        token.travel_stage = TurnStage::NeedMove;
        token.move_points = 6;
        token.hex = HexCoord::ZERO;
        let before = s.clone();
        let err = step(
            s,
            gm,
            MemberRole::Player,
            Action::MoveToken {
                to: HexCoord::new(5, 0),
            },
        )
        .unwrap_err();
        assert!(matches!(err, DomainError::IllegalMove { .. }));
        assert_eq!(before.tokens[0].hex, HexCoord::ZERO);
    }

    #[test]
    fn d6_then_move() {
        let (s, gm) = setup();
        let s = start_playing(s, gm);
        let s = advance_to_d6_stage(s, gm);
        let out = step(s, gm, MemberRole::Player, Action::RollD6Move).unwrap();
        let points = out.state.tokens[0].move_points;
        assert!((1..=6).contains(&points));
        let from = out.state.tokens[0].hex;
        let to = nnk_domain::sector_hexes()
            .into_iter()
            .find(|to| can_move(from, *to, points))
            .unwrap();
        let out = step(out.state, gm, MemberRole::Player, Action::MoveToken { to }).unwrap();
        assert_eq!(out.state.tokens[0].hex, to);
        assert_eq!(out.state.tokens[0].move_points, 0);
        assert_ne!(out.state.active_user_id, Some(gm));
    }

    #[test]
    fn location_and_ccc() {
        let (s, gm) = setup();
        let s = start_playing(s, gm);
        let out = step(s, gm, MemberRole::Player, Action::RollD20Location).unwrap();
        assert!(out.state.tokens[0].location.is_some());
        let out = step(out.state, gm, MemberRole::Player, Action::DrawCcc).unwrap();
        let sector = out.state.tokens[0].sector.unwrap();
        assert!((1..=100).contains(&sector));
        let last = out.state.history.last().unwrap();
        assert!(last.starts_with("ccc:"));
        assert_eq!(last.split(':').count(), 5);
    }

    #[test]
    fn unknown_player() {
        let (s, _) = setup();
        let err = step(
            s,
            UserId::new(),
            MemberRole::Player,
            Action::SetReady { ready: true },
        )
        .unwrap_err();
        assert_eq!(err, DomainError::UnknownPlayer);
    }

    #[test]
    fn add_player_idempotent() {
        let (s, _) = setup();
        let p = UserId::new();
        let s = try_add_player(s, PlayerToken::new(p, "A")).unwrap();
        let s2 = try_add_player(s.clone(), PlayerToken::new(p, "A")).unwrap();
        assert_eq!(s.tokens.len(), s2.tokens.len());
    }

    #[test]
    fn legal_lobby_vs_playing() {
        let (s, gm) = setup();
        let acts = legal_actions(&s, gm, MemberRole::Gm);
        assert!(acts.iter().any(|a| matches!(a, Action::SetReady { .. })));
        assert!(!acts.iter().any(|a| matches!(a, Action::StartGame)));

        let s = fill_lobby_ready(s, gm);
        let acts = legal_actions(&s, gm, MemberRole::Gm);
        assert!(acts.iter().any(|a| matches!(a, Action::StartGame)));
    }

    #[test]
    fn can_move_helper() {
        assert!(can_move(HexCoord::ZERO, HexCoord::new(1, 0), 1));
        assert!(!can_move(HexCoord::ZERO, HexCoord::new(2, 0), 1));
        assert!(!can_move(HexCoord::ZERO, HexCoord::ZERO, 5));
    }

    #[test]
    fn chat_truncated() {
        let (s, gm) = setup();
        let long = "x".repeat(400);
        let out = step(s, gm, MemberRole::Player, Action::Chat { text: long }).unwrap();
        let last = out.state.history.last().unwrap();
        assert!(last.len() < 400);
    }

    #[test]
    fn replay_determinism_same_actions() {
        let gm = UserId::new();
        let room = RoomId::new();
        let s1 = start_playing(
            init(room, "R1".into(), PlayerToken::new(gm, "GM"), 1234),
            gm,
        );
        let s2 = start_playing(
            init(room, "R1".into(), PlayerToken::new(gm, "GM"), 1234),
            gm,
        );
        let a1 = advance_to_d6_stage(s1, gm);
        let a2 = advance_to_d6_stage(s2, gm);
        let a1 = step(a1, gm, MemberRole::Player, Action::RollD6Move).unwrap();
        let a2 = step(a2, gm, MemberRole::Player, Action::RollD6Move).unwrap();
        assert_eq!(
            a1.state.tokens[0].move_points,
            a2.state.tokens[0].move_points
        );
    }

    #[test]
    fn cannot_join_or_ready_after_start() {
        let (s, gm) = setup();
        let s = start_playing(s, gm);
        let err = try_add_player(s.clone(), PlayerToken::new(UserId::new(), "late")).unwrap_err();
        assert_eq!(err, DomainError::AlreadyPlaying);
        // Existing member can reconnect after start.
        let back = try_add_player(s.clone(), PlayerToken::new(gm, "GM")).unwrap();
        assert_eq!(back.phase, RoomPhase::Playing);
        assert!(back.find_token(gm).is_some());
        let err = step(
            s.clone(),
            gm,
            MemberRole::Player,
            Action::SetReady { ready: false },
        )
        .unwrap_err();
        assert_eq!(err, DomainError::AlreadyPlaying);
        let err = step(s, gm, MemberRole::Gm, Action::StartGame).unwrap_err();
        assert_eq!(err, DomainError::AlreadyPlaying);
    }

    #[test]
    fn legal_playing_includes_moves() {
        let (s, gm) = setup();
        let mut s = start_playing(s, gm);
        s.tokens[0].travel_stage = TurnStage::NeedMove;
        s.tokens[0].move_points = 2;
        let acts = legal_actions(&s, gm, MemberRole::Gm);
        assert!(acts.iter().any(|a| matches!(a, Action::MoveToken { .. })));
        assert!(!acts.iter().any(|a| matches!(a, Action::RollD20Location)));
        assert!(!acts
            .iter()
            .any(|a| matches!(a, Action::StartMission { .. })));
        assert!(acts.iter().any(|a| matches!(a, Action::Chat { .. })));
    }

    #[test]
    fn d6_four_generates_uncapped_sector_moves() {
        let (s, gm) = setup();
        let mut s = start_playing(s, gm);
        let token = s.find_token_mut(gm).unwrap();
        token.travel_stage = TurnStage::NeedMove;
        token.hex = HexCoord::new(0, -2);
        token.move_points = 4;

        let acts = legal_actions(&s, gm, MemberRole::Player);
        let moves: Vec<_> = acts
            .iter()
            .filter_map(|a| match a {
                Action::MoveToken { to } => Some(*to),
                _ => None,
            })
            .collect();
        assert!(moves.contains(&HexCoord::new(0, 2)));
        assert!(moves.iter().all(|to| nnk_domain::is_sector_hex(*to)));
        assert!(moves
            .iter()
            .all(|to| HexCoord::new(0, -2).distance(*to) <= 4));
        assert!(!moves.contains(&HexCoord::new(3, -2)));
    }

    #[test]
    fn add_player_compat_wrapper() {
        let (s, _) = setup();
        let s = add_player(s, PlayerToken::new(UserId::new(), "x"));
        assert_eq!(s.player_count(), 2);
    }
}
