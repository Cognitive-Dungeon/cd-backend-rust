use bevy::ecs::prelude::*;
use cd_data::defs::{CreatureId, FurnitureId, SpellDef, SpellId};
use cd_ecs::InstanceId;
use cd_map::MaterialID;
use std::{collections::HashMap, sync::Arc};

use crate::world::defs::{CreatureDef, FurnitureDef, MaterialDef};

/// Обертка над картой мира для ECS
#[derive(Resource, Default)]
pub struct MapResource {
    /// Ключ - ID инстанса. У каждого подземелья - своя независимая физическая карта
    pub instances: HashMap<InstanceId, cd_map::WorldMap>,
}

impl MapResource {
    pub fn get_map(&self, instance: InstanceId) -> Option<&cd_map::WorldMap> {
        self.instances.get(&instance)
    }

    pub fn get_mut_map(&mut self, instance: InstanceId) -> &mut cd_map::WorldMap {
        self.instances
            .entry(instance)
            .or_insert_with(cd_map::WorldMap::new)
    }
}

/// Обертка над пространственной сеткой для ECS
#[derive(Resource)]
pub struct GridResource {
    pub inner: cd_ecs::SpatialGrid,
}

/// Обертка над реестром (ObjectGuid <-> Entity)
#[derive(Resource, Default)]
pub struct RegistryResource {
    pub inner: cd_ecs::EntityRegistry,
}

/// Данные игры.
#[derive(Resource, Clone)]
pub struct GameDataResource {
    pub provider: Arc<dyn cd_data::provider::DataProvider>,
}

/// Контекст тика (RNG и номер тика)
#[derive(Resource)]
pub struct TickResource {
    pub id: crate::TickId,
    pub world_seed: u64,
}

#[derive(Resource)]
pub struct TelemetryResource(pub Arc<dyn cd_telemetry::TelemetrySink>);

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
    /// Заполняет кэш из распарсенного Провайдера
    pub fn rebuild_from(&mut self, provider: &dyn cd_data::provider::DataProvider) {
        match provider.load_creatures() {
            Ok(creatures) => {
                self.creatures.clear();
                self.slug_to_creature.clear();
                for (_, def) in creatures {
                    self.slug_to_creature.insert(def.slug.clone(), def.id);
                    self.creatures.insert(def.id, def);
                }
                tracing::info!("Loaded {} creatures", self.creatures.len());
            }
            Err(e) => tracing::error!("Failed to load creatures: {}", e),
        }

        match provider.load_materials() {
            Ok(materials) => {
                self.materials.clear();
                self.slug_to_material.clear();
                for (_, def) in materials {
                    self.slug_to_material.insert(def.slug.clone(), def.id);
                    self.materials.insert(def.id, def);
                }
                tracing::info!("Loaded {} materials", self.materials.len());
            }
            Err(e) => tracing::error!("Failed to load materials: {}", e),
        }

        match provider.load_furniture() {
            Ok(furniture) => {
                self.furniture.clear();
                self.slug_to_furniture.clear();
                for (_, def) in furniture {
                    self.slug_to_furniture.insert(def.slug.clone(), def.id);
                    self.furniture.insert(def.id, def);
                }
                tracing::info!("Loaded {} furniture items", self.furniture.len());
            }
            Err(e) => tracing::error!("Failed to load furniture: {}", e),
        }

        match provider.load_spells() {
            Ok(spells) => {
                self.spells.clear();
                self.slug_to_spell.clear();
                for (_, def) in spells {
                    self.slug_to_spell.insert(def.slug.clone(), def.id);
                    self.spells.insert(def.id, def);
                }
                tracing::info!("Loaded {} spells", self.spells.len());
            }
            Err(e) => tracing::error!("Failed to load spells: {}", e),
        }
    }
}
