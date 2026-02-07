use cd_core::ObjectGuid;
use hecs::Entity;
use std::collections::HashMap;

/// Отвечает за связь между постоянным GUID (БД/Сеть) и временным ECS Entity.
#[derive(Default)]
pub struct EntityRegistry {
    // Guid -> Entity (для поиска O(1))
    guid_to_entity: HashMap<ObjectGuid, Entity>,
    // Entity -> Guid (для обратного маппинга при сериализации)
    entity_to_guid: HashMap<Entity, ObjectGuid>,
}

impl EntityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, guid: ObjectGuid, entity: Entity) {
        self.guid_to_entity.insert(guid, entity);
        self.entity_to_guid.insert(entity, guid);
    }

    pub fn unregister(&mut self, guid: ObjectGuid) {
        if let Some(entity) = self.guid_to_entity.remove(&guid) {
            self.entity_to_guid.remove(&entity);
        }
    }

    pub fn get_entity(&self, guid: ObjectGuid) -> Option<Entity> {
        self.guid_to_entity.get(&guid).copied()
    }

    pub fn get_guid(&self, entity: Entity) -> Option<ObjectGuid> {
        self.entity_to_guid.get(&entity).copied()
    }
}