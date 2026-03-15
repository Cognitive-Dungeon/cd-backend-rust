use cd_core::ObjectGuid;

/// Плоское представление сущности для сетевого слоя.
/// Не содержит ECS-типов — безопасно передавать через границу crate.
#[derive(Debug, Clone)]
pub struct EntitySnapshot {
    pub guid: Option<ObjectGuid>, // None если entity без GUID (не должно быть, но безопасно)
    pub x: i32,
    pub y: i32,
    pub glyph: char,
    pub color_rgb: u32,
}
