use bevy_ecs::component::Component;
use cd_core::{ObjectGuid, WorldPos};
use cd_data::defs::{CreatureId, FurnitureId};

#[derive(Debug, Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct Position(pub WorldPos);

#[derive(Component, Clone, Copy)]
pub struct Guid(pub ObjectGuid);

#[derive(Debug, Clone, Copy, Component)]
pub struct Stats {
    pub hp: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub max_mana: i32,
}

/// Визуальное представление (Glyph).
/// В Go: RenderComponent { Glyph }
#[derive(Debug, Clone, Copy, Component)]
pub struct Render {
    pub glyph: cd_common::Glyph,
}

/// Имя (для UI и логов).
#[derive(Debug, Clone, Component)]
pub struct Name(pub String);

#[derive(Debug, Clone, Component)]
pub struct Controller {
    pub agent_id: String, // ID сессии / токен
}

/// Маркеры, указывающие из какого чертежа (Blueprint) создана сущность.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct Creature(pub CreatureId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct Furniture(pub FurnitureId);

#[derive(Debug, Clone, Component)]
pub struct Door {
    pub is_open: bool,
}
