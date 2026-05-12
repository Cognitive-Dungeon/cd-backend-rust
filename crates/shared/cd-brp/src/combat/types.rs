use crate::dice::DiceType;
use rand::Rng;
use rand::RngExt;
use serde::{Deserialize, Serialize};

/// Спецэффекты оружия по правилам BRP UGE (стр. 149)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponSpecial {
    /// Кровотечение (мечи, топоры)
    Bleeding,
    /// Дробящее: удваивает Damage Modifier, оглушает (булавы, молоты)
    Crushing,
    /// Опутывающее: обездвиживает (сети, лассо)
    Entangling,
    /// Пронзающее: удваивает базовый урон оружия (копья, стрелы, рапиры)
    Impaling,
    /// Отбрасывающее: бросок против SIZ (щиты, мощные удары)
    Knockback,
    None,
}

/// Базовый урон оружия (например, 1D6+1 для короткого меча)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponDamage {
    pub count: u32,
    pub dice: DiceType,
    pub flat_bonus: i32,
}

impl Default for WeaponDamage {
    fn default() -> Self {
        Self {
            count: 1,
            dice: DiceType::D6,
            flat_bonus: 0,
        }
    }
}

impl WeaponDamage {
    pub const fn new(count: u32, dice: DiceType, flat_bonus: i32) -> Self {
        Self {
            count,
            dice,
            flat_bonus,
        }
    }

    /// Обычный бросок урона оружия
    pub fn roll<R: Rng + ?Sized>(&self, rng: &mut R) -> i32 {
        let mut total = self.flat_bonus;
        for _ in 0..self.count {
            total += rng.random_range(1..=self.dice.faces() as i32);
        }
        total.max(0)
    }

    /// Максимально возможный урон (используется при Critical успехах)
    pub fn max_damage(&self) -> i32 {
        (self.count as i32 * self.dice.faces() as i32) + self.flat_bonus
    }
}

/// Итоговый результат столкновения атаки и защиты
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectiveHit {
    /// Промах или полностью заблокировано
    MissOrBlocked,
    /// Обычное попадание (броня защищает)
    Normal,
    /// Спец-попадание (броня защищает, применяется WeaponSpecial)
    Special,
    /// Крит (броня ИГНОРИРУЕТСЯ, макс. урон)
    Critical,
}

/// Результат проверки по Attack и Defense Matrix
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackResolution {
    pub hit: EffectiveHit,
    /// Урон, который получает щит или оружие парирования
    pub defense_item_damage: i32,
}

/// Данные, которые возвращает движок боевки для передачи в ECS
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CombatHitResult {
    /// Урон, который доходит до цели (броня вычитается из этого значения)
    pub target_damage: i32,
    /// Урон, который получает оружие/щит парирования
    pub parry_item_damage: i32,
    /// Оружие парирования уничтожено
    pub parry_item_destroyed: bool,
    /// Крит: броня цели игнорируется
    pub ignores_armor: bool,
    /// Спецэффект: цель начинает кровоточить (1 ХП в раунд)
    pub apply_bleeding: bool,
    /// Спецэффект: цель опутана
    pub entangling: bool,
    /// Цель должна кинуть Stamina roll или будет оглушена на 1D3 раунда
    pub stun_check: bool,
    /// На сколько метров цель отброшена (если > 0, нужна проверка Agility, иначе Prone)
    pub knockback_meters: i32,
    /// Атакующий провалил проверку
    pub attacker_fumbled: bool,
    /// Защищающийся провалил проверку
    pub defender_fumbled: bool,
}
