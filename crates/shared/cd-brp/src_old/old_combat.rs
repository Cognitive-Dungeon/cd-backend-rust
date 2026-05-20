use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};

use crate::dice::{DamageModifier, DiceType, Sign};
use crate::resistance_chance;
use crate::rolls::SuccessLevel;

// ============================================================================
// Типы оружия и урона
// ============================================================================

/// Спецэффекты оружия по правилам BRP UGE (стр. 149)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponSpecial {
    Bleeding,   // Кровотечение (мечи, топоры)
    Crushing,   // Дробящее: удваивает Damage Modifier, оглушает (булавы, молоты)
    Entangling, // Опутывающее: обездвиживает (сети, лассо)
    Impaling,   // Пронзающее: удваивает базовый урон оружия (копья, стрелы, рапиры)
    Knockback,  // Отбрасывающее: бросок против SIZ (щиты, мощные удары)
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

// ============================================================================
// Матрица Атаки и Защиты
// ============================================================================

/// Итоговый результат столкновения атаки и защиты
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectiveHit {
    MissOrBlocked, // Промах или полностью заблокировано
    Normal,        // Обычное попадание (броня защищает)
    Special,       // Спец-попадание (броня защищает, применяется WeaponSpecial)
    Critical,      // Крит (броня ИГНОРИРУЕТСЯ, макс. урон)
}

/// Результат проверки по Attack and Defense Matrix (стр. 147)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackResolution {
    pub hit: EffectiveHit,
    pub defense_item_damage: i32, // Урон, который получает щит или оружие парирования
}

impl AttackResolution {
    /// Вычисляет итог столкновения атакующего и защищающегося
    pub fn resolve(atk: SuccessLevel, def: Option<SuccessLevel>) -> Self {
        // Если защиты нет, используем фиктивный провал для упрощения match
        let defense: SuccessLevel = def.unwrap_or(SuccessLevel::Failure);

        let (hit, defense_item_damage) = match (atk, defense) {
            // 1. Атака провалилась
            (SuccessLevel::Failure | SuccessLevel::Fumble, _) => (EffectiveHit::MissOrBlocked, 0),

            // 2. Атака: SUCCESS
            (SuccessLevel::Success, SuccessLevel::Critical) => (EffectiveHit::MissOrBlocked, 2),
            (SuccessLevel::Success, SuccessLevel::Special) => (EffectiveHit::MissOrBlocked, 1),
            (SuccessLevel::Success, _) => (EffectiveHit::Normal, 0), // (Success, Success | Failure | Fumble)

            // 3. Атака: SPECIAL
            (SuccessLevel::Special, SuccessLevel::Critical) => (EffectiveHit::MissOrBlocked, 1),
            (SuccessLevel::Special, SuccessLevel::Special) => (EffectiveHit::MissOrBlocked, 0),
            (SuccessLevel::Special, SuccessLevel::Success) => (EffectiveHit::Normal, 2),
            (SuccessLevel::Special, _) => (EffectiveHit::Special, 0), // (Special, Failure | Fumble)

            // 4. Атака: CRITICAL
            (SuccessLevel::Critical, SuccessLevel::Critical) => (EffectiveHit::MissOrBlocked, 0),
            (SuccessLevel::Critical, SuccessLevel::Special) => (EffectiveHit::Normal, 2),
            (SuccessLevel::Critical, SuccessLevel::Success) => (EffectiveHit::Special, 4),
            (SuccessLevel::Critical, _) => (EffectiveHit::Critical, 0), // (Critical, Failure | Fumble)
        };

        Self {
            hit,
            defense_item_damage,
        }
    }
}

// ============================================================================
// Вычисление итогового урона
// ============================================================================

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
}

/// Промежуточный результат расчёта парирования.
/// Используется внутри `resolve_attack`, не выходит наружу.
struct ParryOutcome {
    target_damage: i32,
    item_damage: i32,
}

/// Crushing: удваивает DM перед броском (стр. 150).
/// Вызывается только для Special/Critical.
fn apply_crushing_modifier(dmg_mod: DamageModifier) -> DamageModifier {
    match dmg_mod.sign {
        Sign::Negative => DamageModifier::NONE,
        // Нет DM → минимальный бонус +1D4, как предписано правилами
        Sign::None => DamageModifier::new(Sign::Positive, 1, DiceType::D4),
        Sign::Positive => DamageModifier::new(Sign::Positive, dmg_mod.count * 2, dmg_mod.dice),
    }
}

