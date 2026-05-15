use bevy::ecs::prelude::*;
use cd_core::{ObjectGuid, WorldPos};
use cd_ecs::{
    InstanceId,
    components::{Controller, Creature, Door, Furniture, Guid, Position, Render, Stats},
};

use crate::world::resources::DefsCache;

/// Трейт-расширение для удобного спавна игровых объектов.
/// Прячет внутри себя всю логику поиска определений (Def) и сборки компонентов.
pub trait EntityFactoryExt<'w, 's> {
    /// Спавнит существо (моба или игрока).
    fn spawn_creature(
        &mut self,
        slug: &str,
        guid: ObjectGuid,
        pos: WorldPos,
        name: impl Into<String>,
        defs: &DefsCache,
        is_player: bool,
        instance: InstanceId,
    ) -> Option<Entity>;

    /// Спавнит мебель (двери, сундуки и т.д.).
    fn spawn_furniture(
        &mut self,
        slug: &str,
        guid: ObjectGuid,
        pos: WorldPos,
        defs: &DefsCache,
        instance: InstanceId,
    ) -> Option<Entity>;
}

impl<'w, 's> EntityFactoryExt<'w, 's> for Commands<'w, 's> {
    fn spawn_creature(
        &mut self,
        slug: &str,
        guid: ObjectGuid,
        pos: WorldPos,
        name: impl Into<String>,
        defs: &DefsCache,
        is_player: bool,
        instance: InstanceId,
    ) -> Option<Entity> {
        // 1. Пытаемся найти ID по слагу
        let Some(&id) = defs.slug_to_creature.get(slug) else {
            tracing::error!("spawn_creature: slug '{}' not found in defs", slug);
            return None;
        };

        // 2. Достаем само определение
        let Some(def) = defs.creatures.get(&id) else {
            tracing::error!("spawn_creature: CreatureDef not found for id {:?}", id);
            return None;
        };

        // 3. Формируем базовый бандл компонентов существа
        let mut entity_cmds = self.spawn(cd_ecs::CreatureBundle {
            guid: Guid(guid),
            instance,
            position: Position(pos),
            name: Name::new(name.into()),
            render: Render { glyph: def.glyph },
            creature: Creature(id),
            stats: Stats {
                hp: def.base_hp,
                max_hp: def.base_hp,
                mana: def.base_mp,
                max_mana: def.base_mp,
            },
        });

        // 4. Довешиваем опциональные компоненты
        if is_player {
            entity_cmds.insert(Controller {
                agent_id: "player".into(),
            });
        }

        Some(entity_cmds.id())
    }

    fn spawn_furniture(
        &mut self,
        slug: &str,
        guid: ObjectGuid,
        pos: WorldPos,
        defs: &DefsCache,
        instance: InstanceId,
    ) -> Option<Entity> {
        let Some(&id) = defs.slug_to_furniture.get(slug) else {
            tracing::error!("spawn_furniture: slug '{}' not found in defs", slug);
            return None;
        };

        let Some(def) = defs.furniture.get(&id) else {
            tracing::error!("spawn_furniture: FurnitureDef not found for id {:?}", id);
            return None;
        };

        let mut entity_cmds = self.spawn(cd_ecs::FurnitureBundle {
            guid: Guid(guid),
            instance,
            position: Position(pos),
            name: Name::new(def.name.clone()),
            render: Render { glyph: def.glyph },
            furniture: Furniture(id),
        });

        // Добавляем специфичную логику (например, для дверей)
        if slug == "door_closed" || slug.starts_with("door") {
            entity_cmds.insert(Door { is_open: false });
        }

        Some(entity_cmds.id())
    }
}
