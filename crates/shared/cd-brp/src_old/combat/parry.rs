use super::types::EffectiveHit;
use crate::resistance_chance;
use rand::Rng;
use rand::RngExt;

pub struct ParryOutcome {
    /// Урон, дошедший до цели (броня вычитается из него позже)
    pub target_damage: i32,
    /// Урон, полученный предметом парирования
    pub item_damage: i32,
}

/// Стандартное парирование.
/// Применяет урон по Матрице к предмету. Если предмет уничтожен (footnote *),
/// избыточный урон пробивает в цель.
pub fn handle_standard_parry(
    total_damage: i32,
    hit_type: EffectiveHit,
    matrix_item_damage: i32,
    parry_item_hp: Option<i32>,
) -> ParryOutcome {
    if hit_type == EffectiveHit::MissOrBlocked {
        return ParryOutcome {
            target_damage: 0,
            item_damage: matrix_item_damage,
        };
    }

    // Если защиты нет (Dodge) → предмет не повреждается
    let Some(hp) = parry_item_hp else {
        return ParryOutcome {
            target_damage: total_damage,
            item_damage: 0,
        };
    };

    // Footnote *: если матричный урон >= HP предмета, он ломается, остаток летит в цель
    if matrix_item_damage >= hp {
        let overflow = (matrix_item_damage - hp).max(0);
        return ParryOutcome {
            target_damage: total_damage + overflow, // Остаток пробивает блок
            item_damage: matrix_item_damage,
        };
    }

    ParryOutcome {
        target_damage: total_damage,
        item_damage: matrix_item_damage,
    }
}

/// Crushing vs парирующий предмет (стр. 150).
///
/// Resistance Roll: `total_damage` vs `parry_item_hp`.
/// Предмет выстоял → поглощает удар, получает урон по Матрице, цель в безопасности.
/// Предмет сломался → получает полный урон атаки, остаток пробивает в цель (броня цели всё ещё работает).
pub fn handle_crushing_parry<R: Rng + ?Sized>(
    total_damage: i32,
    matrix_item_damage: i32,
    parry_item_hp: i32,
    rng: &mut R,
) -> ParryOutcome {
    let chance = resistance_chance(total_damage, parry_item_hp);

    if rng.random_range(1..=100) <= chance {
        // Предмет не выдержал: ломается (получает полный урон), остаток летит в цель
        let overflow = (total_damage - parry_item_hp).max(0);
        ParryOutcome {
            target_damage: overflow,
            item_damage: total_damage,
        }
    } else {
        // Предмет устоял (Защита победила): принимает стандартный урон по Матрице, атака заблокирована
        ParryOutcome {
            target_damage: 0,
            item_damage: matrix_item_damage,
        }
    }
}
