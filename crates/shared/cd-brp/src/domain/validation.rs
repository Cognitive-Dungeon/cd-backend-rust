// src/domain/validation.rs
use std::collections::BTreeMap;
use thiserror::Error;

use crate::domain::character::{CharacterProfile, DerivedStats};
use crate::domain::chars::CharacteristicBlock;
use crate::rules::character::{
    calculate_derived_stats, calculate_personal_budget, calculate_professional_budget,
};
use crate::types::{
    GameSessionConfig, PowerLevel, ProfessionId, SkillRating, SkillType, WealthLevel,
};

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Professional points limit exceeded. Allowed: {allowed}, Used: {used}")]
    ProfessionalBudgetExceeded { allowed: u16, used: u16 },

    #[error("Personal points limit exceeded. Allowed: {allowed}, Used: {used}")]
    PersonalBudgetExceeded { allowed: u16, used: u16 },

    #[error("Skill {0:?} is not allowed for the selected profession")]
    IllegalProfessionalSkill(SkillType),

    #[error("Skill rating for {0:?} exceeds the cap of {1}")]
    SkillCapExceeded(SkillType, u16),
}

/// "Сырые" намерения игрока по созданию персонажа (То, что приходит по сети)
pub struct CharacterDraft {
    pub name: String,
    pub age: u16,
    pub profession: ProfessionId,
    pub wealth: WealthLevel,
    pub stats: CharacteristicBlock,
    pub invested_prof_points: BTreeMap<SkillType, u16>,
    pub invested_personal_points: BTreeMap<SkillType, u16>,
}

/// Чистая функция-компилятор.
/// Принимает сырые данные и конфиг сервера, прогоняет через рулбук и возвращает готового персонажа или ошибку.
pub fn validate_and_build(
    draft: CharacterDraft,
    config: &GameSessionConfig,
    allowed_prof_skills: &[SkillType], // Справочник должен передаваться снаружи!
) -> Result<CharacterProfile, ValidationError> {
    // 1. Проверка бюджетов
    let prof_budget = calculate_professional_budget(config.power_level, draft.stats.edu);
    let used_prof: u16 = draft.invested_prof_points.values().sum();
    if used_prof > prof_budget {
        return Err(ValidationError::ProfessionalBudgetExceeded {
            allowed: prof_budget,
            used: used_prof,
        });
    }

    let pers_budget = calculate_personal_budget(
        draft.stats.int,
        config.power_level,
        config.use_increased_personal_skills,
    );
    let used_pers: u16 = draft.invested_personal_points.values().sum();
    if used_pers > pers_budget {
        return Err(ValidationError::PersonalBudgetExceeded {
            allowed: pers_budget,
            used: used_pers,
        });
    }

    // 2. Проверка легальности профессиональных навыков
    for skill in draft.invested_prof_points.keys() {
        if !allowed_prof_skills.contains(skill) {
            return Err(ValidationError::IllegalProfessionalSkill(*skill));
        }
    }

    // 3. Вычисление итоговых шансов и проверка капов (Стр. 25)
    let max_skill_cap = match config.power_level {
        PowerLevel::Normal => 75,
        PowerLevel::Heroic => 90,
        PowerLevel::Epic => 101,
        PowerLevel::Superhuman => u16::MAX,
    };

    let mut final_skills = BTreeMap::new();
    let mut all_skills: Vec<SkillType> = draft
        .invested_prof_points
        .keys()
        .chain(draft.invested_personal_points.keys())
        .copied()
        .collect();
    all_skills.sort();
    all_skills.dedup();

    for skill in all_skills {
        let prof_pts = draft.invested_prof_points.get(&skill).copied().unwrap_or(0);
        let pers_pts = draft
            .invested_personal_points
            .get(&skill)
            .copied()
            .unwrap_or(0);

        // TODO: Добавить Base Chance и Category Bonus здесь, используя вынесенные в `rules` функции.
        let base_chance = 0;

        let total = base_chance + prof_pts + pers_pts;

        if total > max_skill_cap {
            return Err(ValidationError::SkillCapExceeded(skill, max_skill_cap));
        }

        final_skills.insert(skill, SkillRating::new(total));
    }

    // 4. Формирование финального агрегата (Успех!)
    let derived = calculate_derived_stats(&draft.stats, config);

    Ok(CharacterProfile {
        name: draft.name,
        age: draft.age,
        profession: draft.profession,
        wealth: draft.wealth,
        base_stats: draft.stats,
        derived_stats: derived,
        skills: final_skills,
    })
}
