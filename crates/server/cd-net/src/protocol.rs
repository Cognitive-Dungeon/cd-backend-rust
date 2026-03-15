use serde::{Deserialize, Serialize};

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
    AuthSuccess { guid: String },
    AuthFailed { reason: String },
    Snapshot { tick: u64, entities: Vec<EntityView> },
}

#[derive(Debug, Serialize, Clone)]
pub struct EntityView {
    pub guid: String,
    pub x: i32,
    pub y: i32,
    pub glyph: char,
    pub color: String, // Hex
}