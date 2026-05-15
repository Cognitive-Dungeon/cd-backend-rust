use bevy::ecs::component::Component;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    BodyPart, HitLocationType,
    anatomy::{DamageResult, SubstancePool, VitalStats, WoundType},
};

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Anatomy {
    pub total_hp: i32,
    pub current_hp: i32,
    pub parts: HashMap<HitLocationType, BodyPart>,
    pub substances: SubstancePool,
    pub vitals: VitalStats,
}

impl Anatomy {
    pub fn new_humanoid(total_hp: i32) -> Self {
        let mut parts = HashMap::new();
        for loc in [
            HitLocationType::RightLeg,
            HitLocationType::LeftLeg,
            HitLocationType::Abdomen,
            HitLocationType::Chest,
            HitLocationType::RightArm,
            HitLocationType::LeftArm,
            HitLocationType::Head,
        ] {
            parts.insert(loc, BodyPart::new(total_hp, loc, 0));
        }
        Self {
            total_hp,
            current_hp: total_hp,
            parts,
            substances: SubstancePool::default_human(),
            vitals: VitalStats::default(),
        }
    }

    pub fn is_alive(&self) -> bool {
        if self.current_hp <= 0 {
            return false;
        }
        for critical_loc in [
            HitLocationType::Head,
            HitLocationType::Chest,
            HitLocationType::Abdomen,
        ] {
            if let Some(part) = self.parts.get(&critical_loc)
                && part.is_destroyed()
            {
                return false;
            }
        }
        true
    }

    /// Legacy BRP-метод (возвращает i32 для совместимости)
    pub fn apply_damage(&mut self, location: HitLocationType, raw_damage: i32) -> i32 {
        self.apply_damage_detailed(location, raw_damage, WoundType::Blunt, 50.0)
            .damage_dealt()
    }

    /// Новый симулятивный метод
    pub fn apply_damage_detailed(
        &mut self,
        location: HitLocationType,
        raw_damage: i32,
        wound_type: WoundType,
        penetration_mm: f32,
    ) -> DamageResult {
        let Some(part) = self.parts.get_mut(&location) else {
            return DamageResult::Missed;
        };
        let actual_damage = (raw_damage - part.armor).max(0);
        if actual_damage == 0 {
            return DamageResult::Blocked;
        }

        let max_possible = part.max_hp * 2;
        let final_damage = actual_damage.min(max_possible);

        part.hp -= final_damage;
        self.current_hp -= final_damage;

        if part.is_destroyed() && !part.injuries.contains(&crate::Injury::Severed) {
            part.injuries.push(crate::Injury::Severed);
            part.is_destroyed = true;
        } else if part.is_useless() && !part.injuries.contains(&crate::Injury::Fractured) {
            part.injuries.push(crate::Injury::Fractured);
            part.is_useless = true;
        }

        // Здесь можно добавить логику повреждения тканей/органов на основе penetration_mm
        let pain = (final_damage as f32 * 2.0).min(100.0);
        DamageResult::Hit {
            damage_dealt: final_damage,
            bleeding_added: 0.0,
            pain_caused: pain,
        }
    }
}
