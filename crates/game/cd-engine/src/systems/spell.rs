use bevy_ecs::prelude::*;
use cd_data::defs::SpellTarget;
use cd_ecs::Guid;
use cd_ecs::components::Position;

use crate::systems::intents::IntentCastSpell;
use crate::world::resources::DefsCache;
use crate::world::subsystems::{CombatSubsystem, SpatialSubsystem};

pub fn spell_system(
    mut reader: MessageReader<IntentCastSpell>,
    defs: Res<DefsCache>,
    spatial: SpatialSubsystem,
    positions: Query<(&Guid, &Position)>,
    mut combat: CombatSubsystem,
) {
    for intent in reader.read() {
        let Some(spell) = defs.spells.get(&intent.spell_id) else {
            continue;
        };

        let Ok((caster_guid, caster_pos)) = positions.get(intent.caster) else {
            continue;
        };

        let targets: Vec<Entity> = match spell.target {
            SpellTarget::Self_ => {
                let mut found = Vec::new();
                let mut seen_guids = std::collections::HashSet::new();

                for &guid in spatial.get_entities_in_bucket(caster_pos.0) {
                    if guid != caster_guid.0
                        && seen_guids.insert(guid)
                        && let Some(entity) = spatial.get_entity(guid)
                        && let Ok((_, target_pos)) = positions.get(entity)
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

        // Применяем эффект через фасад (ни одной строки работы с HP здесь нет!)
        for target_entity in targets {
            combat.apply_effect(target_entity, &spell.effect);
        }
    }
}
