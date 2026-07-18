use nnk_domain::{
    CccDraw, DomainError, HexCoord, LocationId, MemberRole, PlayerToken, RoomId, RoomState, UserId,
};

use crate::{Action, GameRng};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepOutcome {
    pub state: RoomState,
    pub events: Vec<String>,
}

/// Create a new room state (engine `init`).
pub fn init(room_id: RoomId, code: String, gm: PlayerToken, seed: u64) -> RoomState {
    let mut state = RoomState::new(room_id, code, gm, seed);
    state.history.push("room_created".into());
    state
}

pub fn add_player(mut state: RoomState, token: PlayerToken) -> RoomState {
    if state.find_token(token.user_id).is_none() {
        state
            .history
            .push(format!("player_joined:{}", token.display_name));
        state.tokens.push(token);
    }
    state
}

pub fn legal_actions(state: &RoomState, user_id: UserId, role: MemberRole) -> Vec<Action> {
    let mut actions = vec![
        Action::RollD20Location,
        Action::DrawCcc,
        Action::RollD20Hex,
        Action::RollD6Move,
        Action::Chat {
            text: String::new(),
        },
    ];
    if role == MemberRole::Gm {
        actions.insert(0, Action::StartMission { mission_id: state.mission_id });
    }
    if let Some(token) = state.find_token(user_id) {
        if token.move_points > 0 {
            // Neighbors within remaining points (radius = move_points, capped for listing).
            let radius = token.move_points.min(2);
            for q in -i32::from(radius)..=i32::from(radius) {
                for r in -i32::from(radius)..=i32::from(radius) {
                    let to = HexCoord::new(token.hex.q + q, token.hex.r + r);
                    if to != token.hex && token.hex.distance(to) <= u32::from(token.move_points) {
                        actions.push(Action::MoveToken { to });
                    }
                }
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
    // Advance RNG by history length for deterministic continuation after prior steps.
    for _ in 0..state.history.len() {
        let _ = rng.d20();
    }

    let mut next = state;
    let mut events = Vec::new();

    match action {
        Action::StartMission { mission_id } => {
            if role != MemberRole::Gm {
                return Err(DomainError::Forbidden("only GM can start missions"));
            }
            next.mission_id = mission_id;
            let ev = format!("mission_started:{mission_id}");
            next.history.push(ev.clone());
            events.push(ev);
        }
        Action::RollD20Location => {
            ensure_player(&next, user_id)?;
            let (value, location) = roll_location(&mut rng);
            let token = next.find_token_mut(user_id).unwrap();
            token.location = Some(location);
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
            ensure_player(&next, user_id)?;
            let (tens, units) = rng.ccc_parts();
            let draw = CccDraw::combine(tens, units)?;
            let token = next.find_token_mut(user_id).unwrap();
            token.sector = Some(draw.sector);
            let ev = format!("ccc:{}:{}", draw.sector, user_id.0);
            next.history.push(ev.clone());
            events.push(ev);
        }
        Action::RollD20Hex => {
            ensure_player(&next, user_id)?;
            let value = rng.d20();
            let ev = format!("d20_hex:{value}:{}", user_id.0);
            next.history.push(ev.clone());
            events.push(ev);
        }
        Action::RollD6Move => {
            ensure_player(&next, user_id)?;
            let value = rng.d6();
            let token = next.find_token_mut(user_id).unwrap();
            token.move_points = value;
            let ev = format!("d6_move:{value}:{}", user_id.0);
            next.history.push(ev.clone());
            events.push(ev);
        }
        Action::MoveToken { to } => {
            let token = next
                .find_token(user_id)
                .ok_or(DomainError::UnknownPlayer)?;
            let from = token.hex;
            let points = token.move_points;
            let distance = from.distance(to);
            if distance == 0 || distance > u32::from(points) {
                return Err(DomainError::IllegalMove { distance, points });
            }
            let token = next.find_token_mut(user_id).unwrap();
            token.hex = to;
            token.move_points = points.saturating_sub(distance as u8);
            let ev = format!("moved:{}:{from:?}->{to:?}", user_id.0);
            next.history.push(ev.clone());
            events.push(ev);
        }
        Action::Chat { text } => {
            ensure_player(&next, user_id)?;
            let text: String = text.chars().take(280).collect();
            let ev = format!("chat:{}:{text}", user_id.0);
            next.history.push(ev.clone());
            events.push(ev);
        }
    }

    // Mix seed forward so subsequent steps diverge deterministically with history.
    next.rng_seed = next.rng_seed.wrapping_add(next.history.len() as u64);

    Ok(StepOutcome {
        state: next,
        events,
    })
}

fn ensure_player(state: &RoomState, user_id: UserId) -> Result<(), DomainError> {
    if state.find_token(user_id).is_none() {
        Err(DomainError::UnknownPlayer)
    } else {
        Ok(())
    }
}

fn roll_location(rng: &mut GameRng) -> (u8, LocationId) {
    loop {
        let v = rng.d20();
        if let Ok(Some(loc)) = LocationId::from_d20(v) {
            return (v, loc);
        }
    }
}

pub fn can_move(from: HexCoord, to: HexCoord, move_points: u8) -> bool {
    let d = from.distance(to);
    d > 0 && d <= u32::from(move_points)
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

    #[test]
    fn init_history() {
        let (s, _) = setup();
        assert_eq!(s.history, vec!["room_created".to_string()]);
    }

    #[test]
    fn gm_starts_mission() {
        let (s, gm) = setup();
        let out = step(s, gm, MemberRole::Gm, Action::StartMission { mission_id: 3 }).unwrap();
        assert_eq!(out.state.mission_id, 3);
    }

    #[test]
    fn player_cannot_start_mission() {
        let (s, gm) = setup();
        let err = step(
            s,
            gm,
            MemberRole::Player,
            Action::StartMission { mission_id: 2 },
        )
        .unwrap_err();
        assert_eq!(err, DomainError::Forbidden("only GM can start missions"));
    }

    #[test]
    fn illegal_move_keeps_state() {
        let (s, gm) = setup();
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
        // caller keeps `before` — step consumed a clone conceptually
        assert_eq!(before.tokens[0].hex, HexCoord::ZERO);
    }

    #[test]
    fn d6_then_move() {
        let (s, gm) = setup();
        let out = step(s, gm, MemberRole::Player, Action::RollD6Move).unwrap();
        let points = out.state.tokens[0].move_points;
        assert!((1..=6).contains(&points));
        let to = HexCoord::new(1, 0);
        let out = step(
            out.state,
            gm,
            MemberRole::Player,
            Action::MoveToken { to },
        )
        .unwrap();
        assert_eq!(out.state.tokens[0].hex, to);
        assert_eq!(out.state.tokens[0].move_points, points - 1);
    }

    #[test]
    fn location_and_ccc() {
        let (s, gm) = setup();
        let out = step(s, gm, MemberRole::Player, Action::RollD20Location).unwrap();
        assert!(out.state.tokens[0].location.is_some());
        let out = step(out.state, gm, MemberRole::Player, Action::DrawCcc).unwrap();
        let sector = out.state.tokens[0].sector.unwrap();
        assert!((1..=100).contains(&sector));
    }

    #[test]
    fn unknown_player() {
        let (s, _) = setup();
        let err = step(
            s,
            UserId::new(),
            MemberRole::Player,
            Action::RollD6Move,
        )
        .unwrap_err();
        assert_eq!(err, DomainError::UnknownPlayer);
    }

    #[test]
    fn add_player_idempotent() {
        let (s, _) = setup();
        let p = UserId::new();
        let s = add_player(s, PlayerToken::new(p, "A"));
        let s2 = add_player(s.clone(), PlayerToken::new(p, "A"));
        assert_eq!(s.tokens.len(), s2.tokens.len());
    }

    #[test]
    fn legal_includes_gm_mission() {
        let (s, gm) = setup();
        let acts = legal_actions(&s, gm, MemberRole::Gm);
        assert!(acts
            .iter()
            .any(|a| matches!(a, Action::StartMission { .. })));
        let acts = legal_actions(&s, gm, MemberRole::Player);
        assert!(!acts
            .iter()
            .any(|a| matches!(a, Action::StartMission { .. })));
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
        let s1 = init(room, "R1".into(), PlayerToken::new(gm, "GM"), 1234);
        let s2 = init(room, "R1".into(), PlayerToken::new(gm, "GM"), 1234);
        let a1 = step(s1, gm, MemberRole::Player, Action::RollD6Move).unwrap();
        let a2 = step(s2, gm, MemberRole::Player, Action::RollD6Move).unwrap();
        assert_eq!(a1.state.tokens[0].move_points, a2.state.tokens[0].move_points);
        assert_eq!(a1.events, a2.events);
    }
}
