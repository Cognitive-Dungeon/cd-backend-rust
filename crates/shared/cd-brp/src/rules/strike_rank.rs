//! Модуль вычисления фаз инициативы (Strike Ranks, стр. 35, 48-49).

use crate::{
    PowerPoints,
    types::{Dex, Siz, Stat, StrikeRank, WeaponLength},
};

/// Константа: сколько рангов занимает подготовка оружия или простая перезарядка (Стр. 49).
pub const READY_WEAPON_SR_COST: u8 = 5;

/// Максимальный Strike Rank, в который можно совершить действие в текущем раунде (Стр. 48).
pub const MAX_SR_PER_ROUND: u8 = 10;

/// Внутренний хелпер: вычисляет базовый "шаг" инициативы по значению характеристики.
/// Границы в BRP универсальны: <10, 10-15, 16-19, 20+.
#[inline]
const fn stat_sr_step(val: u16) -> u8 {
    match val {
        20..=u16::MAX => 1,
        16..=19 => 2,
        10..=15 => 3,
        0..=9 => 4,
    }
}

/// Добавляет задержку к текущему Strike Rank (например, за доставание оружия).
/// Если итоговый SR превышает 10, действие должно быть перенесено на следующий раунд.
#[must_use]
pub const fn add_action_delay(base_sr: StrikeRank, delay: u8) -> StrikeRank {
    StrikeRank::new(base_sr.get().saturating_add(delay))
}

/// Проверяет, может ли действие быть завершено в текущем боевом раунде.
/// В BRP раунд длится 12 секунд (стр. 48), и все действия происходят с 1 по 10 Strike Rank.
/// Если SR > 10, действие "переливается" в следующий раунд.
#[must_use]
pub const fn fits_in_current_round(sr: StrikeRank) -> bool {
    sr.get() <= MAX_SR_PER_ROUND
}

/// Возвращает Strike Rank, на который перенесется действие в СЛЕДУЮЩЕМ раунде (Стр. 49).
/// Например, если игрок достает меч (SR +5) на DEX SR 3, а у меча Weapon SR 4,
/// итоговый SR = 12. Это больше 10. В следующем раунде он ударит на SR 2 (12 - 10 = 2).
#[must_use]
pub const fn overflow_to_next_round(sr: StrikeRank) -> StrikeRank {
    let val = sr.get();
    if val > MAX_SR_PER_ROUND {
        StrikeRank::new(val - MAX_SR_PER_ROUND)
    } else {
        // Если переполнения нет, технически функция не должна вызываться,
        // но для безопасности возвращаем текущий SR.
        sr
    }
}

/// Вычисляет базовый Strike Rank от Ловкости (DEX SR).
/// Стр. 49: DEX 20+ = 1, DEX 16-19 = 2, DEX 10-15 = 3, DEX 1-9 = 4.
#[must_use]
#[inline]
pub const fn calculate_dex_strike_rank(dex: Stat<Dex>) -> StrikeRank {
    StrikeRank::new(stat_sr_step(dex.get()))
}

/// Вычисляет Strike Rank от Размера (SIZ SR).
/// Используется ТОЛЬКО для атак без оружия (Brawl, Grapple) и монстров.
/// Стр. 49: SIZ 20+ = 0, SIZ 16-19 = 1, SIZ 10-15 = 2, SIZ 1-9 = 3.
#[must_use]
#[inline]
pub const fn calculate_siz_strike_rank(siz: Stat<Siz>) -> StrikeRank {
    // В BRP таблица SIZ SR математически идентична DEX SR,
    // но всегда смещена на 1 вниз (бьет быстрее на 1 ранг).
    StrikeRank::new(stat_sr_step(siz.get()).saturating_sub(1))
}

/// Вычисляет Strike Rank от длины оружия (Weapon SR).
/// Стр. 49: У длинного оружия модификатор меньше (оно бьет быстрее).
#[must_use]
#[inline]
pub const fn weapon_length_strike_rank(length: WeaponLength) -> StrikeRank {
    StrikeRank::new(length as u8)
}

/// Вычисляет итоговый Strike Rank для атаки в БЛИЖНЕМ бою.
/// Melee SR = DEX SR + Weapon SR (или SIZ SR для кулаков).
#[must_use]
#[inline]
pub const fn calculate_melee_strike_rank(
    dex_sr: StrikeRank,
    weapon_length: Option<WeaponLength>,
    siz_sr: StrikeRank,
) -> StrikeRank {
    let base = dex_sr.get();

    let modifier = match weapon_length {
        Some(length) => weapon_length_strike_rank(length).get(),
        None => siz_sr.get(), // Без оружия используем SIZ SR
    };

    StrikeRank::new(base.saturating_add(modifier))
}

/// Вычисляет итоговый Strike Rank для дистанционной атаки (Ranged / Firearm).
/// Стр. 49: Огнестрел стреляет на DEX SR. Прицеливание (Aiming) не меняет SR,
/// но подготовка оружия может занять раунды.
#[must_use]
#[inline]
pub const fn calculate_ranged_strike_rank(dex_sr: StrikeRank) -> StrikeRank {
    // В базовом BRP дистанционная атака обычно идет строго по DEX SR,
    // если оружие уже в руках и готово к бою.
    dex_sr
}

/// Вычисляет Strike Rank для произнесения заклинания (Magic/Sorcery).
/// Стр. 49: DEX SR + 1 SR за каждое вложенное очко магии (Power Point).
#[must_use]
#[inline]
pub const fn calculate_magic_strike_rank(
    dex_sr: StrikeRank,
    power_points_spent: PowerPoints,
) -> StrikeRank {
    // PowerPoints у нас хранятся как i16 (чтобы разрешать отрицательные значения).
    // Для магии мы тратим положительное количество очков, но на всякий случай
    // берем только положительную часть через max(0), чтобы избежать багов с отрицательным временем каста.

    let spent = power_points_spent.get();
    let safe_spent = if spent > 0 { spent as u8 } else { 0 };

    StrikeRank::new(dex_sr.get().saturating_add(safe_spent))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dex_and_siz_sr() {
        assert_eq!(calculate_dex_strike_rank(Stat::<Dex>::new(12)).get(), 3);
        assert_eq!(calculate_siz_strike_rank(Stat::<Siz>::new(17)).get(), 1);
    }

    #[test]
    fn test_melee_unarmed_sr() {
        // DEX 12 (SR 3) + SIZ 17 (SR 1) = SR 4
        let dex_sr = calculate_dex_strike_rank(Stat::<Dex>::new(12));
        let siz_sr = calculate_siz_strike_rank(Stat::<Siz>::new(17));

        let sr = calculate_melee_strike_rank(dex_sr, None, siz_sr);
        assert_eq!(sr.get(), 4);
    }

    #[test]
    fn test_melee_weapon_sr() {
        // DEX 12 (SR 3) + Medium Weapon (SR 2) = SR 5
        let dex_sr = calculate_dex_strike_rank(Stat::<Dex>::new(12));
        let siz_sr = calculate_siz_strike_rank(Stat::<Siz>::new(10)); // Игнорируется

        let sr = calculate_melee_strike_rank(dex_sr, Some(WeaponLength::Medium), siz_sr);
        assert_eq!(sr.get(), 5);
    }

    #[test]
    fn test_magic_sr() {
        // DEX 16 (SR 2) + 3 Маны = SR 5
        let dex_sr = calculate_dex_strike_rank(Stat::<Dex>::new(16));
        assert_eq!(
            calculate_magic_strike_rank(dex_sr, PowerPoints::new(3)).get(),
            5
        );
    }
}
