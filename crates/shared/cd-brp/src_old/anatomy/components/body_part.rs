use bevy::ecs::component::Component;
use bevy::reflect::Reflect;
use enum_map::EnumMap;
use enum_map::enum_map;
use serde::{Deserialize, Serialize};

use crate::anatomy::AnatomyEvent;
use crate::anatomy::BLEED_MOD_BLUNT;
use crate::anatomy::BLEED_MOD_CUTTING;
use crate::anatomy::BLEED_MOD_DEFAULT;
use crate::anatomy::BLEED_MOD_PIERCING;
use crate::anatomy::PAIN_BASE_MULTIPLIER;
use crate::anatomy::TISSUE_PENETRATION_ORDER;
use crate::anatomy::WoundSeverity;
use crate::anatomy::WoundType;
use crate::anatomy::types::location::calculate_location_hp;
use crate::{
    HitLocationType, Injury,
    anatomy::{Organ, OrganType, TissueLayer, TissueType, Wound},
};

/// Результат симуляции проникновения урона через ткани
pub struct TissuePenetrationResult {
    pub affected_tissues: Vec<TissueType>,
    pub total_bleeding_rate: f32,
    pub total_pain: f32,
    pub remaining_penetration: f32,
    pub events: smallvec::SmallVec<[AnatomyEvent; 4]>,
}

#[derive(Debug, Clone, Component, Serialize, Deserialize, Reflect)]
pub struct BodyPart {
    // Legacy BRP поля (для совместимости с action_points.rs)
    pub location: HitLocationType,
    pub hp: i32,
    pub max_hp: i32,
    pub armor: i32,
    pub injuries: Vec<Injury>,

    // Симулятивные поля
    #[reflect(ignore)]
    pub tissues: EnumMap<TissueType, Option<TissueLayer>>,
    #[reflect(ignore)]
    pub organs: EnumMap<OrganType, Option<Organ>>,
    pub wounds: Vec<Wound>,
    pub is_useless: bool,
    pub is_destroyed: bool,
}

impl Default for BodyPart {
    fn default() -> Self {
        Self::new(10, crate::HitLocationType::Chest, 0)
    }
}

impl BodyPart {
    pub fn new(total_hp: i32, location: HitLocationType, armor: i32) -> Self {
        let max_hp = calculate_location_hp(total_hp, location);

        let mut tissues = enum_map! { _ => None };

        tissues[TissueType::Skin] = Some(TissueLayer::default_skin());
        tissues[TissueType::Muscle] = Some(TissueLayer::default_muscle(max_hp));

        // Кости для конечностей/головы
        if !matches!(location, HitLocationType::Abdomen) {
            tissues[TissueType::Bone] = Some(TissueLayer {
                tissue_type: TissueType::Bone,
                thickness: 8.0,
                integrity: 1.0,
                max_integrity: 1.0,
                pain_receptors: 5.0,
                bleeding_rate: 0.0,
            });
        }

        Self {
            location,
            hp: max_hp,
            max_hp,
            armor,
            injuries: Vec::new(),
            tissues,
            organs: enum_map! { _ => None },
            wounds: Vec::new(),
            is_useless: false,
            is_destroyed: false,
        }
    }

    pub fn is_useless(&self) -> bool {
        self.is_useless || self.hp <= 0 || self.injuries.contains(&Injury::Severed)
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed || self.hp <= -self.max_hp || self.injuries.contains(&Injury::Severed)
    }

    /// Пропускает урон через слои тканей и возвращает агрегированный результат
    pub fn process_tissue_penetration(
        &mut self,
        damage: f32,
        initial_penetration: f32,
        wound_type: WoundType,
    ) -> TissuePenetrationResult {
        let mut result = TissuePenetrationResult {
            affected_tissues: Vec::new(),
            total_bleeding_rate: 0.0,
            total_pain: 0.0,
            remaining_penetration: initial_penetration,
            events: smallvec::SmallVec::new(),
        };

        let bleed_modifier = match wound_type {
            WoundType::Cutting => BLEED_MOD_CUTTING,
            WoundType::Piercing => BLEED_MOD_PIERCING,
            WoundType::Blunt => BLEED_MOD_BLUNT,
            _ => BLEED_MOD_DEFAULT,
        };

        for tissue_type in TISSUE_PENETRATION_ORDER {
            if result.remaining_penetration <= 0.0 {
                break;
            }

            if let Some(tissue) = &mut self.tissues[tissue_type] {
                result.affected_tissues.push(tissue_type);

                let depth_in_tissue = result.remaining_penetration.min(tissue.thickness);
                let damage_ratio = depth_in_tissue / tissue.thickness;

                // Разрушение ткани
                let tissue_damage = (damage * damage_ratio) / tissue.max_integrity;
                tissue.apply_damage(tissue_damage);

                result.events.push(AnatomyEvent::TissueDamaged {
                    location: self.location,
                    tissue: tissue_type,
                    damage_ratio,
                });

                if tissue_type == TissueType::Bone && tissue.is_destroyed() {
                    result.events.push(AnatomyEvent::BoneFractured {
                        location: self.location,
                    });
                }

                if matches!(tissue_type, TissueType::Artery | TissueType::Vein) {
                    result.events.push(AnatomyEvent::VesselRuptured {
                        location: self.location,
                        bleed_rate: tissue.bleeding_rate,
                    });
                }

                // Агрегация боли и крови
                result.total_pain += tissue.pain_receptors * damage_ratio * PAIN_BASE_MULTIPLIER;
                result.total_bleeding_rate += tissue.bleeding_rate * damage_ratio * bleed_modifier;
                result.remaining_penetration -= depth_in_tissue;
            }
        }

        result
    }

    /// Определяет тяжесть раны по правилам симуляции
    pub fn evaluate_wound_severity(
        &self,
        final_damage: i32,
        affected_tissues: &[TissueType],
    ) -> WoundSeverity {
        if self.hp <= -self.max_hp {
            WoundSeverity::Missing
        } else if self.hp <= 0 {
            WoundSeverity::FunctionLoss
        } else if affected_tissues.contains(&TissueType::Bone) {
            WoundSeverity::Broken
        } else if final_damage > self.max_hp / 2 {
            WoundSeverity::Inhibited
        } else {
            WoundSeverity::Minor
        }
    }
}
