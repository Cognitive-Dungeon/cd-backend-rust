// domain/gear.rs

use serde::{Deserialize, Serialize};

use crate::{
    ArmorBurden, ArmorPoints, Currency, DefId, Dex, DiceExpression, EncumbrancePoints,
    HandednessReq, HitPoints, ItemCondition, ItemLegality, ItemQuality, ItemValue, Meters,
    RateOfFire, SkillRating, SkillType, SpecialSuccessEffect, Stat, Str, WeaponClass, WeaponLength,
    WeaponPropulsion,
};

/// Режим работы оружия (определяет специфику боевки)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum WeaponMode {
    Melee {
        length: WeaponLength, // Специфично для ближнего боя
    },
    Ranged {
        propulsion: WeaponPropulsion,
        rate_of_fire: RateOfFire,
        base_range: Meters,
    },
}

/// Тип предмета с его специфическими неизменяемыми характеристиками.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlueprintData {
    /// Оружие ближнего и дальнего боя.
    Weapon {
        // --- Общие свойства ЛЮБОГО оружия ---
        class: WeaponClass,
        handedness: HandednessReq,
        base_damage: DiceExpression,
        special_effect: SpecialSuccessEffect,
        can_parry: bool,

        // --- Специфика ближнего/дальнего боя ---
        mode: WeaponMode,
    },
    /// Броня (одежда, кольчуга, кевлар).
    Armor {
        /// В BRP броня поглощает урон. Может быть фиксированной (например, 6)
        /// или рандомной (например, 1D6 для кожаной куртки, стр. 114).
        armor_value: DiceExpression,
        burden: ArmorBurden,
    },
    /// Щиты (используются и для защиты, и для атаки).
    Shield {
        class: WeaponClass,   // Обычно WeaponClass::Shield
        length: WeaponLength, // Для атаки щитом
        base_damage: DiceExpression,
        armor_value: ArmorPoints,
    },
    /// Инструменты (Отмычки, аптечки).
    Tool {
        /// Какой навык этот предмет баффает (или делает возможным его использование).
        /// Например: `SkillType::FineManipulation` для отмычек.
        associated_skill: SkillType,
        /// Дает ли предмет бонус к шансу навыка (например, +20% за превосходный набор).
        skill_bonus_percent: SkillRating,
    },
    /// Расходники (Стрелы, патроны, зелья).
    Consumable {
        stack_size: u16,
        /// Уникальный тег для группировки (например, "ammo_9mm")
        consume_tag: String,
    },
    Misc, // Веревки, факелы и т.д.
}

/// Статический чертеж предмета.
/// Опционально: можно добавить #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemBlueprint {
    pub id: DefId,
    pub name_key: String, // Ключ локализации (например, "item.weapon.iron_sword")
    pub enc: EncumbrancePoints,
    pub max_hp: HitPoints,
    /// Реальная цена в минимальных монетах сеттинга (медные монеты, центы, кредиты).
    /// Задаётся вручную, если не удаётся получить цену от GM, применяющего правила BRP: UGE.
    /// Используется только как крайняя мера (fallback).
    /// Опциональное правило BRP: UGE.
    pub base_value: Currency,
    /// Семантическая ценность (для фильтров аукциона, цвета подсветки в UI: Зеленый, Синий)
    pub value_tier: ItemValue,
    pub legality: ItemLegality,
    pub min_str: Stat<Str>, // Требования
    pub min_dex: Stat<Dex>,
    pub data: BlueprintData,
}

impl ItemBlueprint {
    /// Проверяет, хватает ли персонажу статов для использования предмета.
    #[inline]
    #[must_use]
    pub const fn meets_requirements(&self, str_stat: Stat<Str>, dex_stat: Stat<Dex>) -> bool {
        str_stat.get() >= self.min_str.get() && dex_stat.get() >= self.min_dex.get()
    }

    /// Проверяет, является ли предмет оружием.
    #[inline]
    #[must_use]
    pub const fn is_weapon(&self) -> bool {
        matches!(self.data, BlueprintData::Weapon { .. })
    }
}

/// Экземпляр предмета в игровом мире (в инвентаре или надетый).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemInstance {
    pub blueprint_id: DefId,
    pub current_hp: HitPoints,
    pub condition: ItemCondition,
    pub quality: ItemQuality,
    /// Модификатор к навыку (например, +10% за Superior качество).
    /// Сервер сам устанавливает его при крафте/зачаровании.
    pub skill_modifier: SkillRating,
}

impl ItemInstance {
    /// Создается на основе данных из Blueprint.
    pub fn new(blueprint: &ItemBlueprint) -> Self {
        Self {
            blueprint_id: blueprint.id,
            current_hp: blueprint.max_hp,
            condition: ItemCondition::Intact,
            quality: ItemQuality::Average,
            skill_modifier: SkillRating::ZERO,
        }
    }

    /// Позволяет системе крафта создать кастомный предмет (Superior/Inferior).
    /// Здесь сервер может передать увеличенные HP или бонус к навыку,
    /// строго следуя решению ГМа или логике крафта MMO.
    pub const fn new_crafted(
        blueprint: &ItemBlueprint,
        quality: ItemQuality,
        custom_hp: HitPoints,
        skill_modifier: SkillRating,
    ) -> Self {
        Self {
            blueprint_id: blueprint.id,
            current_hp: custom_hp,
            condition: ItemCondition::Intact,
            quality,
            skill_modifier,
        }
    }

    /// Проверяет, сломан ли предмет.
    #[inline]
    pub const fn is_broken(&self) -> bool {
        matches!(self.condition, ItemCondition::Broken)
    }

    /// Наносит урон предмету. Возвращает `true`, если предмет был разрушен этим ударом.
    /// В BRP броня и оружие ломаются, когда их HP падает до 0 (Стр. 115).
    pub fn take_damage(&mut self, damage: HitPoints) -> bool {
        if self.is_broken() {
            return false; // Уже сломан
        }

        self.current_hp -= damage;

        if self.current_hp.is_negative_or_zero() {
            self.current_hp = HitPoints::ZERO;
            self.condition = ItemCondition::Broken;
            true // Предмет сломался
        } else {
            self.condition = ItemCondition::Damaged;
            false // Предмет поврежден, но еще "жив"
        }
    }
}

/// Трейт-провайдер для получения статических чертежей.
/// Позволяет библиотеке абстрагироваться от способа хранения данных (Memcache, БД, Bevy Assets).
pub trait BlueprintProvider {
    /// Возвращает ссылку на чертеж за O(1).
    /// Возвращает None, если ID невалиден (защита от краша сервера).
    fn get_item(&self, id: DefId) -> Option<&ItemBlueprint>;
}

/// Ошибка при доступе к чертежам.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlueprintError {
    NotFound(DefId),
}

/// Трейт, предоставляющий доступ к базе данных чертежей.
/// В MMO сервере это будет обертка над `HashMap<DefId, ItemBlueprint>` или In-Memory БД.
pub trait BlueprintRegistry {
    fn get_item(&self, id: DefId) -> Result<&ItemBlueprint, BlueprintError>;

    /// Проверяет, существует ли такой чертеж.
    fn item_exists(&self, id: DefId) -> bool {
        self.get_item(id).is_ok()
    }
}
