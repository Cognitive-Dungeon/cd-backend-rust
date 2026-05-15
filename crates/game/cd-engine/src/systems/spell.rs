use bevy::ecs::prelude::*;
use cd_data::defs::SpellTarget;
use cd_ecs::components::Position;
use cd_ecs::{Guid, InstanceId};

use crate::systems::intents::IntentCastSpell;
use crate::world::resources::DefsCache;
use crate::world::subsystems::{CombatSubsystem, SpatialSubsystem};

pub fn spell_system(
    mut reader: MessageReader<IntentCastSpell>,
    defs: Res<DefsCache>,
    spatial: SpatialSubsystem,
    positions: Query<(&Guid, &Position, &InstanceId)>,
    mut combat: CombatSubsystem,
) {
    for intent in reader.read() {
        let Some(spell) = defs.spells.get(&intent.spell_id) else {
            continue;
        };

        let Ok((caster_guid, caster_pos, instance)) = positions.get(intent.caster) else {
            continue;
        };

        let targets: Vec<Entity> = match spell.target {
            SpellTarget::Self_ => {
                let mut found = Vec::new();
                let mut seen_guids = std::collections::HashSet::new();

                for &guid in spatial.get_entities_in_bucket(*instance, caster_pos.0) {
                    if guid != caster_guid.0
                        && seen_guids.insert(guid)
                        && let Some(entity) = spatial.get_entity(guid)
                        && let Ok((_, target_pos, _)) = positions.get(entity)
                    {
                        let caster_tile = cd_core::TilePos::new(caster_pos.0.x(), caster_pos.0.y());
                        let target_tile = cd_core::TilePos::new(target_pos.0.x(), target_pos.0.y());
                        if caster_tile.chebyshev_distance(target_tile) <= spell.range {
                            found.push(entity);
                        }
                    }
                }
                found
            }
            _ => continue,
        };

        if targets.is_empty() {
            tracing::info!(
                "{} cast {} — no targets in range",
                caster_guid.0,
                spell.slug
            );
            continue;
        }

        // 1. Инициируем бой (Если кастер еще не в бою, он стянет всех мобов в радиусе)
        combat.initiate_combat(intent.caster, *instance, caster_pos.0, &spatial);

        // 2. Тратим 2 AP за любое заклинание
        if let Err(reason) = combat.try_consume_ap(intent.caster, 2) {
            tracing::warn!("SpellSystem: {} cannot cast: {}", caster_guid.0, reason);
            continue;
        }

        // 3. Применяем эффект через фасад (ни одной строки работы с HP здесь нет!)
        for target_entity in targets {
            combat.apply_effect(target_entity, &spell.effect);
        }
    }
}
