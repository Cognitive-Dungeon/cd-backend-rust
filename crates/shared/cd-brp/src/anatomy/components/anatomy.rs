use bevy::ecs::component::Component;
use bevy::reflect::Reflect;
use enum_map::{EnumMap, enum_map};
use serde::{Deserialize, Serialize};

use crate::anatomy::types::location::{is_critical, iter_by_criticality};
use crate::anatomy::{AnatomyEvent, DamageInput, SimulationOutput};
use crate::{
    BodyPart, HitLocationType,
    anatomy::{
        DamageResult, PenetrationProfile, SubstancePool, TISSUE_PENETRATION_ORDER, VitalStats,
        Wound, WoundSeverity, WoundType,
    },
};

#[derive(Debug, Clone, Component, Serialize, Deserialize, Reflect)]
pub struct Anatomy {
    pub total_hp: i32,
    pub current_hp: i32,
    #[reflect(ignore)]
    pub parts: EnumMap<HitLocationType, BodyPart>,
    pub substances: SubstancePool,
    pub vitals: VitalStats,
}

impl Anatomy {
    pub fn new_humanoid(total_hp: i32, siz: i32) -> Self {
        let parts = enum_map! {
            loc => BodyPart::new(total_hp, loc, 0)
        };

        let mut substance_pool = SubstancePool::default_human();
        substance_pool.max_blood_volume = SubstancePool::calculate_blood_volume_by_siz(siz);
        substance_pool.blood_volume = SubstancePool::calculate_blood_volume_by_siz(siz);
        Self {
            total_hp,
            current_hp: total_hp,
            parts,
            substances: substance_pool,
            vitals: VitalStats::default(),
        }
    }

    pub fn is_alive(&self) -> bool {
        if self.current_hp <= 0 {
            return false;
        }

        for loc in iter_by_criticality() {
            if is_critical(loc) && self.parts[loc].is_destroyed() {
                return false;
            }
        }

        true
    }

    /// Legacy BRP-метод (возвращает i32 для совместимости)
    pub fn apply_damage(&mut self, location: HitLocationType, raw_damage: i32) -> i32 {
        let profile = PenetrationProfile::blunt();
        self.apply_damage_detailed(DamageInput {
            location,
            raw_damage,
            profile,
            timestamp_secs: (0.0),
        })
        .damage_result
        .damage_dealt()
    }

    /// Новый симулятивный метод проникновения через ткани
    pub fn apply_damage_detailed(&mut self, input: DamageInput) -> SimulationOutput {
        let mut events = smallvec::SmallVec::new();
        let part = &mut self.parts[input.location];

        // 1. Броня защищает
        let effective_depth = input.profile.effective_depth(part.armor, 1.0);
        let actual_damage = (input.raw_damage - part.armor).max(0);

        if actual_damage == 0 || effective_depth <= 0.0 {
            return SimulationOutput {
                damage_result: DamageResult::Blocked,
                events,
            };
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

            if let Some(tissue) = &mut part.tissues[tissue_type] {
                affected_tissues.push(tissue_type);

                // Доля урона, приходящаяся на эту ткань
                let depth_in_tissue = remaining_penetration.min(tissue.thickness);
                let damage_ratio = depth_in_tissue / tissue.thickness;

                // Разрушение ткани
                let tissue_damage = (final_damage as f32 * damage_ratio) / tissue.max_integrity;
                tissue.apply_damage(tissue_damage);

                let location = input.location;
                events.push(AnatomyEvent::TissueDamaged {
                    location,
                    tissue: tissue_type,
                    damage_ratio,
                });

                if tissue_type == crate::anatomy::TissueType::Bone && tissue.is_destroyed() {
                    events.push(AnatomyEvent::BoneFractured { location });
                }

                if matches!(
                    tissue_type,
                    crate::anatomy::TissueType::Artery | crate::anatomy::TissueType::Vein
                ) {
                    events.push(AnatomyEvent::VesselRuptured {
                        location,
                        bleed_rate: tissue.bleeding_rate,
                    });
                }

                // Расчет боли и кровотечения от этой ткани
                total_pain += tissue.pain_receptors * damage_ratio * 10.0; // Базовый множитель

                // Специфика кровотечения по типу урона
                let bleed_modifier = match input.profile.tip_type {
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

        let location = input.location;
        events.push(AnatomyEvent::WoundInflicted { location, severity });

        // 5. Создание физической Раны (Wound)
        let wound = Wound {
            wound_type: input.profile.tip_type,
            severity,
            affected_tissues,
            depth: effective_depth - remaining_penetration,
            bleeding_rate: total_bleeding_rate,
            pain_level: total_pain,
            infection_risk: if input.profile.tip_type == WoundType::Burning {
                0.0
            } else {
                0.15
            },
            created_at: input.timestamp_secs,
        };

        part.wounds.push(wound);

        // Обновление флагов BRP
        if severity >= WoundSeverity::Missing && !part.injuries.contains(&crate::Injury::Severed) {
            part.injuries.push(crate::Injury::Severed);
            part.is_destroyed = true;
            events.push(AnatomyEvent::LimbSevered { location });
        } else if severity >= WoundSeverity::FunctionLoss
            && !part.injuries.contains(&crate::Injury::Fractured)
        {
            part.injuries.push(crate::Injury::Fractured);
            part.is_useless = true;
        }

        if total_bleeding_rate > 0.0 {
            events.push(AnatomyEvent::BloodSpilled {
                location,
                amount_ml: total_bleeding_rate * 2.0,
            });
        }

        let res = DamageResult::Hit {
            damage_dealt: final_damage,
            bleeding_added: total_bleeding_rate,
            pain_caused: total_pain,
        };

        SimulationOutput {
            damage_result: res,
            events,
        }
    }
}
