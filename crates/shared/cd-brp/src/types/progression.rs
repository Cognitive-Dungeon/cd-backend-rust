// В crates/shared/cd-brp/src/types/character.rs (Или создай progression.rs, если хочешь)

use crate::types::SkillType;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Записи об успешном применении навыков (Experience Checks, стр. 45-47).
/// В BRP за одну сессию/приключение навык может получить только ОДНУ галочку (Check),
/// независимо от того, сколько раз он был успешно применен.
/// Поэтому мы используем BTreeSet (множество уникальных значений).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExperienceChecks {
    /// Список навыков, ожидающих броска на улучшение.
    pub pending_checks: BTreeSet<SkillType>,
}

impl ExperienceChecks {
    /// Добавляет навык в список на улучшение.
    /// В MMO сервере вызывай эту функцию только когда бросок навыка был:
    /// 1. Успешным (Success, Special или Critical).
    /// 2. Сделан в стрессовой/важной ситуации (не при тренировке на манекене).
    pub fn mark_skill(&mut self, skill: SkillType) {
        self.pending_checks.insert(skill);
    }

    /// Проверяет, имеет ли навык отметку.
    pub fn has_check(&self, skill: &SkillType) -> bool {
        self.pending_checks.contains(skill)
    }

    /// Очищает все отметки (вызывается после того, как все проверки на рост были сделаны).
    pub fn clear(&mut self) {
        self.pending_checks.clear();
    }
}

/// Строгий тип для бонуса опыта (Experience Bonus).
/// По умолчанию равен ceil(INT / 2) (Стр. 34).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct ExperienceBonus(pub u16);

impl ExperienceBonus {
    pub const ZERO: Self = Self(0);

    #[inline]
    pub const fn new(val: u16) -> Self {
        Self(val)
    }

    #[inline]
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// Строгий тип для целевого порога мастеров (Mastery Target).
/// Используется для проверок навыков, достигших 100%+. По умолчанию равен INTx5 (Стр. 46).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct MasteryTarget(pub u16);

impl MasteryTarget {
    #[inline]
    pub const fn new(val: u16) -> Self {
        Self(val)
    }

    #[inline]
    pub const fn get(self) -> u16 {
        self.0
    }
}
