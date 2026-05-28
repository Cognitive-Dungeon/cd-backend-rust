use serde::{Deserialize, Serialize};

use crate::{CombatRounds, HandednessReq, WeaponClass};

/// Фазы боевого раунда в строгом порядке BRP (Стр. 48).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default, Hash,
)]
#[serde(rename_all = "snake_case")]
pub enum CombatPhase {
    /// 1. Игроки и ГМ заявляют, что будут делать в этом раунде.
    #[default]
    StatementOfIntent,
    /// 2. Разрешение движения, разговоров и небоевых навыков (First Aid, Climb).
    MovementAndNonCombat,
    /// 3. Разрешение атак и магии. Требует пошагового отсчета Strike Ranks (от 1 до 10).
    MeleeMissileAndMagic,
    /// 4. Фаза очистки (применение урона от ядов, кровотечения, огня).
    ResolutionAndBookkeeping,
}

impl std::fmt::Display for CombatPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::StatementOfIntent => "Заявление намерений",
            Self::MovementAndNonCombat => "Движение и не-боевые действия",
            Self::MeleeMissileAndMagic => "Атаки и магия",
            Self::ResolutionAndBookkeeping => "Разрешение и учёт",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CombatAction {
    Move,
    Attack,
    NonCombatAction,
    Engage,
    Disengage,
    Parry,
    Dodge,
    FightDefensively,
    Speak,
}

impl CombatAction {
    #[inline]
    #[must_use]
    pub const fn category(&self) -> CombatActionCategory {
        match self {
            Self::Move | Self::Engage | Self::Disengage => CombatActionCategory::Movement,
            Self::Attack | Self::FightDefensively => CombatActionCategory::Attack,
            Self::NonCombatAction | Self::Speak => CombatActionCategory::NonCombat,
            Self::Parry | Self::Dodge => CombatActionCategory::Defense,
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_reaction(&self) -> bool {
        matches!(self, Self::Dodge | Self::Parry { .. })
    }
}

/// Классификация действий для определения, в какой фазе они должны разрешаться.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CombatActionCategory {
    Movement,  // Бег, ходьба, отступление
    NonCombat, // Применение навыка (FirstAid, PickLock)
    Attack,    // Выстрел, удар мечом
    Magic,     // Каст заклинания
    Defense,   // Dodge, Parry (Разрешаются вне фаз, в ответ на атаку!)
}

impl CombatActionCategory {
    /// Битовая позиция категории в PhaseActionMask.
    /// Явная привязка — порядок вариантов в enum не имеет значения.
    pub const fn bit(self) -> u16 {
        match self {
            Self::Movement => 1 << 0,
            Self::NonCombat => 1 << 1,
            Self::Attack => 1 << 2,
            Self::Magic => 1 << 3,
            Self::Defense => 1 << 4,
        }
    }
}

/// Категории дистанции дистанционного боя (стр. 60-61 рулбука).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RangeCategory {
    /// В упор (до DEX метров). Шанс становится Easy (x2).
    PointBlank,
    /// Базовая дальность оружия. Шанс обычный.
    BaseRange,
    /// До двойной базовой дальности. Шанс становится Difficult (x1/2).
    DoubleBaseRange,
    /// Свыше двойной дальности. Выстрел невозможен для обычного оружия.
    BeyondDoubleBaseRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FumbleTableType {
    MeleeAttack,
    MeleeParry,
    MissileAttack,
    NaturalWeapon,
}

/// Спецэффекты оружия при Special/Critical успехах (стр. 55)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecialSuccessEffect {
    Bleeding,   // Кровотечение
    Crushing,   // Оглушение и двойной Damage Modifier
    Entangling, // Опутывание (сети, лассо)
    Impaling,   // Пронзание (двойной урон кубиков)
    Knockback,  // Отбрасывание
    None,
}

/// Типы энергии/урона (Energy Types, стр. 136 + таблицы оружия)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DamageType {
    #[default]
    Kinetic, // Обычный физический урон (пули, мечи, падения)
    FireHeat,   // Огонь, плазма
    ColdFrost,  // Холод
    Electric,   // Электричество, молнии
    Sonic,      // Звук
    LaserLight, // Лазеры, свет
    Magnetic,   // Магнетизм
    Emp,        // ЭМИ (урон только по технике)
    Radiation,  // Радиация
    Antimatter, // Антиматерия
    Biological, // Яды, болезни
    Stun,       // Нелетальный шок
    Darkness,   // Тьма
    Gravity,    // Гравитация
    Wind,       // Ветер
}

