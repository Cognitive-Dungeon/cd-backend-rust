use crate::anatomy::*;
use bevy::prelude::*;

pub struct AnatomyPlugin;

impl Plugin for AnatomyPlugin {
    fn build(&self, app: &mut App) {
        // Регистрируем типы для рефлексии/сериализации
        app.register_type::<Anatomy>()
            .register_type::<BodyPart>()
            .register_type::<VitalStats>();

        // Добавляем системы в расписание
        app.add_systems(
            Update,
            (
                apply_damage_system,       // Обработка входящего урона
                update_vitals_system,      // Пересчёт боли/шока/сознания
                infection_progress_system, // Прогрессия патогенов
            )
                .chain(), // Порядок важен: урон → виталы → инфекции
        );

        // Регенерация — реже, раз в игровые часы
        app.add_systems(
            FixedUpdate,
            app.add_systems(FixedUpdate, healing_tick_system),
        );
    }
}
