use bevy::ecs::component::Component;

#[derive(Debug, Clone, Copy, Component)]
pub struct IsDead;

#[derive(Debug, Clone, Copy, Component)]
pub struct IsAgent;
