//! Модуль Окружающей Среды: Удушье, Падения, Огонь (Environment, стр. 92-96).

use crate::rules::resolution::resolve_skill;
use crate::types::{
    AsphyxiationResult, D100Roll, FallingResult, HitPoints, SkillRating, SuccessLevel,
};

// ============================================================================
// ПАДЕНИЕ (FALLING)
// ============================================================================

/// Рассчитывает урон от падения.
///
/// `distance_meters` — высота падения в метрах.
/// `mitigation_roll_result` — результат броска Jump или Agility (DEXx5) для смягчения удара.
/// `rolled_damage_dice` — массив брошенных кубиков D6 (сервер кидает по 1D6 за каждые 3м).
#[must_use]
pub fn resolve_falling(
    distance_meters: u16,
    mitigation_roll_result: Option<SuccessLevel>,
    rolled_damage_dice: &[u16], // Результаты D6
) -> FallingResult {
    // В большинстве реализаций BRP урон от падения ограничен
    // (обычно 20D6, что соответствует ~60 метрам — скорость, после которой урон не растёт из-за предельной скорости падения).
    // 1D6 урона за каждые полные 3 метра (стр. 94)
    const MAX_FALLING_DICE: usize = 20;
    let dice_count = ((distance_meters / 3) as usize).min(MAX_FALLING_DICE);

    // Если падение меньше 3 метров, урона нет
    if dice_count == 0 {
        return FallingResult {
            damage_taken: HitPoints::ZERO,
            mitigated: false,
        };
    }

    // Собираем базовый урон из брошенных кубиков
    // Берем только нужное количество кубиков (защита от ошибки сервера)
    let mut total_damage: u16 = rolled_damage_dice.iter().take(dice_count).sum();
    let mut mitigated = false;

    // Смягчение урона (Jump или Agility roll)
    if let Some(success_level) = mitigation_roll_result
        && success_level.is_success()
    {
        mitigated = true;
        // По базовым правилам:
        // Normal Success: -1D6 урона (отнимаем максимальный кубик или 3 в среднем, мы отнимем среднее 3 для детерминизма или вычтем первый куб)
        // Но чтобы быть точными: урон просто уменьшается на 1 кубик.
        // Special: -2D6, Critical: -3D6.

        let dice_to_remove = match success_level {
            SuccessLevel::CriticalSuccess => 3,
            SuccessLevel::SpecialSuccess => 2,
            _ => 1,
        };

        // Отнимаем урон от брошенных кубиков, начиная с наибольших (в пользу игрока)
        let mut sorted_dice = rolled_damage_dice
            .iter()
            .take(dice_count)
            .copied()
            .collect::<Vec<_>>();
        sorted_dice.sort_unstable_by(|a, b| b.cmp(a)); // По убыванию

        total_damage = sorted_dice.into_iter().skip(dice_to_remove).sum();
    }

    FallingResult {
        damage_taken: HitPoints::new(total_damage as i16),
        mitigated,
    }
}

// ============================================================================
// УДУШЬЕ (ASPHYXIATION)
// ============================================================================

/// Рассчитывает последствия удушья (нахождение под водой, в газу) в текущем раунде.
///
/// `stamina_chance` — базовый шанс (CONx5) или текущий сниженный шанс.
/// `stamina_roll` — бросок D100.
/// `damage_roll` — бросок 1D3 (урон при провале).
#[must_use]
pub fn resolve_asphyxiation(
    stamina_chance: SkillRating,
    stamina_roll: D100Roll,
    damage_roll: u16, // Обычно 1D3
) -> AsphyxiationResult {
    let result = resolve_skill(stamina_roll, stamina_chance);

    // Стр. 92: При успехе персонаж задерживает дыхание.
    // При провале начинает задыхаться и теряет 1D3 HP в раунд.
    // Фамбл (Fumble) может означать немедленную потерю сознания или двойной урон (на усмотрение GM, оставим потерю сознания).
    if result.is_success() {
        AsphyxiationResult::Safe
    } else if result == SuccessLevel::Fumble {
        AsphyxiationResult::Unconscious
    } else {
        AsphyxiationResult::TakesDamage(HitPoints::new(damage_roll as i16))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{D100Roll, SkillRating};

    #[test]
    fn test_falling_damage() {
        // Падение на 10 метров -> 3D6 урона.
        let dice = vec![4, 2, 5]; // Сумма = 11

        let result = resolve_falling(10, None, &dice);
        assert_eq!(result.damage_taken.get(), 11);
        assert!(!result.mitigated);
    }

    #[test]
    fn test_falling_mitigation_success() {
        // 10 метров -> 3D6 урона. Кубики: 4, 2, 5.
        // Успех (Success) убирает 1 кубик. Убираем наибольший (5). Остается 4+2=6.
        let dice = vec![4, 2, 5];

        let result = resolve_falling(10, Some(SuccessLevel::Success), &dice);
        assert_eq!(result.damage_taken.get(), 6);
        assert!(result.mitigated);
    }

    #[test]
    fn test_falling_mitigation_critical() {
        // 10 метров -> 3D6. Крит убирает 3D6. Урон должен стать 0.
        let dice = vec![6, 6, 6];

        let result = resolve_falling(10, Some(SuccessLevel::CriticalSuccess), &dice);
        assert_eq!(result.damage_taken.get(), 0);
        assert!(result.mitigated);
    }

    #[test]
    fn test_asphyxiation_safe() {
        let chance = SkillRating::new(60); // Stamina (CONx5)
        let roll = D100Roll::new(40); // Успех

        let result = resolve_asphyxiation(chance, roll, 2); // 1D3 выпало 2
        assert_eq!(result, AsphyxiationResult::Safe);
    }

    #[test]
    fn test_asphyxiation_damage() {
        let chance = SkillRating::new(60);
        let roll = D100Roll::new(80); // Провал

        let result = resolve_asphyxiation(chance, roll, 3); // 1D3 выпало 3
        assert_eq!(result, AsphyxiationResult::TakesDamage(HitPoints::new(3)));
    }
}
