//! Интеграция с Bevy ECS. Оборачивает чистые BRP-типы в компоненты и системы.
pub mod components;
pub mod systems;

pub use components::*;
pub use systems::*;
