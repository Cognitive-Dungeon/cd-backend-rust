use crate::{game_error::GameError, world::resources::TickResource};

use crate::tick::TickContext;
use bevy_ecs::system::{Query, Res};
use cd_ecs::components::{Name, Position};

/// Система движения.
///
/// Сейчас: логирует позиции (placeholder).
/// Будет: обрабатывать IntentMove компонент → реальное перемещение.
pub fn run(query: Query<(&Name, &Position)>, tick: Res<TickResource>) {
    for (name, pos) in query.iter() {
        tracing::trace!("[Tick {}] {} is at {:?}", tick.id.0, name.0, pos.0);
    }
}
