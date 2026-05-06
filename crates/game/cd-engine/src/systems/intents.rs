use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::*;
use cd_core::WorldPos;
use cd_data::defs::SpellId;

/// Намерение сущности переместиться.
#[derive(Debug, Clone, Message)]
pub struct IntentMove {
    pub entity: Entity,
    pub target: WorldPos,
}

/// Намерение сущности применить спелл.
#[derive(Debug, Clone, Message)]
pub struct IntentCastSpell {
    pub caster: Entity,
    pub spell_id: SpellId,
}

/// Намерение завершить свой ход в бою.
#[derive(Debug, Clone, Message)]
pub struct IntentEndTurn {
    pub entity: Entity,
}
