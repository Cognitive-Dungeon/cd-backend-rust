use bevy::ecs::prelude::*;

use crate::Anatomy;

pub fn update_vitals_system(time: Res<bevy::time::Time>, mut anatomy_query: Query<&mut Anatomy>) {
    let delta_secs = time.delta_secs();

    for mut anatomy in anatomy_query.iter_mut() {
        anatomy.process_vitals_tick(delta_secs);
    }
}