/// Строгий тип для Strike Rank (Инициатива, Стр. 35, 48-49).
/// Чем МЕНЬШЕ значение, тем быстрее действует персонаж (1 - очень быстро, 10 - медленно).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default, Hash,
)]
#[serde(transparent)]
pub struct StrikeRank(u8);

impl StrikeRank {
    /// Максимальное значение Strike Rank в одном боевом раунде (стр. 48 BRP: UGE).
    pub const MAX: u8 = 10;

    #[inline]
    pub const fn new(val: u8) -> Self {
        Self(val)
    }

    #[inline]
    pub const fn get(self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn saturating_add(self, rhs: u8) -> Self {
        Self(self.0.saturating_add(rhs))
    }

    #[inline]
    pub const fn saturating_sub(self, rhs: u8) -> Self {
        Self(self.0.saturating_sub(rhs))
    }

    #[inline]
    pub const fn clamp(self, min: u8, max: u8) -> Self {
        Self({
            assert!(min <= max);
            if self.0 < min {
                min
            } else if self.0 > max {
                max
            } else {
                self.0
            }
        })
    }
}

/// Модификатор к Strike Rank (магия, черты, состояние, снаряжение).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrikeRankModifier {
    /// Дельта к базовому SR: положительное значение = бонус (ходишь раньше),
    /// отрицательное = штраф (ходишь позже).
    pub delta: StrikeRankShift,

    /// Источник модификатора — для отладки и логирования.
    pub source: crate::domain::combat::ModifierSource,
}

impl StrikeRankModifier {
    /// Применяет модификатор к базовому Strike Rank.
    ///
    /// Положительный `delta` = бонус (уменьшает SR, ходишь раньше).
    /// Отрицательный `delta` = штраф (увеличивает SR, ходишь позже).
    #[must_use]
    pub const fn apply(&self, base_sr: StrikeRank) -> StrikeRank {
        self.delta.apply_to(base_sr)
    }
}

/// Направление модификатора инициативы.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrikeRankShiftDir {
    /// Уменьшает SR (персонаж ходит БЫСТРЕЕ / РАНЬШЕ).
    Faster,
    /// Увеличивает SR (персонаж ходит МЕДЛЕННЕЕ / ПОЗЖЕ).
    Slower,
}

/// Строгий тип для модификатора (сдвига) инициативы.
/// Заменяет "сырой" i8. Гарантирует правильное применение к StrikeRank.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrikeRankShift {
    pub direction: StrikeRankShiftDir,
    pub amount: u8,
}

impl StrikeRankShift {
    /// Создает бонус к скорости (сдвигает SR ближе к 1).
    pub const fn faster(amount: u8) -> Self {
        Self {
            direction: StrikeRankShiftDir::Faster,
            amount,
        }
    }

    /// Создает штраф к скорости (сдвигает SR ближе к 10).
    pub const fn slower(amount: u8) -> Self {
        Self {
            direction: StrikeRankShiftDir::Slower,
            amount,
        }
    }

    /// Безопасно применяет сдвиг к текущему StrikeRank.
    #[must_use]
    pub const fn apply_to(self, base_sr: StrikeRank) -> StrikeRank {
        let val = base_sr.get();
        let new_val = match self.direction {
            // Быстрее = число меньше (вычитаем)
            StrikeRankShiftDir::Faster => val.saturating_sub(self.amount),
            // Медленнее = число больше (прибавляем)
            StrikeRankShiftDir::Slower => val.saturating_add(self.amount),
        };

        // В BRP инициатива не может быть быстрее 1 и медленнее 10 (в одном раунде)
        StrikeRank::new(new_val).clamp(1, StrikeRank::MAX)
    }
}

/// Каким предметом или действием персонаж пытается защититься.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefenseMethod {
    /// Уклонение (использует только тело).
    Dodge,
    /// Парирование одноручным или двуручным оружием (меч, копье).
    WeaponParry {
        class: WeaponClass,
        handedness: HandednessReq,
        /// Использовалось ли это оружие для атаки в текущем раунде (Strike Rank).
        used_to_attack_this_round: bool,
    },
    /// Парирование щитом (отдельный предмет с огромным запасом прочности).
    ShieldParry {
        /// В BRP щиты можно использовать для пассивного прикрытия.
        is_actively_blocking: bool,
    },
}

