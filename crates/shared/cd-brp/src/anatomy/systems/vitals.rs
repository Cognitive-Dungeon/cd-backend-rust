use bevy::ecs::prelude::*;

use crate::Anatomy;

pub fn update_vitals_system(mut anatomy_query: Query<&mut Anatomy>) {
    for mut anatomy in anatomy_query.iter_mut() {
        if anatomy.current_hp <= 0 {
            anatomy.vitals.consciousness = 0.0;
            anatomy.vitals.state = crate::anatomy::components::vitals::CharacterState::Dead;
        }
    }
}
