use bevy::ecs::prelude::*;

use crate::{
    HitLocationType,
    anatomy::{PenetrationProfile, WoundType},
};

#[derive(Component)]
pub struct Damageable;

#[derive(Message, Clone)]
pub struct DamageMessage {
    pub target: Entity,
    pub location: HitLocationType,
    pub raw_damage: i32,
    pub wound_type: WoundType,
    pub penetration: PenetrationProfile,
}

pub fn apply_damage_system(
    mut damage_messages: MessageReader<DamageMessage>,
    mut anatomy_query: Query<&mut crate::anatomy::Anatomy, With<Damageable>>,
) {
    for message in damage_messages.read() {
        let Ok(mut anatomy) = anatomy_query.get_mut(message.target) else {
            continue;
        };
        let result = anatomy.apply_damage_detailed(
            message.location,
            message.raw_damage,
            message.wound_type,
            message.penetration.depth_mm,
        );
        tracing::trace!("Damage to {:?}: {:?}", message.location, result);
    }
}
