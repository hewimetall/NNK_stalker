//! Event tokens + event deck from Пилигрим «Правила Карт Событий».

use serde::{Deserialize, Serialize};

use crate::HexCoord;

/// Marker placed on the sector during Phase I.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventToken {
    pub id: u32,
    pub hex: HexCoord,
    pub resolved: bool,
}

/// Card kinds present in the Пилигрим event deck (core set for the web port).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventCard {
    Alfa,
    Artifact,
    Anomaly,
    MutantAttack,
    Gasoline,
    AbandonedTech,
    SwampDoctor,
    Blowout,
    Helicopter,
    GopStop,
    Dirt,
    ZoneBreath,
    ZoneSpirit,
    Eclipse,
    Information,
    Trap,
    Campfire,
    FalseAlarm,
    Forester,
    Mine,
    Minefield,
    Invasion,
    Surprise,
    Squad,
    PdaMessage,
    Guide,
    Companion,
    Memory,
    Radiation,
    HiredWork,
    RadioactiveRain,
    InstrumentGlitch,
    Chance,
    Corpse,
    TahTah,
    Loot,
    HourOfHorror,
    ForeignTerritory,
    ClearSky,
    StrayBullet,
    PoisonFog,
    Pilgrim,
    Hunt,
    SmokeBreak,
    Miracle,
    Arrest,
    Sos,
    Shapeshifter,
    Warning,
    Neutral,
}

