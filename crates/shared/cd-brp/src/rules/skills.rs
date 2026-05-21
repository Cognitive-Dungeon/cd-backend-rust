use serde_json::error::Category;

use crate::{Dex, Edu, Int, KnowledgeType, Pow, Stat, VehicleCategory};
// src/rules/skills.rs
use crate::math::BrpFractions;
use crate::types::{SkillCategory, SkillType};

impl SkillType {
    /// Возвращает категорию навыка.
    pub const fn category(&self) -> SkillCategory {
        use SkillCategory::*;
        use SkillType::*;

        match self {
            // Combat
            Artillery(_) | Brawl | EnergyWeapon(_) | Firearm(_) | Grapple | HeavyWeapon(_)
            | MartialArts(_) | MeleeWeapon(_) | MissileWeapon(_) | Parry(_) | Shield(_) => Combat,

            // Communication
            Bargain | Command | Disguise | Etiquette(_) | FastTalk | LanguageOwn(_)
            | LanguageOther(_) | Perform(_) | Persuade | Status(_) | Teach => Communication,

            // Manipulation
            Art(_) | Craft(_) | Demolition | FineManipulation | HeavyMachine(_) | Repair(_)
            | SleightOfHand => Manipulation,

            // Mental
            Appraise | FirstAid | Gaming | Knowledge(_) | Literacy(_) | Medicine
            | Psychotherapy | Science(_) | Strategy | TechnicalSkill(_) => Mental,

            // Perception
            Insight | Listen | Navigate | Research | Sense | Spot | Track => Perception,

            // Physical
            Climb | Dodge | Drive(_) | Fly | Hide | Jump | Pilot(_) | Projection | Ride(_)
            | Stealth | Swim | Throw => Physical,
        }
    }

    /// Возвращает статичный базовый шанс навыка.
    /// Если навык вычисляется динамически (Dodge, Fly, LanguageOwn и т.д.), возвращает None.
    pub const fn static_base_chance(&self) -> Option<u16> {
        use SkillType::*;

        match self {
            // === ДИНАМИЧЕСКИЕ НАВЫКИ (Требуют характеристик персонажа) ===
            Dodge | Fly | Projection | Gaming | LanguageOwn(_) | Literacy(_) => None,

            // === УНИКАЛЬНЫЕ БАЗОВЫЕ ШАНСЫ (По рулбуку) ===
            Knowledge(knowledge_type) => match knowledge_type {
                // По правилам "Blasphemous Lore skill begins at 0%, not 05%"
                KnowledgeType::BlasphemousLore => Some(0),
                _ => Some(5),
            },

            LanguageOther(_) => Some(0), // Чужие языки всегда начинаются с 0, если не не приобретены

            Drive(vehicle_cat) => match vehicle_cat {
                // Наземный/Простой транспорт (20%)
                VehicleCategory::AnimalDrawn
                | VehicleCategory::Automobile
                | VehicleCategory::Motorcycle
                | VehicleCategory::Train
                | VehicleCategory::Hovercraft
                | VehicleCategory::LandSkimmer => Some(20),

                // Heavy/Military/Uncommon for Drive (1%)
                VehicleCategory::Tank
                | VehicleCategory::Mech
                | VehicleCategory::Boat
                | VehicleCategory::Ship
                | VehicleCategory::Submarine
                | VehicleCategory::AirVehicle
                | VehicleCategory::Spacecraft => Some(1),
            },

            // Во всем, что летает/плавает/ходит (Мехи), Pilot - это 1% (стр. 39)
            Pilot(_) => Some(1),

            // === ОРУЖИЕ (Зависит от чертежа, база здесь 0) ===
            Artillery(_) | EnergyWeapon(_) | Firearm(_) | HeavyWeapon(_) | MeleeWeapon(_)
            | MissileWeapon(_) | Parry(_) | Shield(_) => Some(0),

            // === 40% ===
            Climb => Some(40),

            // === 30% ===
            FirstAid => Some(30),

            // === 25% ===
            Brawl | Grapple | Jump | Listen | Research | Spot | Swim | Throw => Some(25),

            // === 15% ===
            Appraise | Persuade | Repair(_) | Status(_) => Some(15),

            // === 10% ===
            Hide | Navigate | Sense | Stealth | Teach | Track => Some(10),

            // === 05% ===
            Art(_) | Bargain | Command | Craft(_) | Etiquette(_) | FastTalk | FineManipulation
            | Insight | Medicine | Perform(_) | Ride(_) | SleightOfHand | TechnicalSkill(_) => {
                Some(5)
            }

            // === 01% ===
            Demolition | Disguise | HeavyMachine(_) | MartialArts(_) | Psychotherapy
            | Science(_) | Strategy => Some(1),
        }
    }
}

pub const fn calc_dodge_base(dex: Stat<Dex>) -> u16 {
    dex.get().saturating_mul(2)
}

pub const fn calc_projection_base(dex: Stat<Dex>) -> u16 {
    dex.get().saturating_mul(2)
}

pub fn calc_fly_base(dex: Stat<Dex>, has_wings: bool) -> u16 {
    if has_wings {
        dex.get().saturating_mul(4)
    } else {
        dex.get().half_ceil()
    }
}

pub const fn calc_gaming_base(int: Stat<Int>, pow: Stat<Pow>) -> u16 {
    int.get().saturating_add(pow.get())
}

pub fn calc_language_own_base(int: Stat<Int>, edu: Option<Stat<Edu>>, use_edu_rule: bool) -> u16 {
    let stat = if use_edu_rule {
        edu.map(|e| e.get()).unwrap_or_else(|| int.get())
    } else {
        int.get()
    };
    stat.saturating_mul(5)
}
