use crate::systems::intents::IntentMove;
use crate::world::resources::{GridResource, MapResource};
use crate::{game_error::GameError, world::resources::TickResource};

use crate::tick::TickContext;
use bevy_ecs::message::MessageReader;
use bevy_ecs::system::{Query, Res, ResMut};
use cd_ecs::components::{Name, Position};
use cd_ecs::{Guid, Stats};

/// Система движения.
///
/// Сейчас: логирует позиции (placeholder).
/// Будет: обрабатывать IntentMove компонент → реальное перемещение.
pub fn movement_system(
    mut reader: MessageReader<IntentMove>, // Читаем, кто куда хочет пойти
    mut positions: Query<(&Guid, &Name, &mut Position, &Stats)>, // Запрашиваем компоненты
    map: Res<MapResource>,
    mut grid: ResMut<GridResource>,
) {
    for intent in reader.read() {
        // Достаем компоненты того, кто хочет двигаться
        if let Ok((guid, name, mut pos, stats)) = positions.get_mut(intent.entity) {
            // ПРАВИЛО 1: Мертвые не ходят
            if stats.hp <= 0 {
                continue;
            }

            // ПРАВИЛО 2: Коллизия с картой
            if map.inner.is_solid_fast(intent.target) {
                tracing::warn!("{} уперся в стену!", name.0);
                continue;
            }

            // Все проверки пройдены! Двигаем.
            let old_pos = pos.0;
            pos.0 = intent.target;
            grid.inner.move_entity(guid.0, old_pos, intent.target);

            tracing::info!("{} шагнул на {:?}", name.0, intent.target);
        }
    }
}