/// Бросает урон оружия с учётом типа попадания и спецэффекта.
///
/// | hit_type | Impaling      | прочее         |
/// |----------|---------------|----------------|
/// | Critical | max_damage()  | max_damage()   |
/// | Special  | 2× кубики+бонус | обычный бросок |
/// | Normal   | обычный бросок | обычный бросок |
fn roll_weapon_damage<R: Rng + ?Sized>(
    weapon: &WeaponDamage,
    weapon_special: WeaponSpecial,
    hit_type: EffectiveHit,
    rng: &mut R,
) -> i32 {
    match hit_type {
        EffectiveHit::Critical => weapon.max_damage(),
        EffectiveHit::Special if weapon_special == WeaponSpecial::Impaling => {
            // Impaling удваивает кубики оружия, DM прибавляется отдельно позже
            WeaponDamage::new(weapon.count * 2, weapon.dice, weapon.flat_bonus * 2).roll(rng)
        }
        _ => weapon.roll(rng), // Для Normal и Blocked кидаем урон (он нужен для поломки щита)
    }
}

/// Стандартное парирование.
///
/// Предмет получает урон согласно Матрице (`matrix_item_damage`).
/// Если защиты не было (`parry_item_hp == None`) — предмет не берём в расчёт.
/// Урон цели: полный при попадании, 0 при MissOrBlocked.
fn handle_standard_parry(
    total_damage: i32,
    hit_type: EffectiveHit,
    matrix_item_damage: i32,
    parry_item_hp: Option<i32>,
) -> ParryOutcome {
    let target_damage = match hit_type {
        EffectiveHit::MissOrBlocked => 0,
        _ => total_damage,
    };
    // Dodge или отсутствие защиты не повреждают предмет
    let item_damage = if parry_item_hp.is_some() {
        matrix_item_damage
    } else {
        0
    };

    ParryOutcome {
        target_damage,
        item_damage,
    }
}

/// Crushing vs парирующий предмет (стр. 150).
///
/// Resistance roll: `total_damage` vs HP предмета.
/// * Атака победила → предмет поглощает свои ХП, остаток идёт цели.
/// * Предмет выстоял → весь урон впитан, до цели ничего не доходит.
///
/// В обоих случаях `item_damage == total_damage` (предмет испытывает полный удар).
fn handle_crushing_parry<R: Rng + ?Sized>(
    total_damage: i32,
    parry_item_hp: i32,
    rng: &mut R,
) -> ParryOutcome {
    let chance = resistance_chance(total_damage, parry_item_hp);
    let item_damage = total_damage; // Crushing всегда бьёт по предмету на полную

    if rng.random_range(1..=100) <= chance {
        // Атака пробила: разница идёт в цель
        ParryOutcome {
            target_damage: (total_damage - parry_item_hp).max(0),
            item_damage,
        }
    } else {
        // Предмет устоял: цель не получает урона
        ParryOutcome {
            target_damage: 0,
            item_damage,
        }
    }
}

/// Спецэффекты, применяемые к телу цели при Special/Critical.
///
/// Вызывается только если `target_damage > 0`.
fn apply_body_specials<R: Rng + ?Sized>(
    weapon_special: WeaponSpecial,
    total_damage: i32,
    target_siz: i32,
    result: &mut CombatHitResult,
    rng: &mut R,
) {
    match weapon_special {
        WeaponSpecial::Bleeding => result.apply_bleeding = true,
        WeaponSpecial::Entangling => result.entangling = true,
        WeaponSpecial::Knockback => {
            // Resistance roll: суммарный урон атаки против SIZ цели (стр. 151)
            let chance = resistance_chance(total_damage, target_siz);
            if rng.random_range(1..=100) <= chance {
                result.knockback_meters = (total_damage / 5).max(1);
            }
        }
        // Crushing: stun_check выставляется раньше, до броска урона
        _ => {}
    }
}

