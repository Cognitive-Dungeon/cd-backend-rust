use bevy::ecs::prelude::*;

use crate::Anatomy;
use crate::anatomy::CharacterState;

pub fn update_vitals_system(mut anatomy_query: Query<&mut Anatomy>) {
    for mut anatomy in anatomy_query.iter_mut() {
        if anatomy.vitals.state == CharacterState::Dead {
            continue;
        }

        if !anatomy.is_alive() {
            anatomy.vitals.consciousness = 0.0;
            anatomy.vitals.state = CharacterState::Dead;
            continue;
        }

        // 1. Агрегация общей кровопотери и боли из всех ран!
        let mut total_bleeding = 0.0;
        let mut total_pain = 0.0;

        for part in anatomy.parts.values() {
            for wound in &part.wounds {
                if wound.is_active() {
                    total_bleeding += wound.bleeding_rate;
                    total_pain += wound.pain_level;
                }
            }
        }

        anatomy.substances.blood_loss_rate = total_bleeding;
        anatomy.vitals.pain = total_pain;

        // 2. Потеря крови отнимает общий объем (в будущем тут будет time.delta_secs)
        // Допустим, система тикает раз в секунду:
        if anatomy.substances.blood_loss_rate > 0.0 {
            anatomy.substances.blood_volume -= anatomy.substances.blood_loss_rate;
        }

        // 3. Гиповолемический шок (потеря > 50% крови = обморок)
        let max_blood = anatomy.substances.max_blood_volume.max(1.0); // Защита от деления на 0
        let blood_loss_percent =
            1.0 - (anatomy.substances.blood_volume / max_blood).clamp(0.0, 1.0);

        anatomy.vitals.shock_level = blood_loss_percent;

        // 4. Падение сознания от боли и шока
        let pain_penalty = (anatomy.vitals.pain / 150.0).clamp(0.0, 1.0);
        let shock_penalty = (blood_loss_percent * 2.0).clamp(0.0, 1.0);

        anatomy.vitals.consciousness = 1.0 - pain_penalty.max(shock_penalty);

        // 5. Переход в бессознательное состояние
        if anatomy.vitals.consciousness <= 0.0 {
            anatomy.vitals.state = CharacterState::Unconscious;
        } else if anatomy.vitals.pain > 0.0 || anatomy.substances.blood_loss_rate > 0.0 {
            anatomy.vitals.state = CharacterState::Wounded;
        } else {
            anatomy.vitals.state = CharacterState::Healthy;
        }
    }
}
