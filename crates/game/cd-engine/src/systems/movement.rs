use crate::systems::intents::IntentMove;
use crate::world::resources::{GridResource, MapResource, RegistryResource};
use bevy_ecs::message::MessageReader;
use bevy_ecs::system::{Query, Res, ResMut};
use cd_core::ObjectGuid;
use cd_ecs::components::{Name, Position};
use cd_ecs::{Guid, Stats};
use std::collections::HashMap;

pub fn movement_system(
    mut reader: MessageReader<IntentMove>,
    mut movers: Query<(&Guid, &Name, &mut Position, &Stats)>,
    map: Res<MapResource>,
    mut grid: ResMut<GridResource>,
    registry: Res<RegistryResource>,
) {
    let positions: HashMap<ObjectGuid, cd_core::WorldPos> = movers
        .iter()
        .map(|(guid, _, pos, _)| (guid.0, pos.0))
        .collect();

    for intent in reader.read() {
        if let Ok((guid, name, mut pos, stats)) = movers.get_mut(intent.entity) {
            // ПРАВИЛО 1: Мёртвые не ходят
            if stats.hp <= 0 {
                continue;
            }

            // ПРАВИЛО 2: Коллизия с картой
            if map.inner.is_solid_fast(intent.target) {
                tracing::warn!("{} bumped into a solid object", name.0);
                continue;
            }

            // ПРАВИЛО 3: Столкновения с сущностями
            let entities_in_bucket = grid.inner.query_bucket(intent.target);
            let mut bumped = false;

            for &other_guid in entities_in_bucket {
                if other_guid == guid.0 {
                    continue;
                }
                // Проверяем реальную позицию из снапшота
                if positions.get(&other_guid) == Some(&intent.target) {
                    tracing::info!("{} bumped into entity {}!", name.0, other_guid);
                    bumped = true;
                    break;
                }
            }

            if bumped {
                continue;
            }

            let old_pos = pos.0;
            pos.0 = intent.target;
            grid.inner.move_entity(guid.0, old_pos, intent.target);

            tracing::info!("{} moved to {:?}", name.0, intent.target);
        }
    }
}
