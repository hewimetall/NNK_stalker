//! Пилигрим v3.0 round helpers (ExitZone → Explore → ReturnBase).

use nnk_domain::{
    DomainError, EventCard, EventToken, HexCoord, LocationId, Npc, NpcKind, RoomState, RoundPhase,
    TurnStage, UserId,
};

use crate::GameRng;

pub fn begin_playing_round(state: &mut RoomState) {
    state.round_phase = RoundPhase::ExitZone;
    state.round_number = state.round_number.max(1);
    state.event_tokens.clear();
    state.last_event = None;
    state.event_deck = nnk_domain::EventDeck::shuffled(state.rng_seed ^ state.round_number as u64);
    for token in &mut state.tokens {
        token.location = Some(LocationId::Cordon);
        token.sector = Some(1);
        token.ccc_tens = None;
        token.ccc_units = None;
        token.hex = HexCoord::ZERO;
        token.target_hex = None;
        token.move_points = 0;
        token.travel_stage = TurnStage::Idle;
        if token.hp == 0 {
            token.hp = 100;
        }
    }
}

pub fn place_event_tokens(
    state: &mut RoomState,
    rng: &mut GameRng,
    count: u8,
) -> Result<Vec<HexCoord>, DomainError> {
    let hexes = nnk_domain::sector_hexes();
    let mut available: Vec<HexCoord> = hexes.into_iter().filter(|h| *h != HexCoord::ZERO).collect();
    // shuffle available
    for i in (1..available.len()).rev() {
        let j = (rng.d20() as usize + rng.d6() as usize) % (i + 1);
        available.swap(i, j);
    }
    let n = (count as usize).min(available.len()).max(1);
    let mut placed = Vec::new();
    state.event_tokens.clear();
    for hex in available.into_iter().take(n) {
        let id = state.next_event_token_id;
        state.next_event_token_id = state.next_event_token_id.saturating_add(1);
        state.event_tokens.push(EventToken {
            id,
            hex,
            resolved: false,
        });
        placed.push(hex);
    }
    state.round_phase = RoundPhase::Explore;
    Ok(placed)
}

pub fn token_on_unresolved_event(state: &RoomState, user_id: UserId) -> Option<u32> {
    let hex = state.find_token(user_id)?.hex;
    state
        .event_tokens
        .iter()
        .find(|t| !t.resolved && t.hex == hex)
        .map(|t| t.id)
}

pub fn resolve_event_card(
    state: &mut RoomState,
    rng: &mut GameRng,
    user_id: UserId,
    card: EventCard,
    token_id: u32,
) -> Vec<String> {
    let mut events = Vec::new();
    if let Some(tok) = state.event_tokens.iter_mut().find(|t| t.id == token_id) {
        tok.resolved = true;
    }
    let title = card.title_ru();
    state.last_event = Some(format!("{} ({})", title, card.wire_id()));
    events.push(format!("event:{}:{}:{}", card.wire_id(), title, user_id.0));

    match card {
        EventCard::Blowout => {
            for tok in &mut state.event_tokens {
                tok.resolved = true;
            }
            state.round_phase = RoundPhase::ReturnBase;
            events.push(format!("blowout:round:{}", state.round_number));
        }
        EventCard::Artifact => {
            if let Some(p) = state.find_token_mut(user_id) {
                p.artifacts = p.artifacts.saturating_add(1);
                p.rubles = p.rubles.saturating_add(500);
            }
            events.push(format!("loot_artifact:{}", user_id.0));
        }
        EventCard::Loot | EventCard::Corpse | EventCard::Chance => {
            let gain = u32::from(rng.d6()) * 100;
            if let Some(p) = state.find_token_mut(user_id) {
                p.rubles = p.rubles.saturating_add(gain);
            }
            events.push(format!("loot_rubles:{gain}:{}", user_id.0));
        }
        EventCard::MutantAttack | EventCard::GopStop | EventCard::Hunt | EventCard::Squad => {
            let enemy_roll = rng.d6();
            let player_roll = rng.d6();
            let dmg = u16::from(enemy_roll) * 3;
            if let Some(p) = state.find_token_mut(user_id) {
                if player_roll >= enemy_roll {
                    events.push(format!(
                        "combat_win:{}:p{player_roll}:e{enemy_roll}",
                        user_id.0
                    ));
                    p.rubles = p.rubles.saturating_add(u32::from(enemy_roll) * 50);
                } else {
                    p.hp = p.hp.saturating_sub(dmg);
                    events.push(format!(
                        "combat_loss:{}:dmg{dmg}:hp{}",
                        user_id.0, p.hp
                    ));
                }
            }
            if matches!(card, EventCard::MutantAttack) {
                state.npcs.push(Npc::new(
                    "Мутант",
                    NpcKind::Mutant,
                    state.gm_user_id,
                    state
                        .find_token(user_id)
                        .map(|t| t.hex)
                        .unwrap_or(HexCoord::ZERO),
                ));
            }
        }
        EventCard::Radiation | EventCard::RadioactiveRain | EventCard::PoisonFog => {
            let dmg = u16::from(rng.d6()) * 2;
            if let Some(p) = state.find_token_mut(user_id) {
                p.hp = p.hp.saturating_sub(dmg);
            }
            events.push(format!("radiation:{dmg}:{}", user_id.0));
        }
        EventCard::Mine | EventCard::Minefield | EventCard::Trap | EventCard::StrayBullet => {
            let dmg = u16::from(rng.d6()) * 4;
            if let Some(p) = state.find_token_mut(user_id) {
                p.hp = p.hp.saturating_sub(dmg);
            }
            events.push(format!("hazard:{dmg}:{}", user_id.0));
        }
        EventCard::SwampDoctor | EventCard::Campfire | EventCard::Miracle | EventCard::ClearSky => {
            if let Some(p) = state.find_token_mut(user_id) {
                p.hp = (p.hp + 20).min(120);
            }
            events.push(format!("heal:{}", user_id.0));
        }
        EventCard::Pilgrim => {
            state.mission_id = 1;
            events.push("story:pilgrim_started".into());
        }
        EventCard::Anomaly => {
            let roll = rng.d6();
            if roll <= 2 {
                if let Some(p) = state.find_token_mut(user_id) {
                    p.hp = p.hp.saturating_sub(10);
                }
                events.push(format!("anomaly_hurt:{roll}:{}", user_id.0));
            } else if roll >= 5 {
                if let Some(p) = state.find_token_mut(user_id) {
                    p.artifacts = p.artifacts.saturating_add(1);
                }
                events.push(format!("anomaly_find:{roll}:{}", user_id.0));
            } else {
                events.push(format!("anomaly_pass:{roll}:{}", user_id.0));
            }
        }
        _ => {
            events.push(format!("event_flavor:{}:{}", card.wire_id(), user_id.0));
        }
    }

    if state.round_phase == RoundPhase::Explore && state.all_event_tokens_resolved() {
        state.round_phase = RoundPhase::ReturnBase;
        events.push("phase:return_base".into());
    }
    events
}

pub fn finish_base_and_next_round(state: &mut RoomState) {
    state.advance_clock_after_travel_resolution();
    state.advance_clock_after_travel_resolution(); // ~4 game hours at base
    state.round_number = state.round_number.saturating_add(1);
    begin_playing_round(state);
}
