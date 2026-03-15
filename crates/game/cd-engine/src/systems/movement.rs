use crate::game_error::GameError;
use crate::game_world::GameWorld;
use crate::tick::TickContext;
use cd_ecs::components::{Position, Name};

/// Система движения.
/// Пример использования GameWorld + game_system! макроса.
///
/// Сейчас: логирует позиции (placeholder).
/// Будет: обрабатывать IntentMove компонент → реальное перемещение.
pub fn run(
    world: &mut GameWorld,
    _ctx: &mut TickContext,
) -> Result<(), GameError> {
    for (_entity, (name, pos)) in world.query::<(&Name, &Position)>().iter() {
        tracing::trace!("{} is at {:?}", name.0, pos.0);
    }
    Ok(())
}