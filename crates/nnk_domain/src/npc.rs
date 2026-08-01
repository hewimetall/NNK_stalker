use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{HexCoord, LocationId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NpcKind {
    Stalker,
    Mutant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Npc {
    pub id: Uuid,
    pub name: String,
    pub kind: NpcKind,
    pub controlled_by: Option<UserId>,
    pub hex: HexCoord,
    pub location: Option<LocationId>,
    pub sector: Option<u8>,
}

impl Npc {
    pub fn new(name: impl Into<String>, kind: NpcKind, gm_user_id: UserId, hex: HexCoord) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind,
            controlled_by: Some(gm_user_id),
            hex,
            location: None,
            sector: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npc_defaults_to_gm_control() {
        let gm = UserId::new();
        let npc = Npc::new("Сталкер", NpcKind::Stalker, gm, HexCoord::ZERO);
        assert_eq!(npc.controlled_by, Some(gm));
        assert_eq!(npc.location, None);
        assert_eq!(npc.sector, None);
    }
}
