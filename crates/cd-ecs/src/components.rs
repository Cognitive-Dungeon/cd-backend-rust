use cd_core::{ObjectGuid, WorldPos};

// --- Core Components ---

/// Позиция в мире.
/// В Go это было: type PositionComponent struct { TilePos }
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)] // В памяти это просто WorldPos (u64), zero overhead
pub struct Position(pub WorldPos);

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

/// Характеристики (HP, Mana).
#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub hp: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub is_dead: bool,
}

// --- Logic Components (Tags & Flags) ---

/// Тэг: Сущностью управляет игрок или внешний агент.
/// В Go: ControllerComponent
#[derive(Debug, Clone)]
pub struct Controller {
    pub agent_id: String, // ID сессии / токен
}

/// Тэг: Блокирует движение (коллизия).
#[derive(Debug, Clone, Copy)]
pub struct Blocker;

// --- Relations ---

/// Связь: Сущность ссылается на Guid (например, Target).
/// В Go это часто решалось через поля в struct, тут можно делать отдельным компонентом.
#[derive(Debug, Clone, Copy)]
pub struct Target(pub ObjectGuid);