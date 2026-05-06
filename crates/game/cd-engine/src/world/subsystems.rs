use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use cd_core::{ObjectGuid, WorldPos};
use cd_data::defs::SpellEffect;
use cd_ecs::{Guid, IsDead, Stats};
use cd_telemetry::EngineEvent;

use crate::world::resources::{
    GridResource, MapResource, RegistryResource, TelemetryResource, TickResource,
};

/// Фасад для работы с пространством и индексами.
/// Группирует в себе Карту (статика), Сетку (динамика) и Реестр (поиск сущностей).
#[derive(SystemParam)]
pub struct SpatialSubsystem<'w> {
    pub map: Res<'w, MapResource>,
    pub grid: ResMut<'w, GridResource>,
    pub registry: ResMut<'w, RegistryResource>,
}

impl<'w> SpatialSubsystem<'w> {
    /// Регистрирует сущность в пространственных индексах.
    pub fn register_entity(&mut self, guid: ObjectGuid, entity: Entity, pos: WorldPos) {
        self.registry.inner.register(guid, entity);
        self.grid.inner.insert(guid, pos);
    }

    /// Удаляет сущность из индексов (например, при смерти или отключении).
    pub fn unregister_entity(&mut self, guid: ObjectGuid, pos: WorldPos) {
        self.registry.inner.unregister(guid);
        self.grid.inner.remove(guid, pos);
    }

    /// Обновляет позицию в пространственном индексе.
    pub fn move_entity(&mut self, guid: ObjectGuid, old_pos: WorldPos, new_pos: WorldPos) {
        self.grid.inner.move_entity(guid, old_pos, new_pos);
    }

    /// Проверяет, заблокирован ли тайл статической картой (стенами).
    pub fn is_solid_map(&self, pos: WorldPos) -> bool {
        self.map.inner.is_solid_fast(pos)
    }

    /// Возвращает список GUID всех сущностей в том же bucket'е (чанке), что и точка.
    pub fn get_entities_in_bucket(&self, pos: WorldPos) -> &[ObjectGuid] {
        self.grid.inner.query_bucket(pos)
    }

    /// Ищет Bevy Entity по постоянному GUID.
    pub fn get_entity(&self, guid: ObjectGuid) -> Option<Entity> {
        self.registry.inner.get_entity(guid)
    }
}

/// Фасад для боевой системы.
/// Берет на себя всю логику урона, лечения, смерти и телеметрии.
#[derive(SystemParam)]
pub struct CombatSubsystem<'w, 's> {
    // Дробим один огромный Query на маленькие и независимые!
    stats: Query<'w, 's, &'static mut Stats>,
    dead: Query<'w, 's, &'static IsDead>,
    names: Query<'w, 's, &'static cd_ecs::components::Name>,
    guids: Query<'w, 's, &'static Guid>,

    commands: Commands<'w, 's>,
    telemetry: Res<'w, TelemetryResource>,
    tick: Res<'w, TickResource>,
}

impl<'w, 's> CombatSubsystem<'w, 's> {
    pub fn is_alive(&self, entity: Entity) -> bool {
        // Если висит маркер смерти — точно мертв
        if self.dead.get(entity).is_ok() {
            return false;
        }

        // Ищем статы. Если их нет, выводим конкретную ошибку
        match self.stats.get(entity) {
            Ok(stats) => stats.hp > 0,
            Err(_) => {
                tracing::warn!(
                    "CombatSubsystem: Entity {:?} lacks Stats component!",
                    entity
                );
                false
            }
        }
    }

    pub fn apply_effect(&mut self, target: Entity, effect: &SpellEffect) {
        match effect {
            SpellEffect::Damage { amount, .. } => self.apply_damage(target, *amount),
            SpellEffect::Heal { amount } => self.apply_heal(target, *amount),
        }
    }

    pub fn apply_damage(&mut self, target: Entity, amount: i32) {
        if self.dead.get(target).is_ok() {
            return; // Уже мертв
        }

        let Ok(mut stats) = self.stats.get_mut(target) else {
            return;
        };
        if stats.hp <= 0 {
            return;
        }

        // Достаем имя и guid безопасно (даже если их нет, логика не сломается)
        let guid = self
            .guids
            .get(target)
            .map(|g| g.0)
            .unwrap_or(cd_core::ObjectGuid::NIL);
        let name_str = self
            .names
            .get(target)
            .map(|n| n.0.clone())
            .unwrap_or_else(|_| "Unknown".to_string());

        stats.hp = (stats.hp - amount).max(0);
        tracing::info!(target = %guid, name = %name_str, damage = amount, hp_left = stats.hp, "Damage applied");

        self.telemetry.0.emit(EngineEvent::EntityDamaged {
            tick_id: self.tick.id.0,
            guid: guid.to_string(),
            amount,
            hp_left: stats.hp,
        });

        if stats.hp == 0 {
            tracing::info!("{} ({}) died!", name_str, guid);
            self.commands.entity(target).insert(IsDead);
            self.telemetry.0.emit(EngineEvent::EntityDied {
                tick_id: self.tick.id.0,
                guid: guid.to_string(),
            });
        }
    }

    pub fn apply_heal(&mut self, target: Entity, amount: i32) {
        if self.dead.get(target).is_ok() {
            return;
        }

        let Ok(mut stats) = self.stats.get_mut(target) else {
            return;
        };
        if stats.hp <= 0 {
            return;
        }

        stats.hp = (stats.hp + amount).min(stats.max_hp);

        let guid = self
            .guids
            .get(target)
            .map(|g| g.0)
            .unwrap_or(cd_core::ObjectGuid::NIL);
        let name_str = self
            .names
            .get(target)
            .map(|n| n.0.clone())
            .unwrap_or_else(|_| "Unknown".to_string());

        tracing::info!(target = %guid, name = %name_str, heal = amount, hp = stats.hp, "Heal applied");
    }
}
