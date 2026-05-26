// src/domain/validation.rs
use std::collections::BTreeMap;
use thiserror::Error;

use crate::domain::character::CharacterProfile;
use crate::domain::chars::CharacteristicBlock;
use crate::progression::ExperienceChecks;
use crate::rules::character::{
    calculate_derived_stats, calculate_personal_budget, calculate_professional_budget,
};

use crate::types::{
    GameSessionConfig, PowerLevel, ProfessionId, SkillRating, SkillType, WealthLevel,
};
use crate::{BodyPlan, DefId, calculate_category_bonus};

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
    pub body_plan: BodyPlan,
    pub native_language: DefId, // ID родного языка/диалекта расы
    pub invested_prof_points: BTreeMap<SkillType, u16>,
    pub invested_personal_points: BTreeMap<SkillType, u16>,
}

/// Список навыков, которые гарантированно есть у КАЖДОГО персонажа.
/// Сюда не входят навыки со специализациями (Art, Craft),
/// так как они добавляются только если игрок вложил в них очки.
const UNIVERSAL_BASE_SKILLS: &[SkillType] = &[
    SkillType::Appraise,
    SkillType::Bargain,
    SkillType::Brawl,
    SkillType::Climb,
    SkillType::Command,
    SkillType::Demolition,
    SkillType::Disguise,
    SkillType::Dodge,
    SkillType::FastTalk,
    SkillType::FineManipulation,
    SkillType::FirstAid,
    SkillType::Fly, // Имеет смысл для всех, даже если нет крыльев (база 1/2 DEX)
    SkillType::Gaming,
    SkillType::Grapple,
    SkillType::Hide,
    SkillType::Insight,
    SkillType::Jump,
    SkillType::Listen,
    SkillType::Medicine,
    SkillType::Navigate,
    SkillType::Persuade,
    SkillType::Projection,
    SkillType::Psychotherapy,
    SkillType::Research,
    SkillType::Sense,
    SkillType::SleightOfHand,
    SkillType::Spot,
    SkillType::Stealth,
    SkillType::Strategy,
    SkillType::Swim,
    SkillType::Teach,
    SkillType::Throw,
    SkillType::Track,
];

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

    // 3. СБОРКА ВСЕХ НАВЫКОВ
    // Нам нужно объединить:
    // 1. Универсальные навыки
    // 2. Инвестированные навыки (включая специализации вроде Craft(Blacksmithing))
    // 3. Родной язык и грамотность (они зависят от draft.native_language)
    let mut all_skills_to_process: Vec<SkillType> = UNIVERSAL_BASE_SKILLS.to_vec();

    all_skills_to_process.push(SkillType::LanguageOwn(draft.native_language));
    all_skills_to_process.push(SkillType::Literacy(draft.native_language));

    // Собираем все уникальные навыки, в которые игрок вложил хотя бы 1 очко
    all_skills_to_process.extend(draft.invested_prof_points.keys().copied());
    all_skills_to_process.extend(draft.invested_personal_points.keys().copied());

    // Убираем дубликаты (чтобы не считать один и тот же скилл дважды, если в него вложили очки)
    all_skills_to_process.sort();
    all_skills_to_process.dedup();

    // 4. Вычисление итоговых шансов и проверка капов (Стр. 25)
    let max_skill_cap = match config.power_level {
        PowerLevel::Normal => 75,
        PowerLevel::Heroic => 90,
        PowerLevel::Epic => 101,
        PowerLevel::Superhuman => u16::MAX,
    };

    let ctx = crate::rules::skills::BaseChanceContext {
        stats: &draft.stats,
        body_plan: draft.body_plan,
        config,
    };

    let mut final_skills = BTreeMap::new();

    // 4. ВЫЧИСЛЕНИЕ ФИНАЛЬНЫХ ЗНАЧЕНИЙ
    for skill in all_skills_to_process {
        let prof_pts = draft.invested_prof_points.get(&skill).copied().unwrap_or(0);
        let pers_pts = draft
            .invested_personal_points
            .get(&skill)
            .copied()
            .unwrap_or(0);

        // 3.1. Получаем базу (например, 40% для Climb или DEXx2 для Dodge)
        let base_chance = skill.base_chance(&ctx);

        // 3.2. Вычисляем бонус категории (может быть отрицательным!)
        let category_bonus = if config.use_skill_category_bonuses {
            calculate_category_bonus(skill.category(), &draft.stats, config)
        } else {
            0
        };

        // 3.3. Суммируем всё в i32, чтобы безопасно отработать отрицательные бонусы
        let total =
            (base_chance as i32) + (prof_pts as i32) + (pers_pts as i32) + (category_bonus as i32);

        // Защита от ухода в минус (например, огромный штраф категории при нулевой прокачке)
        let total_clamped = total.clamp(0, u16::MAX as i32) as u16;

        // 3.4. Проверка на кап (Ограничение уровня силы кампании)
        if total_clamped > max_skill_cap {
            return Err(ValidationError::SkillCapExceeded(skill, max_skill_cap));
        }

        // Если итоговый шанс больше 0 (или если это базовый навык), сохраняем в профиль
        // (Опциональная проверка: `if total_clamped > 0` - но для базовых лучше сохранить и 0, чтобы было явно)
        final_skills.insert(skill, SkillRating::new(total_clamped));
    }

    // 6. Формирование финального агрегата
    let derived = calculate_derived_stats(&draft.stats, config);

    Ok(CharacterProfile {
        name: draft.name,
        age: draft.age,
        profession: draft.profession,
        wealth: draft.wealth,
        base_stats: draft.stats,
        derived_stats: derived,
        skills: final_skills,
        body_plan: draft.body_plan,
        experience_checks: ExperienceChecks::default(),
    })
}
