use crate::OpposedOutcome;
use crate::math::BrpFractions;
use crate::types::{CharacteristicMarker, D100Roll, SkillRating, Stat, SuccessLevel};

/// Трейт, наделяющий рейтинг навыка способностью генерировать свои пороги успеха (стр. 16 рулбука).
pub trait BrpThresholds {
    fn critical_target(&self) -> u16;
    fn special_target(&self) -> u16;
    fn success_target(&self) -> u16;
    fn fumble_threshold(&self) -> u16;
}

impl BrpThresholds for SkillRating {
    #[inline]
    fn critical_target(&self) -> u16 {
        self.get().twentieth_ceil()
    }

    #[inline]
    fn special_target(&self) -> u16 {
        self.get().fifth_ceil()
    }

    #[inline]
    fn success_target(&self) -> u16 {
        self.get()
    }

    #[inline]
    fn fumble_threshold(&self) -> u16 {
        // Шанс провала (не может быть меньше 0)
        let fail_chance = 100u16.saturating_sub(self.get());

        // Фамбл - это худшие 5% (1/20) от шанса провала.
        let fumble_range = fail_chance.twentieth_ceil().max(1);

        // Порог фамбла отсчитывается сверху вниз от 101.
        101u16.saturating_sub(fumble_range)
    }
}

/// Чистая функция разрешения любой проверки навыка в BRP.
/// Вычисляется моментально, возвращает конкретный уровень успеха.
pub fn resolve_skill(roll: D100Roll, rating: SkillRating) -> SuccessLevel {
    let val = roll.get();

    // 1. Проверяем фамбл (катастрофический провал)
    if val >= rating.fumble_threshold() {
        return SuccessLevel::Fumble;
    }

    // 2. Проверяем крит (1/20)
    if val <= rating.critical_target() {
        return SuccessLevel::CriticalSuccess;
    }

    // 3. Проверяем особый успех (1/5)
    if val <= rating.special_target() {
        return SuccessLevel::SpecialSuccess;
    }

    // 4. Проверяем обычный успех
    if val <= rating.success_target() {
        return SuccessLevel::Success;
    }

    // В остальных случаях это обычный провал
    SuccessLevel::Failure
}

/// Чистая функция разрешения Таблицы Сопротивления (Resistance Table, стр. 18-19, 248).
pub fn resolve_resistance<T: CharacteristicMarker>(
    active: Stat<T>,
    passive: Stat<T>,
    roll: D100Roll,
) -> SuccessLevel {
    let active_val = active.get() as i32;
    let passive_val = passive.get() as i32;

    // Формула из книги: 50% + (active * 5) - (passive * 5)
    let target_chance = 50 + (active_val * 5) - (passive_val * 5);

    // Ограничиваем нижнюю границу нулем
    let clamped_chance = target_chance.max(0) as u16;

    // Превращаем шанс в рейтинг навыка и резолвим по стандартным правилам!
    let resistance_rating = SkillRating::new(clamped_chance);

    resolve_skill(roll, resistance_rating)
}

/// Разрешает Встречную Проверку (Opposed Roll, стр. 26).
/// Используется в бою (Атака vs Защита) или соревнованиях навыков (Hide vs Spot).
pub fn resolve_opposed(
    active_rating: SkillRating,
    active_roll: D100Roll,
    passive_rating: SkillRating,
    passive_roll: D100Roll,
) -> OpposedOutcome {
    let active_res = resolve_skill(active_roll, active_rating);
    let passive_res = resolve_skill(passive_roll, passive_rating);

    // 1. Сравниваем уровни успеха (Critical > Special > Success > Failure > Fumble)
    if active_res > passive_res {
        return OpposedOutcome::ActiveWins(active_res);
    }
    if passive_res > active_res {
        return OpposedOutcome::PassiveWins(passive_res);
    }

    // 2. Если уровни успеха равны (оба выкинули Success, или оба Special):
    // В BRP UGE (стр. 26) побеждает тот, у кого ВЫШЕ базовый навык.
    if active_res.is_success() {
        if active_rating > passive_rating {
            return OpposedOutcome::ActiveWins(active_res);
        }
        if passive_rating > active_rating {
            return OpposedOutcome::PassiveWins(passive_res);
        }
        // Если шансы равны, объявляем ничью
        return OpposedOutcome::Tie;
    }

    // Если оба провалились (Failure или Fumble), никто не побеждает
    OpposedOutcome::Tie
}
