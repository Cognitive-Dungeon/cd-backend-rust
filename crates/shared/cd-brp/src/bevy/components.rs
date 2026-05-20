use bevy::ecs::component::Component;
use cd_core::ObjectGuid;

use crate::{HitPoints, PowerPoints, domain::CharacteristicBlock};

// bevy/components.rs
// Оборачиваем наши чистые типы в компоненты Bevy
#[derive(Component)]
pub struct BRPCharacteristics(pub CharacteristicBlock);

#[derive(Component)]
pub struct BRPVitals {
    pub hp: HitPoints,
    pub mp: PowerPoints,
}

/// Компонент, привязывающий глобальный MMO ID к локальной сущности Bevy
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NetworkId(pub ObjectGuid);
