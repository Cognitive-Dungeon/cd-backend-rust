/// Макрос для генерации безопасных доменных пулов ресурсов (HP, MP, FP, SAN).
/// Все пулы в BRP используют знаковые числа (i16), так как уход в минус
/// означает специфические состояния (смерть, потеря сознания, истощение).
macro_rules! define_pool_type {
    // Вариант с валидацией диапазона: range = 0..=99
    ($name:ident, $doc:expr, range = $range:expr) => {
        define_pool_type!(@base $name, $doc);
        // Кастомный десериализатор с проверкой диапазона
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de> {
                let value = i16::deserialize(deserializer)?;
                if !$range.contains(&value) {
                    return Err(serde::de::Error::custom(
                        format!(
                            "{} value {} out of range {}",
                            stringify!($name), value, stringify!($range)
                        )
                    ));
                }
                Ok($name(value))
            }
        }
    };
    ($name:ident, $doc:expr, range = $range:expr, default = $default:expr) => {
        define_pool_type!(@base_with_default $name, $doc, $default);
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de> {
                let value = i16::deserialize(deserializer)?;
                if !$range.contains(&value) {
                    return Err(serde::de::Error::custom(format!(
                        "{} value {} out of range {}",
                        stringify!($name), value, stringify!($range)
                    )));
                }
                Ok($name(value))
            }
        }
    };
    // Вариант с запретом отрицательных значений
    ($name:ident, $doc:expr, no_negative) => {
        define_pool_type!(@base $name, $doc);
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de> {
                let value = i16::deserialize(deserializer)?;
                if value < 0 {
                    return Err(serde::de::Error::custom(
                        format!("{} cannot be negative, got {}", stringify!($name), value)
                    ));
                }
                Ok($name(value))
            }
        }
    };
    ($name:ident, $doc:expr, no_negative, default = $default:expr) => {
        define_pool_type!(@base_with_default $name, $doc, $default);
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de> {
                let value = i16::deserialize(deserializer)?;
                if value < 0 {
                    return Err(serde::de::Error::custom(format!(
                        "{} cannot be negative, got {}",
                        stringify!($name), value
                    )));
                }
                Ok($name(value))
            }
        }
    };
    ($name:ident, $doc:expr) => {
        define_pool_type!(@base $name, $doc);
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de> {
                i16::deserialize(deserializer).map($name)
            }
        }
    };
    ($name:ident, $doc:expr, default = $default:expr) => {
        define_pool_type!(@base_with_default $name, $doc, $default);
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de> {
                i16::deserialize(deserializer).map($name)
            }
        }
    };
    (@base_with_default $name:ident, $doc:expr, $default:expr) => {
        #[doc = $doc]
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            serde::Serialize,
            Hash,
        )]
        #[serde(transparent)]
        #[repr(transparent)]
        pub struct $name(i16);
        impl Default for $name {
            fn default() -> Self {
                Self($default)
            }
        }
        pool_common_impl!($name);
    };

    (@base $name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            serde::Serialize,
            Default,
            Hash,
        )]
        #[serde(transparent)]
        #[repr(transparent)]
        pub struct $name(i16);
        pool_common_impl!($name);
    };
}

