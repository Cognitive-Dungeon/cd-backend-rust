pub mod error;
pub mod game_error;
pub mod input;
pub mod runtime;
pub mod systems;
pub mod watcher;
pub mod world;

pub use cd_data::depot::{Depot, FromDepotLine};
pub use cd_telemetry::{BroadcastSink, EngineEvent, NullSink, TelemetrySink};
pub use command_bus::{CommandBus, CommandSender, StampedCommand};
pub use engine::Engine;
pub use game_error::{DamageResult, GameError};
pub use input::InputCmd;
pub use tick::{TickContext, TickId};

pub use builder::EngineBuilder;

pub use error::EngineError;

use crate::{
    input::command_bus,
    runtime::{builder, engine, tick},
};
