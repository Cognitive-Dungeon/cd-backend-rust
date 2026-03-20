use bevy_ecs::prelude::*;
use cd_data::{
    defs::{SpellDef, SpellId},
    depot::Depot,
};
use cd_ecs::SpatialGrid;
use cd_map::WorldMap;
use cd_telemetry::TelemetrySink;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::world::defs::{CreatureDef, FurnitureDef, MaterialDef};

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
    pub inner: cd_ecs::EntityRegistry,
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

#[derive(Resource, Default)]
pub struct DefsCache {
    pub creatures: HashMap<String, CreatureDef>,
    pub materials: HashMap<String, MaterialDef>,
    pub furniture: HashMap<String, FurnitureDef>,
    pub spells_by_id: HashMap<SpellId, SpellDef>,
    pub spells_by_slug: HashMap<String, SpellId>,
}

impl DefsCache {
    /// Заполняет кэш из распарсенного Depot
    pub fn rebuild_from(&mut self, depot: &cd_data::depot::Depot) {
        if let Some(sheet) = depot.sheet("Creatures") {
            self.creatures = sheet.load_as_map();
            tracing::info!("Loaded {} creatures", self.creatures.len());
        }

        if let Some(sheet) = depot.sheet("Materials") {
            // Для Materials ключом сделаем slug
            self.materials = sheet
                .load_all::<MaterialDef>()
                .into_iter()
                .map(|m| (m.slug.clone(), m))
                .collect();
            tracing::info!("Loaded {} materials", self.materials.len());
        }

        if let Some(sheet) = depot.sheet("Furniture") {
            self.furniture = sheet.load_as_map();
            tracing::info!("Loaded {} furniture items", self.furniture.len());
        }

        if let Some(sheet) = depot.sheet("Spells") {
            let defs = sheet.load_all::<SpellDef>();
            self.spells_by_id.clear();
            self.spells_by_slug.clear();
            for def in defs {
                self.spells_by_slug.insert(def.slug.clone(), def.id);
                self.spells_by_id.insert(def.id, def);
            }
            tracing::info!("Loaded {} spells", self.spells_by_id.len());
        }
    }
}