/// Генерирует урон и спецэффекты в зависимости от качества попадания.
///
/// Последовательность:
/// 1. Ранний выход при промахе/фамбле.
/// 2. Crushing: удвоение DM + выставление stun_check.
/// 3. Бросок урона (оружие + DM).
/// 4. Распределение урона: стандартное парирование или Crushing parry rule.
/// 5. Проверка уничтожения предмета парирования.
/// 6. Ранний выход, если урон не пробился до тела.
/// 7. Флаг игнорирования брони при крите.
/// 8. Спецэффекты на тело цели.
pub fn resolve_attack<R: Rng + ?Sized>(
    atk_level: SuccessLevel,
    def_level: Option<SuccessLevel>,
    weapon: &WeaponDamage,
    weapon_special: WeaponSpecial,
    mut dmg_mod: DamageModifier, // Модификатор от STR+SIZ
    target_siz: i32,             // SIZ цели для расчета Knockback
    parry_item_hp: Option<i32>,  // ХП щита/оружия (None = Уклонение/Отсутствие защиты)
    rng: &mut R,
) -> CombatHitResult {
    let mut result = CombatHitResult::default();

    // 1. Промах/фамбл — дальнейшие вычисления бессмысленны
    if matches!(atk_level, SuccessLevel::Failure | SuccessLevel::Fumble) {
        return result; // Полный промах
    }

    let resolution = AttackResolution::resolve(atk_level, def_level);
    let hit_type = resolution.hit;
    let mut item_damage = resolution.defense_item_damage;

    let is_special_or_crit = matches!(atk_level, SuccessLevel::Special | SuccessLevel::Critical);

    // 1. Обработка Crushing Damage Modifier (удваивается ПЕРЕД броском) и помечает оглушение
    if is_special_or_crit && weapon_special == WeaponSpecial::Crushing {
        result.stun_check = true;
        dmg_mod = apply_crushing_modifier(dmg_mod);
    }

    // 2. Бросаем урон
    let weapon_roll = roll_weapon_damage(weapon, weapon_special, hit_type, rng);
    let dm_roll = crate::dice::roll_modifier(dmg_mod, rng);
    let total_damage = (weapon_roll + dm_roll).max(0);

    // 3. Распределяем урон между целью и предметом парирования
    let parry = if is_special_or_crit
        && weapon_special == WeaponSpecial::Crushing
        && let Some(php) = parry_item_hp
    {
        handle_crushing_parry(total_damage, php, rng)
    } else {
        handle_standard_parry(
            total_damage,
            hit_type,
            resolution.defense_item_damage,
            parry_item_hp,
        )
    };

    // Записываем финальные значения урона
    result.target_damage = parry.target_damage;
    result.parry_item_damage = parry.item_damage;

    // Уничтожен ли предмет парирования?
    if let Some(php) = parry_item_hp
        && result.parry_item_damage >= php
    {
        result.parry_item_destroyed = true;
    }

    // До тела ничего не дошло — спецэффекты на тело не накладываем
    if result.target_damage == 0 {
        return result;
    }

    // 4. Броня и пробитие
    if hit_type == EffectiveHit::Critical {
        result.ignores_armor = true;
    }

    // 5. Спецэффекты, применяемые к телу цели
    if is_special_or_crit {
        apply_body_specials(weapon_special, total_damage, target_siz, &mut result, rng);
    }

    result
}