impl EventCard {
    pub fn wire_id(self) -> &'static str {
        match self {
            Self::Alfa => "alfa",
            Self::Artifact => "artifact",
            Self::Anomaly => "anomaly",
            Self::MutantAttack => "mutant_attack",
            Self::Gasoline => "gasoline",
            Self::AbandonedTech => "abandoned_tech",
            Self::SwampDoctor => "swamp_doctor",
            Self::Blowout => "blowout",
            Self::Helicopter => "helicopter",
            Self::GopStop => "gop_stop",
            Self::Dirt => "dirt",
            Self::ZoneBreath => "zone_breath",
            Self::ZoneSpirit => "zone_spirit",
            Self::Eclipse => "eclipse",
            Self::Information => "information",
            Self::Trap => "trap",
            Self::Campfire => "campfire",
            Self::FalseAlarm => "false_alarm",
            Self::Forester => "forester",
            Self::Mine => "mine",
            Self::Minefield => "minefield",
            Self::Invasion => "invasion",
            Self::Surprise => "surprise",
            Self::Squad => "squad",
            Self::PdaMessage => "pda_message",
            Self::Guide => "guide",
            Self::Companion => "companion",
            Self::Memory => "memory",
            Self::Radiation => "radiation",
            Self::HiredWork => "hired_work",
            Self::RadioactiveRain => "radioactive_rain",
            Self::InstrumentGlitch => "instrument_glitch",
            Self::Chance => "chance",
            Self::Corpse => "corpse",
            Self::TahTah => "tah_tah",
            Self::Loot => "loot",
            Self::HourOfHorror => "hour_of_horror",
            Self::ForeignTerritory => "foreign_territory",
            Self::ClearSky => "clear_sky",
            Self::StrayBullet => "stray_bullet",
            Self::PoisonFog => "poison_fog",
            Self::Pilgrim => "pilgrim",
            Self::Hunt => "hunt",
            Self::SmokeBreak => "smoke_break",
            Self::Miracle => "miracle",
            Self::Arrest => "arrest",
            Self::Sos => "sos",
            Self::Shapeshifter => "shapeshifter",
            Self::Warning => "warning",
            Self::Neutral => "neutral",
        }
    }

    pub fn title_ru(self) -> &'static str {
        match self {
            Self::Alfa => "Альфа",
            Self::Artifact => "Артефакт",
            Self::Anomaly => "Аномалия",
            Self::MutantAttack => "Атака мутанта",
            Self::Gasoline => "Бензин",
            Self::AbandonedTech => "Брошенная техника",
            Self::SwampDoctor => "Болотный доктор",
            Self::Blowout => "Выброс",
            Self::Helicopter => "Вертолет",
            Self::GopStop => "Гоп-стоп",
            Self::Dirt => "Грязь",
            Self::ZoneBreath => "Дыхание Зоны",
            Self::ZoneSpirit => "Дух Зоны",
            Self::Eclipse => "Затмение",
            Self::Information => "Информация",
            Self::Trap => "Капкан",
            Self::Campfire => "Костер",
            Self::FalseAlarm => "Ложная тревога",
            Self::Forester => "Лесник",
            Self::Mine => "Мина",
            Self::Minefield => "Минное поле",
            Self::Invasion => "Нашествие",
            Self::Surprise => "Неожиданность",
            Self::Squad => "Отряд",
            Self::PdaMessage => "Сообщение на ПДА",
            Self::Guide => "Проводник",
            Self::Companion => "Попутчик",
            Self::Memory => "Память",
            Self::Radiation => "Радиация",
            Self::HiredWork => "Работа по найму",
            Self::RadioactiveRain => "Радиоактивные осадки",
            Self::InstrumentGlitch => "Сбой приборов",
            Self::Chance => "Случай",
            Self::Corpse => "Труп",
            Self::TahTah => "Тах-тах",
            Self::Loot => "Хабар",
            Self::HourOfHorror => "Час ужаса",
            Self::ForeignTerritory => "Чужая территория",
            Self::ClearSky => "Чистое небо",
            Self::StrayBullet => "Шальная пуля",
            Self::PoisonFog => "Ядовитый туман",
            Self::Pilgrim => "Пилигрим",
            Self::Hunt => "Охота",
            Self::SmokeBreak => "Перекур",
            Self::Miracle => "Чудо",
            Self::Arrest => "Арест",
            Self::Sos => "SOS",
            Self::Shapeshifter => "Перевертыш",
            Self::Warning => "Предупреждение",
            Self::Neutral => "Нейтральная",
        }
    }

    /// Rough multiplicity for a ~54-card feel (uniques stay rare).
    pub fn copies_in_deck(self) -> u8 {
        match self {
            Self::Blowout | Self::Pilgrim | Self::SwampDoctor | Self::ClearSky | Self::Miracle => 1,
            Self::Artifact | Self::MutantAttack | Self::Anomaly | Self::Loot | Self::Radiation => 3,
            Self::GopStop | Self::Information | Self::Corpse | Self::Mine | Self::Chance => 2,
            Self::Neutral | Self::FalseAlarm | Self::SmokeBreak => 4,
            _ => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EventDeck {
    pub draw: Vec<EventCard>,
    pub discard: Vec<EventCard>,
}

impl EventDeck {
    pub fn shuffled(seed: u64) -> Self {
        let mut draw = Vec::new();
        for &card in ALL_EVENT_CARDS {
            for _ in 0..card.copies_in_deck() {
                draw.push(card);
            }
        }
        // Deterministic Fisher–Yates from seed (no rand crate in domain).
        let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xA24B_AED4_96E9_1D5F;
        for i in (1..draw.len()).rev() {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1);
            let j = (state >> 33) as usize % (i + 1);
            draw.swap(i, j);
        }
        Self {
            draw,
            discard: Vec::new(),
        }
    }

    pub fn draw_one(&mut self) -> EventCard {
        if self.draw.is_empty() {
            self.draw.append(&mut self.discard);
            // re-shuffle lightly by reversing (seed-free); full reshuffle happens next round.
            self.draw.reverse();
        }
        let card = self.draw.pop().unwrap_or(EventCard::Neutral);
        self.discard.push(card);
        card
    }
}

const ALL_EVENT_CARDS: &[EventCard] = &[
    EventCard::Alfa,
    EventCard::Artifact,
    EventCard::Anomaly,
    EventCard::MutantAttack,
    EventCard::Gasoline,
    EventCard::AbandonedTech,
    EventCard::SwampDoctor,
    EventCard::Blowout,
    EventCard::Helicopter,
    EventCard::GopStop,
    EventCard::Dirt,
    EventCard::ZoneBreath,
    EventCard::ZoneSpirit,
    EventCard::Eclipse,
    EventCard::Information,
    EventCard::Trap,
    EventCard::Campfire,
    EventCard::FalseAlarm,
    EventCard::Forester,
    EventCard::Mine,
    EventCard::Minefield,
    EventCard::Invasion,
    EventCard::Surprise,
    EventCard::Squad,
    EventCard::PdaMessage,
    EventCard::Guide,
    EventCard::Companion,
    EventCard::Memory,
    EventCard::Radiation,
    EventCard::HiredWork,
    EventCard::RadioactiveRain,
    EventCard::InstrumentGlitch,
    EventCard::Chance,
    EventCard::Corpse,
    EventCard::TahTah,
    EventCard::Loot,
    EventCard::HourOfHorror,
    EventCard::ForeignTerritory,
    EventCard::ClearSky,
    EventCard::StrayBullet,
    EventCard::PoisonFog,
    EventCard::Pilgrim,
    EventCard::Hunt,
    EventCard::SmokeBreak,
    EventCard::Miracle,
    EventCard::Arrest,
    EventCard::Sos,
    EventCard::Shapeshifter,
    EventCard::Warning,
    EventCard::Neutral,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_has_blowout_and_pilgrim() {
        let deck = EventDeck::shuffled(42);
        assert!(deck.draw.contains(&EventCard::Blowout));
        assert!(deck.draw.contains(&EventCard::Pilgrim));
        assert!(deck.draw.len() >= 40);
    }

    #[test]
    fn draw_cycles() {
        let mut deck = EventDeck::shuffled(1);
        let n = deck.draw.len();
        for _ in 0..n + 3 {
            let _ = deck.draw_one();
        }
        assert!(!deck.discard.is_empty() || !deck.draw.is_empty());
    }
}
