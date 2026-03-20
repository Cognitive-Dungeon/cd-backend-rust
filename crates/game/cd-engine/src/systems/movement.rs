use crate::systems::intents::IntentMove;
use crate::world::resources::{GridResource, MapResource, RegistryResource};
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
    registry: Res<RegistryResource>,
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
                tracing::warn!("{} bumped into a solid object", name.0);
                continue;
            }

            // ПРАВИЛО 3: Столкновения с сущностями
            let entities_in_target = grid.inner.query_bucket(intent.target);
            let mut bumped_into_solid_entity = false;

            for &other_guid in entities_in_target {
                // Если в клетке есть кто-то другой
                if other_guid != guid.0
                    && let Some(_other_entity) = registry.inner.get_entity(other_guid)
                {
                    // TODO: Здесь мы будем проверять, есть ли у other_entity компонент Solid
                    // или компонент Health (тогда это враг -> атакуем).
                    // Пока просто считаем, что все сущности твердые.

                    tracing::info!("{} bumped into entity {}!", name.0, other_guid);
                    bumped_into_solid_entity = true;

                    // Если бы у нас был IntentInteract или IntentAttack, мы бы сгенерировали
                    // его прямо здесь и прервали бы шаг.
                }
            }

            // Если клетка занята другой сущностью (например, закрытой дверью или гоблином)
            if bumped_into_solid_entity {
                continue; // Отменяем шаг
            }

            // Все проверки пройдены! Двигаем.
            let old_pos = pos.0;
            pos.0 = intent.target;
            grid.inner.move_entity(guid.0, old_pos, intent.target);

            tracing::info!("{} moved to {:?}", name.0, intent.target);
        }
    }
}