// ============================================================================
// Тесты (Сверяем математику с книгой!)
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn test_matrix_logic() {
        // Critical Attack vs Success Parry = Special Hit + 4 Damage to parrying weapon
        let res = AttackResolution::resolve(SuccessLevel::Critical, Some(SuccessLevel::Success));
        assert_eq!(res.hit, EffectiveHit::Special);
        assert_eq!(res.defense_item_damage, 4);

        // Special Attack vs Critical Parry = Blocked + 1 Damage to parrying weapon
        let res = AttackResolution::resolve(SuccessLevel::Special, Some(SuccessLevel::Critical));
        assert_eq!(res.hit, EffectiveHit::MissOrBlocked);
        assert_eq!(res.defense_item_damage, 1);
    }

    #[test]
    fn test_critical_damage_ignores_armor() {
        let mut rng = SmallRng::seed_from_u64(42);
        let weapon = WeaponDamage::new(1, DiceType::D6, 1); // 1D6+1
        let dm = DamageModifier::NONE;

        let result = resolve_attack(
            SuccessLevel::Critical,
            None,
            &weapon,
            WeaponSpecial::Bleeding,
            dm,
            10,
            None,
            &mut rng,
        );

        assert_eq!(result.target_damage, 7); // Max of 1D6+1 is 7 (DM is 0)
        assert!(result.ignores_armor);
        assert!(result.apply_bleeding); // Криты тоже накладывают спецэффекты!
    }

    #[test]
    fn test_impaling_special() {
        let mut rng = SmallRng::seed_from_u64(123);
        let weapon = WeaponDamage::new(1, DiceType::D6, 1); // 1D6+1
        let dm = DamageModifier::NONE;

        let result = resolve_attack(
            SuccessLevel::Special,
            None,
            &weapon,
            WeaponSpecial::Impaling,
            dm,
            10,
            None,
            &mut rng,
        );

        // Impale удваивает базовый урон: (1D6+1) * 2 = 2D6+2.
        // Бросок от сида 123 для 2D6 дает конкретное число, но оно точно в пределах 4..=14
        assert!((4..=14).contains(&result.target_damage));
        assert!(!result.ignores_armor); // Спешиал броню не пробивает
    }

    #[test]
    fn test_crushing_special_dm_doubling() {
        let mut rng = SmallRng::seed_from_u64(777);
        let weapon = WeaponDamage::new(1, DiceType::D8, 0); // Тяжелая дубина 1D8

        // Базовый DM: +1D4
        let base_dm = DamageModifier::new(Sign::Positive, 1, DiceType::D4);

        let result = resolve_attack(
            SuccessLevel::Special,
            None,
            &weapon,
            WeaponSpecial::Crushing,
            base_dm,
            10,
            None,
            &mut rng,
        );

        assert!(result.stun_check); // Должна быть проверка на оглушение
        // Урон от дубины (1D8) + Удвоенный DM (2D4).
        assert!((3..=16).contains(&result.target_damage));
    }

    #[test]
    fn test_crushing_parry_attacker_wins() {
        let mut rng = SmallRng::seed_from_u64(1);
        let weapon = WeaponDamage::new(1, DiceType::D8, 10); // Искусственно делаем большой урон (11-18)
        let parry_hp = 5; // У щита всего 5 ХП

        let result = resolve_attack(
            SuccessLevel::Special,
            Some(SuccessLevel::Success),
            &weapon,
            WeaponSpecial::Crushing,
            DamageModifier::NONE,
            10,
            Some(parry_hp),
            &mut rng,
        );

        // Щит должен получить полный урон атаки, сломаться, а излишек пойти в цель
        assert!(result.parry_item_damage > parry_hp);
        assert!(result.parry_item_destroyed);
        assert!(result.target_damage > 0);
        assert!(result.stun_check);
    }

    #[test]
    fn test_knockback_resolution() {
        let mut rng = SmallRng::seed_from_u64(999);
        let weapon = WeaponDamage::new(1, DiceType::D6, 20); // Намеренно огромный урон
        let target_siz = 5; // Маленькая цель

        let result = resolve_attack(
            SuccessLevel::Special,
            None,
            &weapon,
            WeaponSpecial::Knockback,
            DamageModifier::NONE,
            target_siz,
            None,
            &mut rng,
        );

        // Атака с огромным уроном должна выиграть Resistance roll и отбросить
        assert!(result.knockback_meters > 0);
    }

    #[test]
    fn test_apply_crushing_modifier_doubles_positive() {
        let dm = DamageModifier::new(Sign::Positive, 1, DiceType::D6);
        let out = apply_crushing_modifier(dm);
        assert_eq!(out.count, 2);
        assert_eq!(out.dice, DiceType::D6);
        assert_eq!(out.sign, Sign::Positive);
    }

    #[test]
    fn test_apply_crushing_modifier_negative_becomes_none() {
        let dm = DamageModifier::new(Sign::Negative, 1, DiceType::D4);
        let out = apply_crushing_modifier(dm);
        assert_eq!(out, DamageModifier::NONE);
    }

    #[test]
    fn test_handle_standard_parry_no_item() {
        // Dodge (parry_item_hp = None): предмет не получает урона
        let p = handle_standard_parry(10, EffectiveHit::Normal, 2, None);
        assert_eq!(p.item_damage, 0);
        assert_eq!(p.target_damage, 10);
    }

    #[test]
    fn test_handle_standard_parry_blocked() {
        let p = handle_standard_parry(10, EffectiveHit::MissOrBlocked, 1, Some(8));
        assert_eq!(p.target_damage, 0);
        assert_eq!(p.item_damage, 1);
    }
}
