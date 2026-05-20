use bevy::ecs::prelude::*;
use bevy::time::Time;

use crate::Anatomy;

pub fn healing_tick_system(time: Res<Time>, mut anatomy_query: Query<&mut Anatomy>) {
    // Простой тик регенерации. В будущем здесь будет логика крови/инфекций
    let delta = time.delta_secs();
    for mut anatomy in anatomy_query.iter_mut() {
        if anatomy.current_hp > 0
            && anatomy.vitals.state != crate::anatomy::components::vitals::CharacterState::Dead
        {
            // Медленное восстановление крови
            anatomy.substances.blood_volume =
                (anatomy.substances.blood_volume + delta * 0.5).min(1000.0);
        }
    }
}
