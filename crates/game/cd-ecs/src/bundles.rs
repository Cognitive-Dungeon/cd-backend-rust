use bevy::ecs::{bundle::Bundle, name::Name};

use crate::{
    Stats,
    components::{Creature, Furniture, Guid, InstanceId, Position, Render},
};

/// Базовый набор компонентов для любого живого существа (игрока, моба, NPC)
/// Гарантирует, что никто не заспавнится без ХП или координат.
#[derive(Bundle)]
pub struct CreatureBundle {
    // Базовые идентификаторы и пространство
    pub guid: Guid,
    pub instance: InstanceId,
    pub position: Position,
    pub name: Name,
    pub render: Render,

    // Специфичный маркер сущности
    pub creature: Creature,

    pub stats: Stats,
    // Механики BRP UGE
    // pub characteristics: Characteristics,
    // pub anatomy: Anatomy,
    // pub action_points: ActionPoints,
    // pub encumbrance: Encumbrance,
}

/// Базовый набор компонентов для интерактивных объектов (сундуки, двери)
#[derive(Bundle)]
pub struct FurnitureBundle {
    pub guid: Guid,
    pub instance: InstanceId,
    pub position: Position,
    pub name: Name,
    pub render: Render,
    pub furniture: Furniture,
}
