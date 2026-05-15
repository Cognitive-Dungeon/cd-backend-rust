use bevy::{ecs::component::Component, ecs::reflect::ReflectComponent, reflect::Reflect};

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct IsDead;

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct IsAgent;
