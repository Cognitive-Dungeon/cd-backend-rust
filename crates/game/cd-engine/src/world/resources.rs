use bevy_ecs::prelude::*;
use cd_data::depot::Depot;
use cd_map::{SpatialGrid, WorldMap};
use cd_telemetry::TelemetrySink;
use std::sync::{Arc, RwLock};

/// Обертка над картой мира для ECS
#[derive(Resource)]
pub struct MapResource {
    pub inner: WorldMap,
}

/// Обертка над пространственной сеткой для ECS
#[derive(Resource)]
pub struct GridResource {
    pub inner: SpatialGrid,
}

/// Обертка над реестром (ObjectGuid <-> Entity)
#[derive(Resource, Default)]
pub struct RegistryResource {
    pub inner: crate::world::registry::EntityRegistry,
}

/// Данные игры (Depot). Используем Arc<RwLock>, так как он может
/// перезагружаться на лету (hot-reload) из файла.
#[derive(Resource, Clone)]
pub struct GameDataResource {
    pub depot: Arc<RwLock<Option<Depot>>>,
}

/// Контекст тика (RNG и номер тика)
#[derive(Resource)]
pub struct TickResource {
    pub id: crate::TickId,
    pub world_seed: u64,
}

#[derive(Resource)]
pub struct TelemetryResource(pub Arc<dyn TelemetrySink>);
