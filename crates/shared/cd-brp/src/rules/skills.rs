// src/rules/skills.rs
use crate::domain::chars::CharacteristicBlock;
use crate::types::{GameSessionConfig, SkillCategory, SkillType};

/// Определяет категорию навыка для расчета бонусов (Стр. 68 рулбука).
pub const fn get_skill_category(skill: &SkillType) -> SkillCategory {
    use SkillCategory::*;
    use SkillType::*;

    match skill {
        // Combat
        Artillery(_) | Brawl | EnergyWeapon(_) | Firearm(_) | Grapple | HeavyWeapon(_)
        | MartialArts | MeleeWeapon(_) | MissileWeapon(_) | Parry(_) | Shield => Combat,

        // Communication
        Bargain | Command | Disguise | Etiquette(_) | FastTalk | LanguageOwn(_)
        | LanguageOther(_) | Perform(_) | Persuade | Status | Teach => Communication,

        // Manipulation
        Art(_) | Craft(_) | Demolition | FineManipulation | Repair(_) | SleightOfHand => {
            Manipulation
        }

        // Mental
        Appraise | FirstAid | Gaming | Knowledge(_) | Literacy | Medicine | Psychotherapy
        | Science(_) | Strategy | TechnicalSkill(_) => Mental,

        // Perception
        Insight | Listen | Navigate | Research | Sense | Spot | Track => Perception,

        // Physical
        Climb | Dodge | Drive(_) | Fly | Hide | Jump | Projection | Ride(_) | Stealth | Swim
        | Throw => Physical,
    }
}

/// Вычисляет базовый шанс навыка (Base Chance, Стр. 69-70 рулбука).
/// Учитывает динамические шансы, зависящие от статов (Dodge, Language).
pub fn get_base_chance(
    skill: &SkillType,
    stats: &CharacteristicBlock,
    config: &GameSessionConfig,
) -> u16 {
    use SkillType::*;

    match skill {
        // --- Динамические базовые шансы ---
        Dodge => stats.dex.get().saturating_mul(2),
        Projection => stats.dex.get().saturating_mul(2),
        LanguageOwn(_) => {
            if config.use_education_stat {
                if let Some(edu) = stats.edu {
                    return edu.get().saturating_mul(5);
                }
            }
            stats.int.get().saturating_mul(5)
        }
        Gaming => stats.int.get().saturating_add(stats.pow.get()),

        // --- Статичные базовые шансы ---
        Appraise | Repair(_) | Persuade | Status => 15,
        Bargain | Command | Craft(_) | Art(_) | Etiquette(_) | FastTalk | FineManipulation
        | Insight | Medicine | Perform(_) | Ride(_) | SleightOfHand | TechnicalSkill(_) => 5,
        Brawl | Grapple | Spot | Swim | Throw | Listen | Jump => 25,
        Climb => 40,
        FirstAid => 30,
        Hide | Navigate | Sense | Stealth | Teach | Track => 10,
        Demolition | Disguise | MartialArts | Pilot(_) | Science(_) | Strategy => 1,

        // Для оружия и щитов база зависит от конкретного чертежа предмета,
        // поэтому на уровне "голого" персонажа база равна 0.
        // Она прибавится позже, когда персонаж возьмет предмет в руки
        Artillery(_) | EnergyWeapon(_) | Firearm(_) | HeavyWeapon(_) | MeleeWeapon(_)
        | MissileWeapon(_) | Parry(_) | Shield => 0,

        // Опциональные и специфичные
        LanguageOther(_) | Literacy => 0,
        Fly => stats.dex.get() / 2, // По умолчанию для не-крылатых (стр. 81)
        Psychotherapy => 1,
    }
}
