use serde::{Deserialize, Serialize};

// --- ВНУТРЕННИЙ РОУТИНГ (Не отправляется клиенту) ---

#[derive(Debug, Clone)]
pub enum OutboundTarget {
    Broadcast,
    Single(cd_core::ObjectGuid),
    Multiple(Vec<cd_core::ObjectGuid>),
}

/// "Конверт", который ходит между Движком и Сетью.
#[derive(Debug, Clone)]
pub struct OutboundMessage {
    pub target: OutboundTarget,
    pub packet: ServerPacket,
}

impl OutboundMessage {
    pub fn broadcast(packet: ServerPacket) -> Self {
        Self {
            target: OutboundTarget::Broadcast,
            packet,
        }
    }

    pub fn unicast(guid: cd_core::ObjectGuid, packet: ServerPacket) -> Self {
        Self {
            target: OutboundTarget::Single(guid),
            packet,
        }
    }

    pub fn multicast(guids: Vec<cd_core::ObjectGuid>, packet: ServerPacket) -> Self {
        Self {
            target: OutboundTarget::Multiple(guids),
            packet,
        }
    }
}

/// Сообщения от Клиента к Серверу
#[derive(Debug, Deserialize)]
#[serde(tag = "op", content = "d")] // { "op": "LOGIN", "d": { ... } }
pub enum ClientPacket {
    Login { token: String },
    Move { x: i32, y: i32 },
    // Cast { spell_id: u32, target_guid: String }
}

/// Сообщения от Сервера к Клиенту
#[derive(Debug, Serialize)]
#[serde(tag = "op", content = "d")]
#[derive(Clone)]
pub enum ServerPacket {
    AuthSuccess {
        guid: String,
    },
    AuthFailed {
        reason: String,
    },
    Snapshot {
        tick: u64,
        entities: Vec<EntityView>,
    },
    SystemMessage {
        text: String,
        is_private: bool,
    },
}

#[derive(Debug, Serialize, Clone)]
pub struct EntityView {
    pub guid: String,
    pub x: i32,
    pub y: i32,
    pub glyph: char,
    pub color: String, // Hex
}
