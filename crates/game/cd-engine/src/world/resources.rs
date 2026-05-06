use bevy_ecs::prelude::*;
use cd_data::{
    defs::{CreatureId, FurnitureId, SpellDef, SpellId},
    depot::Depot,
};
use cd_ecs::SpatialGrid;
use cd_map::{MaterialID, WorldMap};
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
    pub creatures: HashMap<CreatureId, CreatureDef>,
    pub materials: HashMap<MaterialID, MaterialDef>,
    pub furniture: HashMap<FurnitureId, FurnitureDef>,
    pub spells: HashMap<SpellId, SpellDef>,

    // TODO: Убрать это вовсе или как то убрать нагрузку на рантайм
    pub slug_to_creature: HashMap<String, CreatureId>,
    pub slug_to_material: HashMap<String, MaterialID>,
    pub slug_to_furniture: HashMap<String, FurnitureId>,
    pub slug_to_spell: HashMap<String, SpellId>,
}

impl DefsCache {
    /// Заполняет кэш из распарсенного Depot
    pub fn rebuild_from(&mut self, depot: &cd_data::depot::Depot) {
        if let Some(sheet) = depot.sheet("Creatures") {
            let defs = sheet.load_all::<CreatureDef>();
            self.creatures.clear();
            self.slug_to_creature.clear();
            for def in defs {
                self.slug_to_creature.insert(def.slug.clone(), def.id);
                self.creatures.insert(def.id, def);
            }
            tracing::info!("Loaded {} creatures", self.creatures.len());
        }

        if let Some(sheet) = depot.sheet("Furniture") {
            let defs = sheet.load_all::<FurnitureDef>();
            self.furniture.clear();
            self.slug_to_furniture.clear();
            for def in defs {
                self.slug_to_furniture.insert(def.slug.clone(), def.id);
                self.furniture.insert(def.id, def);
            }
            tracing::info!("Loaded {} furniture items", self.furniture.len());
        }

        if let Some(sheet) = depot.sheet("Materials") {
            let defs = sheet.load_all::<MaterialDef>();
            self.materials.clear();
            self.slug_to_material.clear();
            for def in defs {
                self.slug_to_material.insert(def.slug.clone(), def.id);
                self.materials.insert(def.id, def);
            }
            tracing::info!("Loaded {} materials", self.materials.len());
        }

        if let Some(sheet) = depot.sheet("Spells") {
            let defs = sheet.load_all::<SpellDef>();
            self.spells.clear();
            self.slug_to_spell.clear();
            for def in defs {
                self.slug_to_spell.insert(def.slug.clone(), def.id);
                self.spells.insert(def.id, def);
            }
            tracing::info!("Loaded {} spells", self.spells.len());
        }
    }
}
