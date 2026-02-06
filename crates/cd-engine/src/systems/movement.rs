use cd_core::ObjectGuid;
use cd_ecs::components::{Position, Name};
use cd_map::{WorldMap, SpatialGrid};
use hecs::{World, Entity};
use std::collections::HashMap;

/// Пример системы. В будущем она будет двигать Velocity -> Position.
pub fn run_movement(
    world: &mut World,
    _map: &WorldMap,
    _grid: &mut SpatialGrid,
    _index: &HashMap<ObjectGuid, Entity>
) {
    // Пример итерации: Найти всех, у кого есть Имя и Позиция
    for (_id, (name, pos)) in world.query::<(&Name, &Position)>().iter() {
        // Тут могла быть логика интерполяции или проверки физики
        // tracing::trace!("Processing movement for {}: {:?}", name.0, pos.0);
    }
}