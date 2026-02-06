use cd_core::{ObjectGuid, WorldPos};

/// Команды, которые меняют состояние мира.
/// Это "чистые" данные.
#[derive(Debug, Clone)]
pub enum InputCmd {
    /// Игрок хочет переместиться
    Move {
        entity_guid: ObjectGuid,
        target: WorldPos,
    },
    /// Игрок хочет скастовать спелл (заготовка на будущее)
    Cast {
        caster: ObjectGuid,
        spell_id: u32,
        target: ObjectGuid,
    },
}