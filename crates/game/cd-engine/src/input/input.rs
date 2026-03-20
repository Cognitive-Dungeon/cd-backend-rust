use bevy_ecs::message::Message;
use cd_core::{ObjectGuid, WorldPos};

/// Команды, которые меняют состояние мира.
/// Это "чистые" данные.
#[derive(Debug, Clone, Message)]
pub enum InputCmd {
    /// Игрок хочет переместиться
    Move {
        entity_guid: ObjectGuid,
        target: WorldPos,
    },
    /// Игрок хочет скастовать спелл (заготовка на будущее)
    CastSpell {
        entity_guid: ObjectGuid,
        spell_slug: String,
    },
    SpawnPlayer {
        entity_guid: ObjectGuid,
        name: String,
    },
}