macro_rules! pool_common_impl {
    ($name:ident) => {
        impl $name {
            /// Абсолютный ноль для данного пула.
            pub const ZERO: Self = Self(0);

            /// Создает новое значение пула.
            #[inline]
            pub const fn new(value: i16) -> Self {
                Self(value)
            }

            /// Возвращает внутреннее значение пула (только на чтение).
            #[inline(always)]
            pub const fn get(self) -> i16 {
                self.0
            }

            /// Возвращает true, если значение строго больше нуля.
            #[inline(always)]
            pub const fn is_positive(self) -> bool {
                self.0 > 0
            }

            /// Возвращает true, если значение <= 0 (частый триггер негативных состояний в BRP).
            #[inline(always)]
            pub const fn is_negative_or_zero(self) -> bool {
                self.0 <= 0
            }

            /// Ограничивает текущее значение заданным максимумом.
            /// Идеально для хилинга: `vitals.hp = (vitals.hp + heal).clamp_to_max(derived.max_hp);`
            #[inline]
            #[must_use]
            pub fn clamp_to_max(self, maximum: Self) -> Self {
                Self(self.0.min(maximum.0))
            }

            #[inline]
            #[must_use]
            pub fn clamp_to_min(self, minimum: Self) -> Self {
                Self(self.0.max(minimum.0))
            }

            /// Полный Clamp (от минимума до максимума).
            /// Полезно для SAN, который не может упасть ниже 0 и подняться выше 99.
            #[inline]
            #[must_use]
            pub fn clamp(self, minimum: Self, maximum: Self) -> Self {
                Self(self.0.clamp(minimum.0, maximum.0))
            }

            /// Безопасное умножение на целое число.
            #[inline]
            #[must_use]
            pub fn saturating_mul(self, multiplier: i16) -> Self {
                Self(self.0.saturating_mul(multiplier))
            }

            /// Проверяет, находится ли значение в диапазоне [min, max]
            #[inline]
            pub const fn in_range(self, min: Self, max: Self) -> bool {
                self.0 >= min.0 && self.0 <= max.0
            }

            /// Возвращает процент от максимума (0..=100). Полезно для прогресс-баров.
            #[inline]
            pub fn percentage_of(self, max: Self) -> u8 {
                if max.0 <= 0 {
                    return 0;
                }
                let current = self.0.max(0) as u32;
                ((current * 100) / max.0 as u32).min(100) as u8
            }

            /// Сложение с явной обработкой переполнения
            #[inline]
            pub fn try_add(self, rhs: Self) -> Option<Self> {
                self.0.checked_add(rhs.0).map(Self)
            }

            /// Вычитание с явной обработкой переполнения
            #[inline]
            pub fn try_sub(self, rhs: Self) -> Option<Self> {
                self.0.checked_sub(rhs.0).map(Self)
            }

            /// Умножение с явной обработкой переполнения
            #[inline]
            pub fn try_mul(self, rhs: i16) -> Option<Self> {
                self.0.checked_mul(rhs).map(Self)
            }
        }

        impl std::ops::Add for $name {
            type Output = Self;
            #[inline]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0.saturating_add(rhs.0))
            }
        }

        impl std::ops::Sub for $name {
            type Output = Self;
            #[inline]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0.saturating_sub(rhs.0))
            }
        }

        impl std::ops::Mul<i16> for $name {
            type Output = Self;
            #[inline]
            fn mul(self, rhs: i16) -> Self::Output {
                Self(self.0.saturating_mul(rhs))
            }
        }

        impl std::ops::AddAssign for $name {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                self.0 = self.0.saturating_add(rhs.0);
            }
        }

        impl std::ops::SubAssign for $name {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 = self.0.saturating_sub(rhs.0);
            }
        }

        impl std::ops::MulAssign<i16> for $name {
            #[inline]
            fn mul_assign(&mut self, rhs: i16) {
                self.0 = self.0.saturating_mul(rhs);
            }
        }

        impl std::ops::Neg for $name {
            type Output = Self;
            #[inline]
            fn neg(self) -> Self::Output {
                Self(self.0.saturating_neg())
            }
        }

        impl PartialEq<i16> for $name {
            #[inline]
            fn eq(&self, other: &i16) -> bool {
                self.0 == *other
            }
        }

        impl PartialOrd<i16> for $name {
            #[inline]
            fn partial_cmp(&self, other: &i16) -> Option<std::cmp::Ordering> {
                self.0.partial_cmp(other)
            }
        }

        impl From<u16> for $name {
            #[inline]
            fn from(val: u16) -> Self {
                // Безопасный каст из беззнакового (защита от переполнения i16)
                let safe_val = if val > i16::MAX as u16 {
                    i16::MAX
                } else {
                    val as i16
                };
                Self(safe_val)
            }
        }

        impl From<i16> for $name {
            #[inline]
            fn from(val: i16) -> Self {
                Self(val)
            }
        }

        impl From<u8> for $name {
            #[inline]
            fn from(val: u8) -> Self {
                Self(val as i16)
            }
        }

        impl From<i8> for $name {
            #[inline]
            fn from(val: i8) -> Self {
                Self(val as i16)
            }
        }

        impl std::iter::Sum<Self> for $name {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::ZERO, |acc, x| acc + x)
            }
        }

        impl<'a> std::iter::Sum<&'a Self> for $name {
            fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                iter.copied().sum()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_pool_type!(HitPoints, "Очки здоровья (HP). Могут быть < 0.");
define_pool_type!(PowerPoints, "Очки магии/энергии (MP).", default = 5);
define_pool_type!(
    FatiguePoints,
    "Очки усталости (FP). Могут уходить в минус до -MaxFP.",
    default = 10
);
define_pool_type!(
    SanityPoints,
    "Очки рассудка (SAN).",
    range = 0..=99,
    default = 99
);
