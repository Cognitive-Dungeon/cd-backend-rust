use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::DamageModifier;
use crate::domain::chars::CharacteristicBlock;
use crate::types::{
    HitPoints, MovementRate, PowerPoints, ProfessionId, SkillRating, SkillType, WealthLevel,
};

/// Производные параметры (Derived Characteristics, стр. 34).
/// В MMO их лучше кэшировать в профиле, но обновлять при изменении базовых статов.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedStats {
    pub max_hp: HitPoints,
    pub max_mp: PowerPoints,
    pub damage_modifier: DamageModifier,
    pub base_movement: MovementRate,
    pub experience_bonus: u16,
    pub major_wound_threshold: HitPoints,
}

/// Финальный Агрегат Персонажа (The Player Character).
/// Готов к сериализации в БД или трансформации в компоненты Bevy ECS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterProfile {
    pub name: String,
    pub age: u16,
    pub profession: ProfessionId,
    pub wealth: WealthLevel,

    pub base_stats: CharacteristicBlock,
    pub derived_stats: DerivedStats,

    /// BTreeMap используется для детерминированной сортировки и быстрого поиска.
    pub skills: BTreeMap<SkillType, SkillRating>,
}

impl CharacterProfile {
    /// Удобный геттер для шанса навыка (если навыка нет, возвращается 0).
    pub fn get_skill_rating(&self, skill: &SkillType) -> SkillRating {
        self.skills.get(skill).copied().unwrap_or(SkillRating::ZERO)
    }
}
