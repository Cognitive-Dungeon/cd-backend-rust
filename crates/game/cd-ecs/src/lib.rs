pub mod bundles;
pub mod components;
pub mod registry;
pub mod spatial_grid;

pub use bundles::*;
pub use components::*;
pub use registry::EntityRegistry;
pub use spatial_grid::SpatialGrid;

/// Плагин ECS-компонентов.
/// Его единственная задача — зарегистрировать типы для рефлексии (Инспектора).
pub struct CdEcsPlugin;

impl bevy::app::Plugin for CdEcsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.register_type::<components::Position>()
            .register_type::<components::Guid>()
            .register_type::<components::Render>()
            .register_type::<components::Controller>()
            .register_type::<components::Creature>()
            .register_type::<components::Furniture>()
            .register_type::<components::Door>()
            .register_type::<components::IsDead>()
            .register_type::<components::IsAgent>()
            .register_type::<components::InstanceId>()
            .register_type::<components::InstanceState>();
    }
}