/// Определяет тип входящей атаки для проверки легальности защиты.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncomingAttackType {
    /// Удар в ближнем бою (Меч, Кулак, Когти)
    Melee,
    /// Бросок/Выстрел мускульной силой (Копье, Стрела, Нож)
    ThrownOrArrow,
    /// Огнестрельное оружие или Энергетический луч
    FirearmOrEnergy,
    /// Атака по площади (Взрыв, Дыхание Дракона)
    AreaOfEffect,
}

impl IncomingAttackType {
    /// Проверяет, физически возможно ли парировать данную атаку указанным предметом.
    /// Эта функция должна вызываться до любых бросков кубиков!
    #[must_use]
    pub const fn is_parry_legal(self, defender_weapon_class: WeaponClass) -> bool {
        match self {
            Self::Melee => {
                // В ближнем бою можно парировать любым оружием или щитом.
                // Исключения составляют луки/арбалеты (ими сложно отбить меч, но возможно по опциональным правилам).
                true
            }
            Self::ThrownOrArrow => {
                // Стрелы и дротики можно отбивать ТОЛЬКО щитом (Стр. 64-65).
                matches!(defender_weapon_class, WeaponClass::Shield)
            }
            Self::FirearmOrEnergy => {
                // Пули и лазеры парировать невозможно.
                false
            }
            Self::AreaOfEffect => {
                // От взрыва нельзя защититься парированием (но иногда можно укрыться за ОГРОМНЫМ щитом,
                // здесь базовая реализация - false).
                // TODO: Когда появятся классы щитов добавить сюда
                false
            }
        }
    }

    /// Проверяет, физически возможно ли уклониться (Dodge) от атаки.
    #[must_use]
    pub const fn is_dodge_legal(self) -> bool {
        match self {
            // Уклониться можно от ударов, стрел и даже выстрелов (уворот с линии огня).
            Self::Melee | Self::ThrownOrArrow | Self::FirearmOrEnergy => true,

            // От АоЕ (взрыв гранаты в замкнутом помещении) часто уклониться нельзя,
            // либо уклонение дает только уменьшение урона (оставляем на усмотрение GM/сервера, базово - сложно).
            Self::AreaOfEffect => false,
        }
    }
}

/// Событие изменения состояния боя — детерминированное и сериализуемое.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseTransition {
    /// Начался новый раунд с указанным номером.
    RoundStarted(CombatRounds),

    /// Произошёл переход между фазами.
    PhaseChanged {
        /// Исходная фаза.
        from: CombatPhase,
        /// Целевая фаза.
        to: CombatPhase,
    },

    /// Продвинут тик инициативы в фазе атак.
    StrikeRankAdvanced {
        /// Предыдущее значение (1..=9).
        from: u8,
        /// Новое значение (2..=10).
        to: u8,
    },

    /// Завершён раунд с указанным номером.
    RoundEnded(CombatRounds),
}

impl std::fmt::Display for PhaseTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RoundStarted(r) => write!(f, "Начат раунд {}", r.get()),
            Self::RoundEnded(r) => write!(f, "Раунд {} завершён", r.get()),
            Self::StrikeRankAdvanced { to, .. } => write!(f, "Тик инициативы: {to}/10"),
            Self::PhaseChanged { to, .. } => write!(f, "{to}"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Система очередей действий с приоритетами
// ─────────────────────────────────────────────────────────────────────────────

/// Категория приоритета действия для разрешения коллизий в одном тике.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionPriority {
    /// Реакции, контратаки, прерывания — обрабатываются первыми.
    Interrupt,
    /// Обычные заявленные действия.
    Normal,
    /// Отложенные действия (Hold Action) — обрабатываются последними.
    Delayed,
}

/// Результат проверки легальности действия в текущем контексте.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionLegality {
    /// Разрешено ли действие.
    pub allowed: bool,
    /// Причина отказа (если `allowed == false`) — для подсказок в UI / логах.
    pub reason: Option<super::error::LegalityReason>,
}
