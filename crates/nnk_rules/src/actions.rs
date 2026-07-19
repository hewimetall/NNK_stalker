use nnk_domain::{HexCoord, NpcKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Toggle or set ready flag while in lobby.
    SetReady {
        ready: bool,
    },
    /// GM starts the match when lobby has 4–5 ready players.
    StartGame,
    StartMission {
        mission_id: u8,
    },
    /// Пилигрим фаза I: D20 → жетоны событий на поле.
    RollExitZone,
    /// Пилигрим фаза II: D6 очки движения по сектору к жетонам.
    RollExploreD6,
    /// Пилигрим фаза II: взять карту события на своём гексе с жетоном.
    DrawEvent,
    /// Пилигрим фаза II→III: уйти на базу (когда жетоны кончились / после Выброса).
    ReturnToBase,
    /// Пилигрим фаза III: закончить базу → следующий раунд (Выход в Зону).
    FinishBase,
    /// Mission targeting chain (ККК + D20 + D6) from general rules.
    RollD20Location,
    DrawCcc,
    RollD20Hex,
    RollD6Move,
    MoveToken {
        to: HexCoord,
    },
    SpawnNpc {
        name: String,
        kind: NpcKind,
    },
    Chat {
        text: String,
    },
}
