use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WealthLevel {
    #[default]
    Destitute,
    Poor,
    Average,
    Affluent,
    Wealthy,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ItemQuality {
    Inferior,
    #[default]
    Average,
    Superior,
}

/// Доступность предмета (стр. 122)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ItemLegality {
    #[default]
    Standard,
    Free,       // Можно найти без усилий и денег
    Restricted, // Требует разрешения, незаконное владение = преступление
}

/// Способ приведения оружия в действие (влияет на расчет Damage Modifier)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeaponPropulsion {
    /// Ближний бой (полный DM)
    Melee,
    /// Метательное/Луки (отрицательный DM полностью, положительный делится на 2)
    MusclePropelled,
    /// Огнестрельное/Энергетическое (DM не применяется)
    SelfPropelled,
}

/// Ценность предмета (соотносится с WealthLevel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemValue {
    Cheap,
    Inexpensive,
    Average,
    Expensive,
    Priceless,
}

/// Классы оружия (стр. 147-148). Критически важно для навыков (Skill Specialties).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeaponClass {
    Axe,
    Bow,
    Brawl,
    Club,
    Crossbow,
    Dagger,
    Explosive,
    Flail,
    Grenade,
    Hammer,
    Hand,
    Improvised,
    Mace,
    MachineGun,
    Missile,
    Pistol,
    PistolEnergy,
    Polearm,
    Revolver,
    Rifle,
    RifleEnergy,
    Shield,
    Shotgun,
    Spear,
    Staff,
    SubmachineGun,
    Sword,
    // Артиллерия (стр. 186)
    Cannon,
    Launcher,
    MountedGun,
    SiegeEngine,
    Turret,
    Other,
}

/// Хват оружия
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandednessReq {
    OneHanded,
    TwoHanded,
    OneOrTwoHanded, // Например, Bastard Sword или Short Spear
}

/// Скорострельность / Количество атак (Attk)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RateOfFire {
    /// N атак за 1 боевой раунд
    PerRound { count: u8 },
    /// 1 атака раз в N раундов (например, арбалеты, требующие перезарядки 1/2, 1/5)
    EveryNRounds { rounds: u8 },
    /// Очередь (Burst). Опционально может указывать кол-во выстрелов в очереди.
    Burst { bullets: u8 },
    /// Полный автомат (Autofire)
    FullAuto,
}

/// Нагрузка от брони (Burden) - влияет на скиллы и усталость
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ArmorBurden {
    #[default]
    None,
    Light,
    Moderate,
    Cumbersome,
}
