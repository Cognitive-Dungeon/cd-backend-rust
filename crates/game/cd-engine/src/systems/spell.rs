use bevy_ecs::prelude::*;
use cd_core::grid::neighbors::NEIGHBORS_8;
use cd_data::defs::{SpellEffect, SpellTarget};
use cd_ecs::Guid;
use cd_ecs::components::{Position, Stats};

use crate::systems::intents::IntentCastSpell;
use crate::world::resources::{DefsCache, GridResource, RegistryResource};

pub fn spell_system(
    mut reader: MessageReader<IntentCastSpell>,
    defs: Res<DefsCache>,
    registry: Res<RegistryResource>,
    grid: Res<GridResource>,
    mut entities: Query<(&Guid, &Position, &mut Stats)>,
) {
    for intent in reader.read() {
        tracing::info!("spell_system: got intent for spell_id {}", intent.spell_id);
        // 1. Находим определение спелла
        let Some(spell) = defs.spells_by_id.get(&intent.spell_id) else {
            tracing::warn!("Unknown spell_id: {}", intent.spell_id);
            continue;
        };

        // 2. Получаем позицию кастера
        let Ok((caster_guid, caster_pos, _)) = entities.get(intent.caster) else {
            continue;
        };
        let caster_guid = caster_guid.0;
        let caster_pos = caster_pos.0;

        // 3. Резолвим цели
        let targets: Vec<Entity> = match spell.target {
            SpellTarget::Self_ => {
                let mut found: Vec<Entity> = Vec::new();
                let mut seen_guids = std::collections::HashSet::new();

                // Ищем в том же bucket что и кастер — все кто рядом
                for &guid in grid.inner.query_bucket(caster_pos) {
                    if guid != caster_guid
                        && seen_guids.insert(guid)
                        && let Some(entity) = registry.inner.get_entity(guid)
                    {
                        // Точная проверка расстояния по реальной позиции
                        if let Ok((_, target_pos, _)) = entities.get(entity) {
                            let caster_tile = cd_core::TilePos::new(caster_pos.x(), caster_pos.y());
                            let target_tile =
                                cd_core::TilePos::new(target_pos.0.x(), target_pos.0.y());
                            if caster_tile.chebyshev_distance(target_tile) <= spell.range {
                                found.push(entity);
                            }
                        }
                    }
                }
                found
            }
            // Остальные таргетинги — в будущих итерациях
            _ => {
                tracing::info!("Spell target {:?} not yet implemented", spell.target);
                continue;
            }
        };

        if targets.is_empty() {
            tracing::info!("{} cast {} — no targets in range", caster_guid, spell.slug);
            continue;
        }

        // 4. Применяем эффект к каждой цели
        for target_entity in targets {
            apply_effect(target_entity, &spell.effect, &mut entities);
        }
    }
}

fn apply_effect(
    target: Entity,
    effect: &SpellEffect,
    entities: &mut Query<(&Guid, &Position, &mut Stats)>,
) {
    let Ok((guid, _, mut stats)) = entities.get_mut(target) else {
        return;
    };

    match effect {
        SpellEffect::Damage { amount, .. } => {
            stats.hp = (stats.hp - amount).max(0);
            tracing::info!(
                target = %guid.0,
                damage = amount,
                hp_left = stats.hp,
                "Spell damage applied"
            );
        }
        SpellEffect::Heal { amount } => {
            stats.hp = (stats.hp + amount).min(stats.max_hp);
            tracing::info!(
                target = %guid.0,
                heal = amount,
                hp = stats.hp,
                "Spell heal applied"
            );
        }
    }
}
