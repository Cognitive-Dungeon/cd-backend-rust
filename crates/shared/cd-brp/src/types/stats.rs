use crate::{Cha, Con, Dex, Edu, Int, Pow, Str};

use super::markers::CharacteristicMarker;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;

/// Обобщенный строгий тип для значения конкретной характеристики.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Stat<T: CharacteristicMarker> {
    value: u16,
    #[serde(skip)]
    _marker: PhantomData<T>,
}

impl<T: CharacteristicMarker> Stat<T> {
    #[inline]
    pub const fn new(value: u16) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub const fn get(self) -> u16 {
        self.value
    }

    /// Вычисляет процентный шанс для броска характеристики с заданным множителем (стр. 23).
    /// Используется для задач разной сложности (например, ×1 для Impossible, ×3 для Hard).
    #[inline]
    pub const fn chance_multiplier(self, multiplier: u16) -> SkillRating {
        SkillRating::new(self.value.saturating_mul(multiplier))
    }

    /// Стандартный бросок характеристики (Average = ×5).
    /// Генерирует шанс (SkillRating), который напрямую совместим с `resolve_skill`.
    #[inline]
    pub const fn x5_chance(self) -> SkillRating {
        self.chance_multiplier(5)
    }
}

impl Stat<Str> {
    /// Effort roll (STR × 5). Бросок Усилия (поднять, толкнуть, выломать дверь).
    #[inline]
    pub const fn effort_chance(self) -> SkillRating {
        self.x5_chance()
    }
}

impl Stat<Con> {
    /// Stamina roll (CON × 5). Бросок Выносливости (сопротивление ядам, болезням, удушью).
    #[inline]
    pub const fn stamina_chance(self) -> SkillRating {
        self.x5_chance()
    }
}

impl Stat<Int> {
    /// Idea roll (INT × 5). Бросок Идеи (осознание, догадки, риск сойти с ума).
    #[inline]
    pub const fn idea_chance(self) -> SkillRating {
        self.x5_chance()
    }
}

impl Stat<Pow> {
    /// Luck roll (POW × 5). Бросок Удачи (везение, случайные обстоятельства).
    #[inline]
    pub const fn luck_chance(self) -> SkillRating {
        self.x5_chance()
    }
}

impl Stat<Dex> {
    /// Agility roll (DEX × 5). Бросок Подвижности (баланс, реакция, избегание падения).
    #[inline]
    pub const fn agility_chance(self) -> SkillRating {
        self.x5_chance()
    }
}

impl Stat<Cha> {
    /// Charisma roll (CHA × 5). Бросок Харизмы (первое впечатление, удача в общении).
    #[inline]
    pub const fn charisma_chance(self) -> SkillRating {
        self.x5_chance()
    }
}

impl Stat<Edu> {
    /// Know roll (EDU × 5). Бросок Знаний (вспомнить общий факт).
    #[inline]
    pub const fn know_chance(self) -> SkillRating {
        self.x5_chance()
    }
}

impl<T: CharacteristicMarker> fmt::Display for Stat<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.value, T::ABBREVIATION)
    }
}

/// Строгий тип для рейтинга навыка (от 0 до бесконечности).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct SkillRating(u16);

impl SkillRating {
    pub const ZERO: Self = Self(0);

    #[inline]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn get(self) -> u16 {
        self.0
    }

    #[inline(always)]
    pub const fn is_automatic(self) -> bool {
        self.get() == u16::MAX
    }

    #[inline(always)]
    pub const fn is_impossible(self) -> bool {
        self.get() == 0
    }
}
