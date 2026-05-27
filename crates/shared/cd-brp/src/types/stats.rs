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
    /// Создает новую характеристику со значением 1.
    /// В BRP базовые характеристики живых существ не могут быть равны 0.
    /// Если нужно описывать нежить (CON 0), используй `new_unchecked`.
    #[inline]
    pub const fn new(value: u16) -> Self {
        Self {
            value: if value == 0 { 1 } else { value },
            _marker: PhantomData,
        }
    }

    /// Позволяет создать характеристику со значением 0 (для нежити, конструктов и т.д.).
    #[inline]
    pub const fn new_unchecked(value: u16) -> Self {
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

impl<T: CharacteristicMarker> std::ops::Add for Stat<T> {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new_unchecked(self.value.saturating_add(rhs.value))
    }
}

impl<T: CharacteristicMarker> std::ops::Add<u16> for Stat<T> {
    type Output = Self;
    #[inline]
    fn add(self, rhs: u16) -> Self::Output {
        Self::new_unchecked(self.value.saturating_add(rhs))
    }
}

impl<T: CharacteristicMarker> std::ops::Sub for Stat<T> {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        // При вычитании стат может упасть до 0. В BRP это означает смерть/паралич.
        Self::new_unchecked(self.value.saturating_sub(rhs.value))
    }
}

impl<T: CharacteristicMarker> std::ops::Sub<u16> for Stat<T> {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: u16) -> Self::Output {
        Self::new_unchecked(self.value.saturating_sub(rhs))
    }
}

impl<T: CharacteristicMarker> std::ops::AddAssign<u16> for Stat<T> {
    #[inline]
    fn add_assign(&mut self, rhs: u16) {
        self.value = self.value.saturating_add(rhs);
    }
}

impl<T: CharacteristicMarker> std::ops::SubAssign<u16> for Stat<T> {
    #[inline]
    fn sub_assign(&mut self, rhs: u16) {
        self.value = self.value.saturating_sub(rhs);
    }
}

impl<T: CharacteristicMarker> fmt::Display for Stat<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.value, T::ABBREVIATION)
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

/// Строгий тип для рейтинга навыка (от 0 до бесконечности).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct SkillRating(u16);

impl SkillRating {
    pub const ZERO: Self = Self(0);
    pub const AUTOMATIC: Self = Self(u16::MAX);

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

    #[inline]
    pub const fn saturating_add(self, rhs: u16) -> Self {
        Self(self.0.saturating_add(rhs))
    }

    /// Безопасное умножение (для модификаторов сложности)
    #[inline]
    pub const fn saturating_mul(self, rhs: u16) -> Self {
        Self(self.0.saturating_mul(rhs))
    }

    #[inline]
    pub const fn saturating_div(self, rhs: u16) -> Self {
        Self(self.0.saturating_div(rhs))
    }

    /// Безопасное деление (защита от деления на 0)
    #[inline]
    pub const fn checked_div(self, rhs: u16) -> Option<Self> {
        match self.0.checked_div(rhs) {
            Some(val) => Some(Self(val)),
            None => None,
        }
    }
}

impl crate::math::BrpFractions for SkillRating {
    #[inline]
    fn half_ceil(self) -> Self {
        Self(self.0.half_ceil())
    }

    #[inline]
    fn fifth_ceil(self) -> Self {
        Self(self.0.fifth_ceil())
    }

    #[inline]
    fn twentieth_ceil(self) -> Self {
        Self(self.0.twentieth_ceil())
    }
}

impl std::ops::Add for SkillRating {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl std::ops::Add<u16> for SkillRating {
    type Output = Self;

    #[inline]
    fn add(self, rhs: u16) -> Self::Output {
        Self(self.0.saturating_add(rhs))
    }
}

impl std::ops::Sub for SkillRating {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl std::ops::Sub<u16> for SkillRating {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: u16) -> Self::Output {
        Self(self.0.saturating_sub(rhs))
    }
}

impl std::ops::AddAssign for SkillRating {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_add(rhs.0);
    }
}

impl std::ops::SubAssign for SkillRating {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_sub(rhs.0);
    }
}

impl fmt::Display for SkillRating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_automatic() {
            write!(f, "Auto")
        } else {
            write!(f, "{}%", self.0)
        }
    }
}

impl std::str::FromStr for SkillRating {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clean = s.trim().to_lowercase();

        if clean == "auto" || clean == "automatic" {
            return Ok(Self::AUTOMATIC);
        }

        let numeric_part = clean.trim_end_matches('%').trim();
        match numeric_part.parse::<u16>() {
            Ok(val) => Ok(Self(val)),
            Err(_) => Err(format!("Invalid skill rating: '{}'", s)),
        }
    }
}

// Сериализуем как удобную строку с процентами (например, "65%") для читаемости
impl Serialize for SkillRating {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

// Десериализуем из числа ИЛИ строки.
// Позволяет писать в JSON: `"stealth": 65` или `"stealth": "65%"`
impl<'de> Deserialize<'de> for SkillRating {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Serde Visitor позволяет обрабатывать разные типы входящих данных
        struct SkillRatingVisitor;

        impl<'de> serde::de::Visitor<'de> for SkillRatingVisitor {
            type Value = SkillRating;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a u16 integer or a string like '65%'")
            }

            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
                if value > u16::MAX as u64 {
                    Err(E::custom(format!("SkillRating out of bounds: {}", value)))
                } else {
                    Ok(SkillRating::new(value as u16))
                }
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                <SkillRating as std::str::FromStr>::from_str(value).map_err(E::custom)
            }
        }

        deserializer.deserialize_any(SkillRatingVisitor)
    }
}

/// Строгий тип для аддитивного модификатора навыка (Situational Modifier, стр. 24).
/// Представляет фиксированные проценты (+20%, -15%), которые применяются к базовому шансу.
/// В отличие от `SkillRating`, может быть отрицательным.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct SkillModifier(i16);

impl SkillModifier {
    pub const ZERO: Self = Self(0);

    #[inline]
    pub const fn new(val: i16) -> Self {
        Self(val)
    }

    #[inline]
    pub const fn get(self) -> i16 {
        self.0
    }
}

// Позволяет складывать два модификатора вместе (например, +10% от баффа и -20% от раны)
impl std::ops::Add for SkillModifier {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}

// Позволяет применять модификатор к SkillRating
impl std::ops::Add<SkillModifier> for SkillRating {
    type Output = Self;
    #[inline]
    fn add(self, modifier: SkillModifier) -> Self::Output {
        let mod_val = modifier.get();
        if mod_val >= 0 {
            // Если модификатор положительный, используем наше сложение
            self + (mod_val as u16)
        } else {
            // Если отрицательный, превращаем в u16 и используем наше вычитание (с защитой от нуля)
            self - (mod_val.unsigned_abs())
        }
    }
}
