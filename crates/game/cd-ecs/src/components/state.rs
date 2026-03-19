use bevy_ecs::component::Component;
use cd_core::{ObjectGuid, WorldPos};

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
