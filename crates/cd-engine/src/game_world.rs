use std::sync::{Arc, RwLock};

use cd_core::{ObjectGuid, WorldPos};
use cd_ecs::components::{Position, Stats};
use cd_map::{SpatialGrid, Tile, WorldMap};
use cd_telemetry::TelemetrySink;
use hecs::{CommandBuffer, World};

use crate::game_error::{DamageResult, GameError};
use crate::registry::EntityRegistry;

/// Фасад для написания игровой логики.
///
/// Разработчик работает только с этим типом — никакого hecs, никаких RwLock,
/// никакого трейсинга вручную.
///
/// Пример использования:
/// ```rust
/// fn fireball(world: &mut GameWorld, pos: WorldPos) -> Result<(), GameError> {
///     for guid in world.entities_in_radius(pos, 3) {
///         world.deal_damage(guid, 35)?;
///     }
///     Ok(())
/// }
/// ```
pub struct GameWorld<'a> {
    pub(crate) world: &'a mut World,
    pub(crate) map: &'a mut WorldMap,
    pub(crate) grid: &'a mut SpatialGrid,
    pub(crate) registry: &'a mut EntityRegistry,
    pub(crate) commands: &'a mut CommandBuffer,
    pub(crate) telemetry: &'a dyn TelemetrySink,
    pub(crate) game_data: Arc<RwLock<Option<cd_depot::Depot>>>,
}

impl<'a> GameWorld<'a> {
    // ------------------------------------------------------------------ Map

    /// Получить тайл в мировых координатах.
    pub fn tile_at(&self, pos: WorldPos) -> Tile {
        self.map.get_tile(pos)
    }

    /// Проверить проходимость.
    pub fn is_solid(&self, pos: WorldPos) -> bool {
        self.map.is_solid_fast(pos)
    }

    /// Изменить тайл (немедленно, без откладывания).
    pub fn set_tile(&mut self, pos: WorldPos, tile: Tile) {
        self.map.set_tile(pos, tile);
    }

    // --------------------------------------------------------------- Spatial

    /// Сущности в радиусе вокруг точки (bucket-approximation).
    pub fn entities_in_radius(&self, center: WorldPos, radius: i32) -> Vec<ObjectGuid> {
        self.grid.query_radius(center, radius)
    }

    // ------------------------------------------------------------ Components

    /// Позиция сущности.
    pub fn position(&self, guid: ObjectGuid) -> Result<WorldPos, GameError> {
        let entity = self.entity(guid)?;
        self.world
            .get::<&Position>(entity)
            .map(|p| p.0)
            .map_err(|_| GameError::MissingComponent {
                guid,
                component: "Position",
            })
    }

    /// Доступ к произвольному компоненту на чтение.
    /// ```rust
    /// let stats = world.get::<Stats>(guid)?;
    /// println!("HP: {}/{}", stats.hp, stats.max_hp);
    /// ```
    pub fn get<C: hecs::Component>(&self, guid: ObjectGuid) -> Result<hecs::Ref<'_, C>, GameError> {
        let entity = self.entity(guid)?;
        self.world
            .get::<&C>(entity)
            .map_err(|_| GameError::MissingComponent {
                guid,
                component: std::any::type_name::<C>(),
            })
    }

    /// Доступ к произвольному компоненту на запись.
    pub fn get_mut<C: hecs::Component>(
        &self,
        guid: ObjectGuid,
    ) -> Result<hecs::RefMut<'_, C>, GameError> {
        let entity = self.entity(guid)?;
        self.world
            .get::<&mut C>(entity)
            .map_err(|_| GameError::MissingComponent {
                guid,
                component: std::any::type_name::<C>(),
            })
    }

    // -------------------------------------------------------------- Movement

    /// Переместить сущность. Проверяет коллизию с картой.
    pub fn move_entity(&mut self, guid: ObjectGuid, target: WorldPos) -> Result<(), GameError> {
        if self.map.is_solid_fast(target) {
            return Err(GameError::MovementBlocked(target));
        }

        let entity = self.entity(guid)?;
        let old_pos = {
            let mut pos = self.world.get::<&mut Position>(entity).map_err(|_| {
                GameError::MissingComponent {
                    guid,
                    component: "Position",
                }
            })?;
            let old = pos.0;
            pos.0 = target;
            old
        };

        self.grid.move_entity(guid, old_pos, target);
        Ok(())
    }

    // -------------------------------------------------------------- Combat

    /// Нанести урон. Если HP ≤ 0 — сущность помечается на удаление.
    pub fn deal_damage(
        &mut self,
        target: ObjectGuid,
        amount: i32,
    ) -> Result<DamageResult, GameError> {
        let entity = self.entity(target)?;

        let (actual_damage, killed) = {
            let mut stats =
                self.world
                    .get::<&mut Stats>(entity)
                    .map_err(|_| GameError::MissingComponent {
                        guid: target,
                        component: "Stats",
                    })?;

            let actual = amount.min(stats.hp);
            stats.hp -= actual;
            (actual, stats.hp <= 0)
        };

        if killed {
            self.commands.despawn(entity);
        }

        Ok(DamageResult {
            actual_damage,
            killed,
        })
    }

    // ------------------------------------------------------------ Lifecycle

    /// Удалить сущность (отложенно — применится в конце тика).
    pub fn despawn(&mut self, guid: ObjectGuid) -> Result<(), GameError> {
        let entity = self.entity(guid)?;
        self.registry.unregister(guid);
        self.grid
            .remove(guid, self.position(guid).unwrap_or(WorldPos::new(0, 0, 0)));
        self.commands.despawn(entity);
        Ok(())
    }

    // ---------------------------------------------------------------- Query

    /// Итерация по всем сущностям с нужными компонентами.
    /// ```rust
    /// for (entity, (pos, stats)) in world.query::<(&Position, &Stats)>() {
    ///     // ...
    /// }
    /// ```
    pub fn query<Q: hecs::Query>(&self) -> hecs::QueryBorrow<'_, Q> {
        self.world.query::<Q>()
    }

    // ----------------------------------------------------------------- Internal

    fn entity(&self, guid: ObjectGuid) -> Result<hecs::Entity, GameError> {
        self.registry
            .get_entity(guid)
            .ok_or(GameError::EntityNotFound(guid))
    }

    /// Доступ к игровым данным из Depot (read-only).
    /// ```rust
    /// let mat = world.depot().materials.get("stone").cloned();
    /// ```
    pub fn depot(&self) -> std::sync::RwLockReadGuard<'_, Option<cd_depot::Depot>> {
        self.game_data.read().unwrap()
    }
}
