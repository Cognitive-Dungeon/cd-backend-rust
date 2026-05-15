use bevy::ecs::entity::Entity;
use bevy::ecs::prelude::*;
use bevy::ecs::system::SystemParam;
use cd_core::{ObjectGuid, WorldPos};
use cd_data::defs::SpellEffect;
use cd_ecs::{CombatBubble, Guid, InCombat, InstanceId, IsDead, Stats};
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
    pub fn register_entity(
        &mut self,
        instance: InstanceId,
        guid: ObjectGuid,
        entity: Entity,
        pos: WorldPos,
    ) {
        self.registry.inner.register(guid, entity);
        self.grid.inner.insert(instance, guid, pos);
    }

    /// Удаляет сущность из индексов (например, при смерти или отключении).
    pub fn unregister_entity(&mut self, instance: InstanceId, guid: ObjectGuid, pos: WorldPos) {
        self.registry.inner.unregister(guid);
        self.grid.inner.remove(instance, guid, pos);
    }

    /// Обновляет позицию в пространственном индексе.
    pub fn move_entity(
        &mut self,
        instance: InstanceId,
        guid: ObjectGuid,
        old_pos: WorldPos,
        new_pos: WorldPos,
    ) {
        self.grid
            .inner
            .move_entity(instance, guid, old_pos, new_pos);
    }

    /// Проверяет, заблокирован ли тайл статической картой (стенами).
    pub fn is_solid_map(&self, instance: InstanceId, pos: WorldPos) -> bool {
        self.map
            .get_map(instance)
            .map(|m| m.is_solid_fast(pos))
            .unwrap_or(false)
    }

    /// Возвращает список GUID всех сущностей в том же bucket'е (чанке), что и точка.
    pub fn get_entities_in_bucket(&self, instance: InstanceId, pos: WorldPos) -> &[ObjectGuid] {
        self.grid.inner.query_bucket(instance, pos)
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
    names: Query<'w, 's, &'static Name>,
    guids: Query<'w, 's, &'static Guid>,

    in_combat: Query<'w, 's, &'static mut InCombat>,
    bubbles: Query<'w, 's, &'static CombatBubble>,

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
            .cloned()
            .unwrap_or_else(|_| "Unknown".into());

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
            .cloned()
            .unwrap_or_else(|_| "Unknown".into());

        tracing::info!(target = %guid, name = %name_str, heal = amount, hp = stats.hp, "Heal applied");
    }

    /// Списывает AP. Если сущность НЕ в бою, ничего не списывает (возвращает Ok).
    pub fn try_consume_ap(&mut self, entity: Entity, amount: i32) -> Result<(), &'static str> {
        if let Ok(mut combat) = self.in_combat.get_mut(entity)
            && let Ok(bubble) = self.bubbles.get(combat.bubble)
        {
            if bubble.current_actor() != Some(entity) {
                return Err("Not your turn!");
            }
            if combat.action_points < amount {
                return Err("Not enough Action Points!");
            }
            combat.action_points -= amount;
            tracing::info!("Consumed {} AP. Left: {}", amount, combat.action_points);
            return Ok(());
        }
        Ok(()) // Вне боя действия бесплатны
    }

    /// Списывает MP (шаги). Аналогично AP.
    pub fn try_consume_mp(&mut self, entity: Entity, amount: i32) -> Result<(), &'static str> {
        if let Ok(mut combat) = self.in_combat.get_mut(entity)
            && let Ok(bubble) = self.bubbles.get(combat.bubble)
        {
            if bubble.current_actor() != Some(entity) {
                return Err("Not your turn!");
            }
            if combat.movement_points < amount {
                return Err("Not enough Movement Points!");
            }
            combat.movement_points -= amount;
            tracing::info!("Consumed {} MP. Left: {}", amount, combat.movement_points);
            return Ok(());
        }
        Ok(())
    }

    /// Начинает бой: создает CombatBubble и втягивает всех вокруг.
    pub fn initiate_combat(
        &mut self,
        initiator: Entity,
        instance: InstanceId,
        center_pos: WorldPos,
        spatial: &SpatialSubsystem,
    ) {
        // Если инициатор уже в бою — пузырь не создаем
        if self.in_combat.get(initiator).is_ok() {
            return;
        }

        tracing::info!("⚔️ Initiating COMBAT BUBBLE around {:?}", center_pos);

        // 1. Ищем всех в радиусе 16 тайлов (1 чанк)
        let nearby_guids = spatial.grid.inner.query_radius(instance, center_pos, 16);
        let mut participants = Vec::new();

        for guid in nearby_guids {
            if let Some(entity) = spatial.get_entity(guid) {
                // Берем только живых и тех, кто еще не сражается
                if self.is_alive(entity) && self.in_combat.get(entity).is_err() {
                    participants.push(entity);
                }
            }
        }

        if participants.len() < 2 {
            tracing::info!("Not enough participants to start combat.");
            return;
        }

        // 2. Инициатор ходит первым! (Сортируем остальных для детерминизма)
        participants.retain(|&e| e != initiator);
        participants.sort();
        participants.insert(0, initiator);

        // 3. Создаем сущность-менеджер боя
        let bubble_entity = self
            .commands
            .spawn(CombatBubble {
                turn_order: participants.clone(),
                current_turn_idx: 0,
                round: 1,
            })
            .id();

        // 4. Вешаем на всех маркер "Я в бою"
        for &actor in &participants {
            self.commands.entity(actor).insert(InCombat {
                bubble: bubble_entity,
                action_points: 6,    // Стартовые AP
                movement_points: 10, // Стартовые MP
            });

            let name = self
                .names
                .get(actor)
                .map(|n| n.as_str().to_string())
                .unwrap_or_default();
            tracing::info!("🛡️ {} joined the combat!", name);
        }

        tracing::info!("🔥 Combat started! Round 1. It is {:?}'s turn.", initiator);
    }
}
