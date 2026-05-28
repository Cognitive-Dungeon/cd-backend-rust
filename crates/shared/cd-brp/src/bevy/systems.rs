// src/bevy/systems.rs
use crate::{action::effect::CombatEffect, bevy::BRPVitals};
use bevy::{platform::collections::HashMap, prelude::*};
use cd_core::ObjectGuid;

/// Глобальный ресурс для мгновенного поиска Entity по ObjectGuid O(1)
#[derive(Resource, Default)]
pub struct NetworkEntityMap(pub HashMap<ObjectGuid, Entity>);

/// Система применения урона на основе сетевых ивентов (CombatEffect)
pub fn apply_combat_effects_system(
    mut events: MessageReader<CombatEffect>,
    entity_map: Res<NetworkEntityMap>,
    mut q_vitals: Query<&mut BRPVitals>,
) {
    for effect in events.read() {
        if let CombatEffect::Hit {
            target_id,
            damage_taken,
            ..
        } = effect
        {
            // 1. Быстро переводим сетевой GUID в локальную Bevy Entity
            if let Some(&entity) = entity_map.0.get(target_id) {
                // 2. Мутируем компоненты
                if let Ok(mut vitals) = q_vitals.get_mut(entity) {
                    vitals.hp -= *damage_taken;
                }
            }
        }
    }
}
