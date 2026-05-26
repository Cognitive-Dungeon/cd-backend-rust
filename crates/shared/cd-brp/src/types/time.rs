use serde::{Deserialize, Serialize};

use crate::CombatRounds;

/// Семантическая длительность времени в BRP (Стр. 30, 48).
/// Позволяет движку (MMO/VTT) самому решать, как мапить это на реальное время,
/// тики сервера или пошаговые раунды.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "unit", content = "value", rename_all = "snake_case")]
#[derive(Default)]
pub enum BrpDuration {
    /// Мгновенно (урон от меча, огненный шар)
    #[default]
    Instantaneous,
    /// Фракция раунда (Используется для задержек инициативы)
    StrikeRanks(u8),
    /// Боевой раунд (По рулбуку = 12 секунд)
    CombatRounds(u32),
    /// Минуты
    Minutes(u32),
    /// Ход (Turn, По рулбуку = 5 минут вне боя)
    Turns(u32),
    /// Часы
    Hours(u32),
    /// Дни
    Days(u32),
    /// Недели (Стандартный цикл естественного лечения, стр. 34)
    Weeks(u32),
    /// Месяцы
    Months(u32),
    /// Годы
    Years(u32),

    // --- ОСОБЫЕ СОСТОЯНИЯ ---
    /// Работает, пока поддерживается активно (концентрация/трата маны каждый ход)
    ActiveMaintenance,
    /// Длится бесконечно, пока не будет снято/вылечено (например, проклятие или болезнь)
    Indefinite,
    /// Навсегда меняет цель (например, мутация)
    Permanent,
}

impl BrpDuration {
    pub const fn is_finite(&self) -> bool {
        !matches!(
            self,
            Self::ActiveMaintenance | Self::Indefinite | Self::Permanent
        )
    }
    pub const fn is_instantaneous(&self) -> bool {
        matches!(self, Self::Instantaneous)
    }

    pub fn to_combat_rounds(&self) -> Option<u64> {
        self.to_seconds()
            .map(|s| s / Self::SECONDS_PER_ROUND as u64)
    }

    /// Требует тактического (раундового) разрешения
    pub const fn is_tactical(&self) -> bool {
        matches!(
            self,
            Self::Instantaneous | Self::StrikeRanks(_) | Self::CombatRounds(_)
        )
    }

    /// Требует концентрации/ресурса каждый ход
    pub const fn requires_upkeep(&self) -> bool {
        matches!(self, Self::ActiveMaintenance)
    }

    /// Не снимается без явного dispel
    pub const fn is_persistent(&self) -> bool {
        matches!(self, Self::Indefinite | Self::Permanent)
    }
}

impl BrpDuration {
    /// Стр. 257: "Combat round = 12 seconds"
    pub const SECONDS_PER_ROUND: u32 = 12;
    /// Стр. 257: "Turn = five minutes (25 combat rounds)"
    pub const SECONDS_PER_TURN: u32 = 300;
    pub const ROUNDS_PER_TURN: u32 = 25; // производная, но книга называет явно

    /// Стр. 257: "an hour is 12 turns" (Movement Rates Table)
    pub const TURNS_PER_HOUR: u32 = 12;

    const SECONDS_PER_MINUTE: u64 = 60;
    const SECONDS_PER_HOUR: u64 = 3_600;
    const SECONDS_PER_DAY: u64 = 86_400;
    const SECONDS_PER_WEEK: u64 = 604_800;
    const SECONDS_PER_MONTH: u64 = 2_592_000;
    const SECONDS_PER_YEAR: u64 = 31_536_000;

    pub fn normalize(self) -> Self {
        match self {
            Self::CombatRounds(n) if n % Self::ROUNDS_PER_TURN == 0 => {
                Self::Turns(n / Self::ROUNDS_PER_TURN)
            }
            Self::Minutes(n) if n % 5 == 0 => Self::Turns(n / 5),
            Self::Turns(n) if n % Self::TURNS_PER_HOUR == 0 => {
                Self::Hours(n / Self::TURNS_PER_HOUR)
            }
            Self::Hours(n) if n % 24 == 0 => Self::Days(n / 24),
            Self::Days(n) if n % 7 == 0 => Self::Weeks(n / 7),
            other => other,
        }
    }

