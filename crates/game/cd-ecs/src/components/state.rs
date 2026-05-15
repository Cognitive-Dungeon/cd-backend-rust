use bevy::ecs::{component::Component, entity::Entity, reflect::ReflectComponent};
use bevy::reflect::Reflect;
use cd_core::{Glyph, ObjectGuid, WorldPos};
use cd_data::defs::{CreatureId, FurnitureId};

#[derive(Debug, Clone, Copy, PartialEq, Component, Reflect)]
#[reflect(Component)]
pub struct Position(pub WorldPos);

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Guid(pub ObjectGuid);

#[derive(Debug, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub struct Stats {
    pub hp: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub max_mana: i32,
}

/// Визуальное представление (Glyph).
/// В Go: RenderComponent { Glyph }
#[derive(Debug, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub struct Render {
    pub glyph: Glyph,
}

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Controller {
    pub agent_id: String, // ID сессии / токен
}

/// Маркеры, указывающие из какого чертежа (Blueprint) создана сущность.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Reflect)]
#[reflect(Component)]
pub struct Creature(pub CreatureId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Reflect)]
#[reflect(Component)]
pub struct Furniture(pub FurnitureId);

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Door {
    pub is_open: bool,
}

/// Вешается на невидимую сущность, которая управляет конкретным боем.
#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct CombatBubble {
    /// Очередь ходов (отсортирована по инициативе)
    pub turn_order: Vec<Entity>,
    /// Индекс текущего ходящего в массиве turn_order
    pub current_turn_idx: usize,
    /// Номер раунда
    pub round: u32,
}

impl CombatBubble {
    pub fn current_actor(&self) -> Option<Entity> {
        self.turn_order.get(self.current_turn_idx).copied()
    }
}

/// Вешается на игрока/моба, которого затянуло в бой.
#[derive(Debug, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub struct InCombat {
    /// Ссылка на сущность CombatBubble
    pub bubble: Entity,
    pub action_points: i32,
    pub movement_points: i32,
}

/// Идентификатор инстанса (уровня, подземелья, зоны).
/// Разделяет существ в пространстве: они не видят друг друга, если ID разные.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct InstanceId(u32);

impl InstanceId {
    /// Главный открытый мир по умолчанию
    pub const OVERWORLD: Self = Self(1);

    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl Default for InstanceId {
    fn default() -> Self {
        Self::OVERWORLD
    }
}

/// Хранит состояние конкретного инстанса (например, для перемотки времени и детерминизма).
/// Вешается на отдельную Entity-менеджера уровня.
#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct InstanceState {
    pub id: InstanceId,
    pub current_tick: u64,
    pub seed: u64,
}
