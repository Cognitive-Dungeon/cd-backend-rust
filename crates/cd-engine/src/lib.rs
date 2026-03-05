pub mod input;
pub mod engine;
pub mod snapshot;
pub mod systems;
pub mod command_bus;
pub mod tick;
mod registry;
pub mod builder;
pub mod game_error;
pub mod game_world;
pub mod system_runner;
pub mod watcher;

pub use engine::Engine;
pub use input::InputCmd;
pub use command_bus::{CommandBus, CommandSender, StampedCommand};
pub use tick::{TickId, TickContext};
pub use cd_telemetry::{TelemetrySink, NullSink, BroadcastSink, EngineEvent};
pub use game_error::{GameError, DamageResult};
pub use game_world::GameWorld;
pub use system_runner::SystemRunner;
pub use cd_depot::{Depot, FromDepotLine};

pub use builder::EngineBuilder;

pub mod error;
pub use error::EngineError;
pub use snapshot::EntitySnapshot;