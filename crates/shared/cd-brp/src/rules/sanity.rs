//! Модуль Психологии и Рассудка (Sanity, стр. 102-110).

use crate::types::{D100Roll, Int, SanityCost, SanityPoints, SanityResolution, Stat};

/// Разрешает столкновение с ужасом (Sanity Roll, стр. 104).
///
/// `current_san` — текущие очки рассудка персонажа (SAN).
/// `san_roll` — бросок D100 игрока.
/// `cost` — цена шока (например, 0 при успехе, 4 при провале).
/// `session_starting_san` — SAN персонажа в начале текущего игрового "часа" (для проверки на 20%).
#[must_use]
pub fn resolve_sanity_encounter(
    current_san: SanityPoints,
    san_roll: D100Roll,
    cost: SanityCost,
    session_starting_san: SanityPoints,
) -> SanityResolution {
    // 1. Проверка броска
    // В отличие от навыков, у SAN нет Critical/Special. Только Success или Failure.
    // Фамбл (99-00) по опциональным правилам наносит макс. урон, но в базовом BRP UGE
    // бросок просто должен быть <= текущему SAN.
    let is_success = san_roll.get() <= current_san.get() as u16;

    // 2. Определение потерь
    let points_lost = if is_success {
        cost.on_success
    } else {
        cost.on_failure
    };

    // 3. Проверка триггеров безумия (Стр. 105-106)

    // Временное безумие: потеря 5 или более SAN за ОДИН раз
    let triggers_temporary_insanity_risk = points_lost.get() >= 5;

    // Бессрочное (Indefinite) безумие: потеря 20% или более от стартового SAN за "час"
    let twenty_percent_threshold = session_starting_san.get().saturating_mul(20) / 100;

    // Считаем общую потерю с начала отсчета (включая текущий удар)
    let total_lost_this_session = session_starting_san
        .get()
        .saturating_sub(current_san.get().saturating_sub(points_lost.get()));

    let triggers_indefinite_insanity =
        total_lost_this_session >= twenty_percent_threshold && points_lost.get() > 0;

    SanityResolution {
        points_lost,
        is_success,
        triggers_temporary_insanity_risk,
        triggers_indefinite_insanity,
    }
}

/// Проверяет, впал ли персонаж во Временное Безумие (Temporary Insanity, стр. 105).
/// Вызывается сервером ТОЛЬКО если `triggers_temporary_insanity_risk` == true.
///
/// ВАЖНО: В BRP УСПЕХ броска Idea означает, что персонаж ОСОЗНАЛ ужас и сошел с ума.
/// ПРОВАЛ означает, что психика заблокировала шок (отрицание, ступор), и безумия нет.
#[must_use]
pub fn check_temporary_insanity(int_stat: Stat<Int>, idea_roll: D100Roll) -> bool {
    let idea_target = int_stat.idea_chance().get();

    // Если выкинул меньше или равно порогу (УСПЕХ) -> Сходишь с ума (Возвращает true)
    idea_roll.get() <= idea_target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanity_success_loses_minimum() {
        let current_san = SanityPoints::new(60);
        let roll = D100Roll::new(50); // Успех!
        let cost = SanityCost::new(SanityPoints::new(1), SanityPoints::new(6)); // 1/1D6

        let result = resolve_sanity_encounter(current_san, roll, cost, SanityPoints::new(60));

        assert!(result.is_success);
        assert_eq!(result.points_lost, SanityPoints::new(1));
        assert!(!result.triggers_temporary_insanity_risk);
    }

    #[test]
    fn test_sanity_failure_triggers_temporary_risk() {
        let current_san = SanityPoints::new(50);
        let roll = D100Roll::new(80); // Провал!
        let cost = SanityCost::new(SanityPoints::new(0), SanityPoints::new(5)); // Выпало 5 урона

        let result = resolve_sanity_encounter(current_san, roll, cost, SanityPoints::new(50));

        assert!(!result.is_success);
        assert_eq!(result.points_lost, SanityPoints::new(5));
        // Потеря 5 SAN -> Риск временного безумия!
        assert!(result.triggers_temporary_insanity_risk);
    }

    #[test]
    fn test_indefinite_insanity_trigger() {
        // У персонажа было 50 SAN час назад. 20% от 50 = 10.
        // Сейчас у него 42 (уже потерял 8).
        // Он проваливает бросок и теряет еще 3. Итого потеряно: 11.
        // 11 >= 10. Должно сработать Бессрочное Безумие.

        let current_san = SanityPoints::new(42);
        let roll = D100Roll::new(99);
        let cost = SanityCost::new(SanityPoints::new(0), SanityPoints::new(3));

        let result = resolve_sanity_encounter(current_san, roll, cost, SanityPoints::new(50));

        assert!(!result.triggers_temporary_insanity_risk); // Потерял всего 3 за раз (не 5)
        assert!(result.triggers_indefinite_insanity); // Но в сумме > 20%
    }

    #[test]
    fn test_idea_roll_insanity_paradox() {
        let int = Stat::<Int>::new(12); // Idea = 60

        // Выкинул 40 (Успех Idea) -> ПОНЯЛ УЖАС -> Сошел с ума!
        assert!(check_temporary_insanity(int, D100Roll::new(40)));

        // Выкинул 85 (Провал Idea) -> НЕ ПОНЯЛ -> Психика спаслась!
        assert!(!check_temporary_insanity(int, D100Roll::new(85)));
    }
}
