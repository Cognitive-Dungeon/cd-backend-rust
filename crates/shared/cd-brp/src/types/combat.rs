use serde::{Deserialize, Serialize};

use crate::{HandednessReq, WeaponClass};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CombatPhase {
    Statements,
    Powers,
    Action,
    Resolution,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct StrikeRank(pub u8);

impl StrikeRank {
    #[inline]
    pub const fn new(val: u8) -> Self {
        Self(val)
    }

    #[inline]
    pub const fn get(self) -> u8 {
        self.0
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
