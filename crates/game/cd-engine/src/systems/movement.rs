use crate::systems::intents::IntentMove;
use crate::world::resources::{GridResource, MapResource, RegistryResource};
use crate::world::subsystems::SpatialSubsystem;
use bevy_ecs::message::MessageReader;
use bevy_ecs::system::{Query, Res, ResMut};
use cd_core::ObjectGuid;
use cd_ecs::components::{Name, Position};
use cd_ecs::{Guid, Stats};
use std::collections::HashMap;

pub fn movement_system(
    mut reader: MessageReader<IntentMove>,
    mut movers: Query<(&Guid, &Name, &mut Position, &Stats)>,
    mut spatial: SpatialSubsystem, // <-- Подключили подсистему!
) {
    for intent in reader.read() {
        if let Ok((guid, name, mut pos, stats)) = movers.get_mut(intent.entity) {
            // Вся грязная логика ушла под капот
            if spatial.move_entity(guid.0, pos.0, intent.target).is_ok() {
                pos.0 = intent.target;
                tracing::info!("{} moved!", name.0);
            }
        }
    }
}
