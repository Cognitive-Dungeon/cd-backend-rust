use serde::{Deserialize, Serialize};

/// Строгий тип для очков урона (до применения брони)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct DamagePoints(pub u16);

impl DamagePoints {
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

/// Строгий тип для ФАКТИЧЕСКИХ очков брони в момент удара
/// (уже после того, как кубики для Random Armor были брошены).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct ArmorPoints(pub u16);

impl ArmorPoints {
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
