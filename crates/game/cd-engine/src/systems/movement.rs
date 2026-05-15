use crate::systems::intents::IntentMove;
use crate::world::subsystems::{CombatSubsystem, SpatialSubsystem};
use bevy::ecs::message::MessageReader;
use bevy::ecs::system::Query;
use cd_core::ObjectGuid;
use cd_ecs::components::Position;
use cd_ecs::{Guid, InstanceId};
use std::collections::HashMap;

pub fn movement_system(
    mut reader: MessageReader<IntentMove>,
    mut movers: Query<(&Guid, &mut Position, &InstanceId)>,
    mut spatial: SpatialSubsystem,
    mut combat: CombatSubsystem,
) {
    let positions: HashMap<ObjectGuid, cd_core::WorldPos> = movers
        .iter()
        .map(|(guid, pos, _instance)| (guid.0, pos.0))
        .collect();

    for intent in reader.read() {
        // 1. Сначала безопасно проверяем статус через подсистему
        if !combat.is_alive(intent.entity) {
            tracing::warn!(
                "MovementSystem: Entity {:?} cannot move, it is dead or invalid",
                intent.entity
            );
            continue;
        }

        // 2. Только если жив, достаем сущность для мутации позиции
        if let Ok((guid, mut pos, instance)) = movers.get_mut(intent.entity) {
            if spatial.is_solid_map(*instance, intent.target) {
                tracing::info!(
                    "MovementSystem: {} bumped into map wall at {:?}",
                    guid.0,
                    intent.target
                );
                continue;
            }

            let entities_in_bucket = spatial.get_entities_in_bucket(*instance, intent.target);
            let mut bumped = false;

            for &other_guid in entities_in_bucket {
                if other_guid == guid.0 {
                    continue;
                }
                if positions.get(&other_guid) == Some(&intent.target) {
                    tracing::info!(
                        "MovementSystem: {} bumped into entity {}!",
                        guid.0,
                        other_guid
                    );
                    bumped = true;
                    break;
                }
            }

            if bumped {
                continue;
            }

            if let Err(reason) = combat.try_consume_mp(intent.entity, 1) {
                tracing::warn!("MovementSystem: {} cannot move: {}", guid.0, reason);
                continue;
            }

            let old_pos = pos.0;
            pos.0 = intent.target;
            spatial.move_entity(*instance, guid.0, old_pos, intent.target);

            tracing::info!(
                "MovementSystem: {} successfully moved to {:?}",
                guid.0,
                intent.target
            );
        }
    }
}