    pub fn to_seconds(&self) -> Option<u64> {
        let s = match self {
            Self::Instantaneous => 0,
            Self::CombatRounds(n) => *n as u64 * Self::SECONDS_PER_ROUND as u64,
            Self::Minutes(n) => *n as u64 * Self::SECONDS_PER_MINUTE,
            Self::Turns(n) => *n as u64 * Self::SECONDS_PER_TURN as u64,
            Self::Hours(n) => *n as u64 * Self::SECONDS_PER_HOUR,
            Self::Days(n) => *n as u64 * Self::SECONDS_PER_DAY,
            Self::Weeks(n) => *n as u64 * Self::SECONDS_PER_WEEK,
            Self::Months(n) => *n as u64 * Self::SECONDS_PER_MONTH, // 30 дней
            Self::Years(n) => *n as u64 * Self::SECONDS_PER_YEAR,

            // StrikeRanks — субъединица раунда, конкретная длительность
            // зависит от издания (RQ3: 10 SR/раунд, другие: 12 или 25)
            // поэтому здесь осознанно None
            Self::StrikeRanks(_) => return None,

            Self::ActiveMaintenance | Self::Indefinite | Self::Permanent => return None,
        };
        Some(s)
    }

    /// Универсальный метод: использует кэш, если передан, иначе вычисляет
    pub fn to_seconds_with_cache(
        &self,
        cache: Option<&std::sync::OnceLock<Option<u64>>>,
    ) -> Option<u64> {
        if let Some(c) = cache {
            *c.get_or_init(|| self.to_seconds())
        } else {
            self.to_seconds()
        }
    }

    pub const fn from_seconds(s: u64) -> Self {
        match s {
            0 => Self::Instantaneous,
            s if s % Self::SECONDS_PER_YEAR == 0 => {
                Self::Years((s / Self::SECONDS_PER_YEAR) as u32)
            }
            s if s % Self::SECONDS_PER_MONTH == 0 => {
                Self::Months((s / Self::SECONDS_PER_MONTH) as u32)
            }
            s if s % Self::SECONDS_PER_WEEK == 0 => {
                Self::Weeks((s / Self::SECONDS_PER_WEEK) as u32)
            }
            s if s % Self::SECONDS_PER_DAY == 0 => Self::Days((s / Self::SECONDS_PER_DAY) as u32),
            s if s % Self::SECONDS_PER_HOUR == 0 => {
                Self::Hours((s / Self::SECONDS_PER_HOUR) as u32)
            }
            s if s % Self::SECONDS_PER_TURN as u64 == 0 => {
                Self::Turns((s / Self::SECONDS_PER_TURN as u64) as u32)
            }
            s if s % Self::SECONDS_PER_ROUND as u64 == 0 => {
                Self::CombatRounds((s / Self::SECONDS_PER_ROUND as u64) as u32)
            }
            s => Self::Minutes((s / Self::SECONDS_PER_MINUTE) as u32),
        }
    }

    pub fn to_ticks(&self, ticks_per_second: u32) -> Option<u64> {
        self.to_seconds()
            .map(|s| s.saturating_mul(ticks_per_second as u64))
    }

    /// Конвертирует длительность в тики на этапе компиляции.
    ///
    /// Работает только для значений, которые можно выразить в секундах.
    /// Возвращает `None` для `StrikeRanks` и особых состояний.
    ///
    /// # Пример использования
    /// ```
    /// const EFFECT_DURATION: BrpDuration = BrpDuration::CombatRounds(5);
    /// const TICKS_60HZ: Option<u64> = EFFECT_DURATION.as_ticks(60); // Some(3600)
    /// ```
    pub const fn as_ticks(self, ticks_per_second: u32) -> Option<u64> {
        // Вспомогательная константная функция для умножения с проверкой переполнения
        const fn safe_mul_secs(secs: u64, tps: u32) -> Option<u64> {
            secs.checked_mul(tps as u64)
        }

        let seconds = match self {
            Self::Instantaneous => 0,

            Self::CombatRounds(n) => {
                // 12 секунд за раунд — константа, умножение безопасно для u32 → u64
                n as u64 * Self::SECONDS_PER_ROUND as u64
            }

            Self::Minutes(n) => n as u64 * Self::SECONDS_PER_MINUTE,
            Self::Turns(n) => n as u64 * Self::SECONDS_PER_TURN as u64,
            Self::Hours(n) => n as u64 * Self::SECONDS_PER_HOUR,
            Self::Days(n) => n as u64 * Self::SECONDS_PER_DAY,
            Self::Weeks(n) => n as u64 * Self::SECONDS_PER_WEEK,
            Self::Months(n) => n as u64 * Self::SECONDS_PER_MONTH, // 30 дней
            Self::Years(n) => n as u64 * Self::SECONDS_PER_YEAR,

            // Не можем вычислить на compile-time: зависит от редакции правил
            Self::StrikeRanks(_) => return None,

            // Особые состояния не имеют фиксированной длительности
            Self::ActiveMaintenance | Self::Indefinite | Self::Permanent => return None,
        };

        safe_mul_secs(seconds, ticks_per_second)
    }

    /// Проверяет, истекла ли длительность к текущему тику
    pub fn has_expired(&self, elapsed_ticks: u64, ticks_per_second: u32) -> bool {
        match self.to_ticks(ticks_per_second) {
            Some(total) => elapsed_ticks >= total,
            None => false, // бесконечные/особые не истекают
        }
    }

