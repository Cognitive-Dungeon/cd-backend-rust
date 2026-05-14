use bevy_ecs::message::Message;
use cd_core::{Direction, ObjectGuid};

/// Команды, которые меняют состояние мира.
/// Это "чистые" данные.
#[derive(Debug, Clone, Message)]
pub enum InputCmd {
    /// Игрок хочет переместиться
    Move {
        entity_guid: ObjectGuid,
        direction: Direction,
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
    EndTurn {
        entity_guid: ObjectGuid,
    },
    ReloadData,
}
