use bevy::ecs::component::Component;
use bevy::reflect::Reflect;
use enum_map::{EnumMap, enum_map};
use serde::{Deserialize, Serialize};

use crate::anatomy::types::location::{is_critical, iter_by_criticality};
use crate::anatomy::{
    AnatomyEvent, BLOOD_SPILLED_VISUAL_MULTIPLIER, BRP_MAX_PART_DAMAGE_MULTIPLIER, DamageInput,
    SimulationOutput,
};
use crate::{
    BodyPart, HitLocationType,
    anatomy::{DamageResult, PenetrationProfile, SubstancePool, VitalStats, Wound, WoundSeverity},
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

    /// Вспомогательная функция для обновления флагов BRP (Injury)
    fn update_brp_injuries(
        &mut self,
        location: HitLocationType,
        severity: WoundSeverity,
        events: &mut smallvec::SmallVec<[AnatomyEvent; 8]>,
    ) {
        let part = &mut self.parts[location];

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
        let max_possible_damage = part.max_hp * BRP_MAX_PART_DAMAGE_MULTIPLIER;
        let final_damage = actual_damage.min(max_possible_damage);

        part.hp -= final_damage;
        self.current_hp -= final_damage;

        // 3. Пробитие тканей (Tissue Penetration)
        let tissue_sim = part.process_tissue_penetration(
            final_damage as f32,
            effective_depth,
            input.profile.tip_type,
        );
        events.extend(tissue_sim.events);

        // 4. Определение тяжести раны (Severity)
        let severity = part.evaluate_wound_severity(final_damage, &tissue_sim.affected_tissues);
        events.push(AnatomyEvent::WoundInflicted {
            location: input.location,
            severity,
        });

        // 5. Создание физической Раны (Wound)
        let wound = Wound::new_simulated(
            input.profile.tip_type,
            severity,
            tissue_sim.affected_tissues,
            effective_depth - tissue_sim.remaining_penetration,
            tissue_sim.total_bleeding_rate,
            tissue_sim.total_pain,
            input.timestamp_secs,
        );
        part.wounds.push(wound);

        // 6. Обновление BRP-статусов (ампутации и переломы)
        self.update_brp_injuries(input.location, severity, &mut events);

        // 7. Визуальные события (кровь)
        if tissue_sim.total_bleeding_rate > 0.0 {
            events.push(AnatomyEvent::BloodSpilled {
                location: input.location,
                amount_ml: tissue_sim.total_bleeding_rate * BLOOD_SPILLED_VISUAL_MULTIPLIER,
            });
        }

        SimulationOutput {
            damage_result: DamageResult::Hit {
                damage_dealt: final_damage,
                bleeding_added: tissue_sim.total_bleeding_rate,
                pain_caused: tissue_sim.total_pain,
            },
            events,
        }
    }

    /// Полный цикл пересчета физиологического состояния
    pub fn process_vitals_tick(&mut self, delta_secs: f32) {
        if self.vitals.state == crate::anatomy::CharacterState::Dead {
            return;
        }

        if !self.is_alive() {
            self.vitals.transition_to_dead();
            return;
        }

        // 1. Сбор агрегированных данных из всех ран
        let (total_bleeding, total_pain) = self.aggregate_wound_effects();

        // 2. Обновление пула веществ (истекание кровью)
        self.substances
            .update_blood_loss(total_bleeding, delta_secs);

        // 3. Пересчет шока, сознания и общих состояний
        self.vitals.recalculate_state(total_pain, &self.substances);
    }

    /// Собирает суммарную кровопотерю и боль по всем частям тела
    fn aggregate_wound_effects(&self) -> (f32, f32) {
        let mut total_bleeding = 0.0;
        let mut total_pain = 0.0;

        for part in self.parts.values() {
            for wound in &part.wounds {
                if wound.is_active() {
                    total_bleeding += wound.bleeding_rate;
                    total_pain += wound.pain_level;
                }
            }
        }
        (total_bleeding, total_pain)
    }
}
