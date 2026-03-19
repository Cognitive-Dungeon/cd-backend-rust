use cd_common::Glyph;
use cd_core::ObjectGuid;

/// Плоское представление сущности для сетевого слоя.
/// Не содержит ECS-типов — безопасно передавать через границу crate.
#[derive(Debug, Clone)]
pub struct EntitySnapshot {
    pub guid: Option<ObjectGuid>, // None если entity без GUID (не должно быть, но безопасно)
    pub x: i32,
    pub y: i32,
    pub glyph: Glyph,
}

/// Плоское представление чанка для передачи наружу (сеть/сохранения)
#[derive(Debug, Clone)]
pub struct ChunkSnapshot {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub palette: Vec<Glyph>,
    pub indices: Vec<u8>, // Всегда 256 элементов (16x16)
}
