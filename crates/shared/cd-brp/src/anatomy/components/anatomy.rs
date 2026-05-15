use bevy::ecs::component::Component;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    BodyPart, HitLocationType,
    anatomy::{
        DamageResult, PenetrationProfile, SubstancePool, TISSUE_PENETRATION_ORDER, VitalStats,
        Wound, WoundSeverity, WoundType,
    },
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
        let profile = PenetrationProfile::blunt();
        self.apply_damage_detailed(location, raw_damage, profile)
            .damage_dealt()
    }

    /// Новый симулятивный метод проникновения через ткани
    pub fn apply_damage_detailed(
        &mut self,
        location: HitLocationType,
        raw_damage: i32,
        penetration: PenetrationProfile,
    ) -> DamageResult {
        let Some(part) = self.parts.get_mut(&location) else {
            return DamageResult::Missed;
        };

        // 1. Броня защищает
        let effective_depth = penetration.effective_depth(part.armor, 1.0);
        let actual_damage = (raw_damage - part.armor).max(0);

        if actual_damage == 0 || effective_depth <= 0.0 {
            return DamageResult::Blocked;
        }

        // 2. Лимит урона по BRP (не больше 2x максимума части за удар)
        let max_possible = part.max_hp * 2;
        let final_damage = actual_damage.min(max_possible);

        part.hp -= final_damage;
        self.current_hp -= final_damage;

        // 3. Пробитие тканей (Tissue Penetration)
        let mut remaining_penetration = effective_depth;
        let mut affected_tissues = Vec::new();
        let mut total_bleeding_rate = 0.0;
        let mut total_pain = 0.0;

        for tissue_type in TISSUE_PENETRATION_ORDER {
            if remaining_penetration <= 0.0 {
                break;
            }

            if let Some(tissue) = part.tissues.get_mut(&tissue_type) {
                affected_tissues.push(tissue_type);

                // Доля урона, приходящаяся на эту ткань
                let depth_in_tissue = remaining_penetration.min(tissue.thickness);
                let damage_ratio = depth_in_tissue / tissue.thickness;

                // Разрушение ткани
                let tissue_damage = (final_damage as f32 * damage_ratio) / tissue.max_integrity;
                tissue.apply_damage(tissue_damage);

                // Расчет боли и кровотечения от этой ткани
                total_pain += tissue.pain_receptors * damage_ratio * 10.0; // Базовый множитель

                // Специфика кровотечения по типу урона
                let bleed_modifier = match penetration.tip_type {
                    WoundType::Cutting => 2.0,
                    WoundType::Piercing => 1.5,
                    WoundType::Blunt => 0.3,
                    _ => 1.0,
                };
                total_bleeding_rate += tissue.bleeding_rate * damage_ratio * bleed_modifier;

                remaining_penetration -= depth_in_tissue;
            }
        }

        // 4. Определение тяжести раны (Severity)
        let severity = if part.hp <= -part.max_hp {
            WoundSeverity::Missing
        } else if part.hp <= 0 {
            WoundSeverity::FunctionLoss
        } else if affected_tissues.contains(&crate::anatomy::TissueType::Bone) {
            WoundSeverity::Broken
        } else if final_damage > part.max_hp / 2 {
            WoundSeverity::Inhibited
        } else {
            WoundSeverity::Minor
        };

        // 5. Создание физической Раны (Wound)
        let wound = Wound {
            wound_type: penetration.tip_type,
            severity,
            affected_tissues,
            depth: effective_depth - remaining_penetration,
            bleeding_rate: total_bleeding_rate,
            pain_level: total_pain,
            infection_risk: if penetration.tip_type == WoundType::Burning {
                0.0
            } else {
                0.15
            },
            created_at: 0.0, // TODO: Передать текущее время
        };

        part.wounds.push(wound);

        // Обновление флагов BRP
        if severity >= WoundSeverity::Missing && !part.injuries.contains(&crate::Injury::Severed) {
            part.injuries.push(crate::Injury::Severed);
            part.is_destroyed = true;
        } else if severity >= WoundSeverity::FunctionLoss
            && !part.injuries.contains(&crate::Injury::Fractured)
        {
            part.injuries.push(crate::Injury::Fractured);
            part.is_useless = true;
        }

        DamageResult::Hit {
            damage_dealt: final_damage,
            bleeding_added: total_bleeding_rate,
            pain_caused: total_pain,
        }
    }
}
