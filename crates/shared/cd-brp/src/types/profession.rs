// Замените содержимое src/types/profession.rs
use crate::types::core::{DefId, SkillCategory};
use crate::types::skills::SkillType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SkillRequirement {
    /// Строго конкретный навык (уже закодирован в Enum)
    Specific { skill: SkillType },
    /// Выбор N специализаций из категории навыков (напр. 2 навыка из Art)
    AnyOfCategory { category: SkillCategory, count: u8 },
    /// Выбор N навыков из жестко заданного списка
    ChooseFrom { count: u8, list: Vec<SkillType> },
    /// Выбор ровно одного навыка из вариантов
    OneOf { list: Vec<SkillType> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProfessionId(pub DefId);
