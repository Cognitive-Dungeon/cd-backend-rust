//! Модуль использования Сил: Магия, Псионика, Мутации (Powers, стр. 129-140).

use crate::types::{HitPoints, PowerActivationResult, PowerPoints, SuccessLevel};

/// Разрешает попытку активации Силы (Заклинания, Псионики).
///
/// `current_mp` — текущие очки магии персонажа.
/// `mp_cost` — сколько MP/POW заявлено к трате на способность.
/// `activation_roll_result` — результат броска навыка (например, `SkillType::Magic`).
/// Если `None`, способность активируется автоматически (как некоторые мутации).
#[must_use]
pub fn resolve_power_activation(
    current_mp: PowerPoints,
    mp_cost: PowerPoints,
    activation_roll_result: Option<SuccessLevel>,
) -> PowerActivationResult {
    // 1. Хватает ли маны?
    if current_mp.get() < mp_cost.get() {
        return PowerActivationResult::NotEnoughPowerPoints;
    }

    // 2. Если способность автоматическая (не требует броска)
    let roll_result = match activation_roll_result {
        Some(res) => res,
        None => {
            return PowerActivationResult::Success {
                level: SuccessLevel::Success, // Базовый успех для авто-способностей
                mp_spent: mp_cost,
            };
        }
    };

    // 3. Обработка броска (Стр. 132-134 "Casting Spells")
    match roll_result {
        SuccessLevel::CriticalSuccess | SuccessLevel::SpecialSuccess | SuccessLevel::Success => {
            // Успех: тратим заявленную ману
            PowerActivationResult::Success {
                level: roll_result,
                mp_spent: mp_cost,
            }
        }
        SuccessLevel::Failure => {
            // Провал: заклинание не работает. По стандартным правилам тратится 1 MP
            // за попытку сфокусировать энергию (даже если заклинание стоило 5 MP).
            let spent = if mp_cost.get() > 0 { 1 } else { 0 };
            PowerActivationResult::Failure {
                mp_spent: PowerPoints::new(spent),
            }
        }
        SuccessLevel::Fumble => {
            // Фамбл: катастрофа! Тратится ВСЯ вложенная мана.
            // На усмотрение GM (или в строгих модулях) кастер получает 1D3 или 1D6 урона.
            // Мы вернем флаг необходимости урона, чтобы сервер кинул кубик (например 1D6).
            // Чтобы сохранить функцию детерминированной, мы возвращаем "штрафной урон",
            // равный количеству потерянной маны (жесткое правило для хардкора).
            PowerActivationResult::Fumble {
                mp_spent: mp_cost,
                backfire_damage: Some(HitPoints::new(mp_cost.get().min(10))), // 1 урон за каждый потраченный MP (макс 10)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_activation_not_enough_mp() {
        let result = resolve_power_activation(
            PowerPoints::new(2),
            PowerPoints::new(5),
            Some(SuccessLevel::Success),
        );
        assert_eq!(result, PowerActivationResult::NotEnoughPowerPoints);
    }

    #[test]
    fn test_power_activation_success() {
        let result = resolve_power_activation(
            PowerPoints::new(10),
            PowerPoints::new(3),
            Some(SuccessLevel::SpecialSuccess),
        );
        assert_eq!(
            result,
            PowerActivationResult::Success {
                level: SuccessLevel::SpecialSuccess,
                mp_spent: PowerPoints::new(3),
            }
        );
    }

    #[test]
    fn test_power_activation_failure_costs_one_mp() {
        let result = resolve_power_activation(
            PowerPoints::new(10),
            PowerPoints::new(6),
            Some(SuccessLevel::Failure),
        );
        // Заявил 6, провалил бросок, потерял только 1 MP
        assert_eq!(
            result,
            PowerActivationResult::Failure {
                mp_spent: PowerPoints::new(1),
            }
        );
    }

    #[test]
    fn test_power_activation_fumble_backfire() {
        let result = resolve_power_activation(
            PowerPoints::new(10),
            PowerPoints::new(4),
            Some(SuccessLevel::Fumble),
        );
        // Заявил 4, сфамблил. Потерял все 4 MP + получил 4 HP урона в виде отката (Backfire)
        assert_eq!(
            result,
            PowerActivationResult::Fumble {
                mp_spent: PowerPoints::new(4),
                backfire_damage: Some(HitPoints::new(4)),
            }
        );
    }

    #[test]
    fn test_automatic_power() {
        let result = resolve_power_activation(
            PowerPoints::new(5),
            PowerPoints::new(2),
            None, // Бросок не требуется (например, Мутация: Жабры)
        );
        assert_eq!(
            result,
            PowerActivationResult::Success {
                level: SuccessLevel::Success,
                mp_spent: PowerPoints::new(2),
            }
        );
    }
}