    /// Возвращает остаток времени в тиках
    pub fn remaining_ticks(&self, elapsed_ticks: u64, ticks_per_second: u32) -> Option<u64> {
        self.to_ticks(ticks_per_second)
            .map(|total| total.saturating_sub(elapsed_ticks))
    }

    /// Складывает через секунды, возвращает нормализованный результат.
    /// None если хотя бы один операнд — особое состояние.
    pub fn saturating_add(self, other: Self) -> Option<Self> {
        let a = self.to_seconds()?;
        let b = other.to_seconds()?;
        Some(Self::from_seconds(a.saturating_add(b)).normalize())
    }

    pub fn saturating_sub(self, other: Self) -> Option<Self> {
        let a = self.to_seconds()?;
        let b = other.to_seconds()?;
        Some(Self::from_seconds(a.saturating_sub(b)).normalize())
    }
}

impl std::fmt::Display for BrpDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Instantaneous => write!(f, "мгновенно"),
            Self::StrikeRanks(n) => write!(f, "{n} SR"),
            Self::CombatRounds(n) => write!(f, "{n} раунд(ов)"),
            Self::Minutes(n) => write!(f, "{n} мин"),
            Self::Turns(n) => write!(f, "{n} ход(ов)"),
            Self::Hours(n) => write!(f, "{n} ч"),
            Self::Days(n) => write!(f, "{n} дн"),
            Self::Weeks(n) => write!(f, "{n} нед"),
            Self::Months(n) => write!(f, "{n} мес"),
            Self::Years(n) => write!(f, "{n} лет"),
            Self::ActiveMaintenance => write!(f, "поддержание"),
            Self::Indefinite => write!(f, "бессрочно"),
            Self::Permanent => write!(f, "навсегда"),
        }
    }
}

impl std::ops::Mul<u32> for BrpDuration {
    type Output = Option<Self>; // None для особых состояний

    fn mul(self, rhs: u32) -> Option<Self> {
        match self {
            Self::StrikeRanks(n) => Some(Self::StrikeRanks(n.saturating_mul(rhs as u8))),
            Self::CombatRounds(n) => Some(Self::CombatRounds(n.saturating_mul(rhs))),
            Self::Minutes(n) => Some(Self::Minutes(n.saturating_mul(rhs))),
            Self::Turns(n) => Some(Self::Turns(n.saturating_mul(rhs))),
            Self::Hours(n) => Some(Self::Hours(n.saturating_mul(rhs))),
            Self::Days(n) => Some(Self::Days(n.saturating_mul(rhs))),
            Self::Weeks(n) => Some(Self::Weeks(n.saturating_mul(rhs))),
            Self::Months(n) => Some(Self::Months(n.saturating_mul(rhs))),
            Self::Years(n) => Some(Self::Years(n.saturating_mul(rhs))),
            // Instantaneous * N = Instantaneous, умножение бессмысленно но не ошибка
            Self::Instantaneous => Some(Self::Instantaneous),
            // Особые состояния не масштабируются
            _ => None,
        }
    }
}

// Для суммирования коллекций длительностей
impl std::iter::Sum for BrpDuration {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::Instantaneous, |acc, d| {
            acc.saturating_add(d).unwrap_or(Self::Indefinite)
        })
    }
}

impl Ord for BrpDuration {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Особые состояния считаем "бесконечно большими"
        match (self.to_seconds(), other.to_seconds()) {
            (Some(a), Some(b)) => a.cmp(&b),
            (None, None) => std::cmp::Ordering::Equal,
            (None, _) => std::cmp::Ordering::Greater,
            (_, None) => std::cmp::Ordering::Less,
        }
    }
}

impl PartialOrd for BrpDuration {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Сравниваем только если оба конвертируются в секунды
        // StrikeRanks и особые состояния → None (несравнимы)
        Some(self.cmp(other))
    }
}

// From-конвертеры для частых случаев
impl From<CombatRounds> for BrpDuration {
    fn from(rounds: CombatRounds) -> Self {
        Self::CombatRounds(rounds.get())
    }
}

// Внутренняя структура для хранения кэша
#[derive(Debug, Clone)]
pub struct BrpDurationCached {
    duration: BrpDuration,
    seconds_cache: std::sync::OnceLock<Option<u64>>,
}

impl BrpDurationCached {
    pub const fn new(duration: BrpDuration) -> Self {
        Self {
            duration,
            seconds_cache: std::sync::OnceLock::new(),
        }
    }

    /// Кэшированная версия to_seconds()
    pub fn to_seconds_cached(&self) -> Option<u64> {
        *self
            .seconds_cache
            .get_or_init(|| self.duration.to_seconds())
    }

    /// Доступ к исходному значению
    pub const fn duration(&self) -> BrpDuration {
        self.duration
    }
}

impl std::fmt::Display for BrpDurationCached {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.duration.fmt(f)
    }
}
