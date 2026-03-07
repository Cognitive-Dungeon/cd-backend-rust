pub mod builder;
pub mod command_bus;
pub mod engine;
pub mod game_error;
pub mod game_world;
pub mod input;
mod registry;
pub mod snapshot;
pub mod system_runner;
pub mod systems;
pub mod tick;
pub mod watcher;

pub use cd_data::depot::{Depot, FromDepotLine};
pub use cd_telemetry::{BroadcastSink, EngineEvent, NullSink, TelemetrySink};
pub use command_bus::{CommandBus, CommandSender, StampedCommand};
pub use engine::Engine;
pub use game_error::{DamageResult, GameError};
pub use game_world::GameWorld;
pub use input::InputCmd;
pub use system_runner::SystemRunner;
pub use tick::{TickContext, TickId};

pub use builder::EngineBuilder;

pub mod error;
pub use error::EngineError;
pub use snapshot::EntitySnapshot;
