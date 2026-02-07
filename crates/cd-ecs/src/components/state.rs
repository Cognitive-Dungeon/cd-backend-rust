use cd_core::{ObjectGuid, WorldPos};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Position(pub WorldPos);

#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub hp: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub max_mana: i32,
}

/// Визуальное представление (Glyph).
/// В Go: RenderComponent { Glyph }
#[derive(Debug, Clone, Copy)]
pub struct Render {
    pub glyph: char,
    pub color_rgb: u32, // 0xRRGGBB
}

/// Имя (для UI и логов).
#[derive(Debug, Clone)]
pub struct Name(pub String);

#[derive(Debug, Clone)]
pub struct Controller {
    pub agent_id: String, // ID сессии / токен
}
