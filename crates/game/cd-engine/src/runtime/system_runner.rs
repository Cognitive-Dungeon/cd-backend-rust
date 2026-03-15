use cd_telemetry::{EngineEvent, TelemetrySink};

use crate::game_error::GameError;
use crate::game_world::GameWorld;
use crate::tick::{TickContext, TickId};

/// Сигнатура игровой системы.
/// Системы получают фасад `GameWorld` — никакого сырого hecs.
pub type SystemFn =
    Box<dyn Fn(&mut GameWorld, &mut TickContext) -> Result<(), GameError> + Send + 'static>;

struct RegisteredSystem {
    name:  &'static str,
    func:  SystemFn,
}

/// Реестр систем. Хранится в Engine, используется каждый тик.
#[derive(Default)]
pub struct SystemRunner {
    systems: Vec<RegisteredSystem>,
}

impl SystemRunner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Зарегистрировать систему.
    /// ```rust
    /// engine.register_system("movement", systems::movement::run);
    /// engine.register_system("fire_spread", |world, _ctx| { Ok(()) });
    /// ```
    pub fn register(
        &mut self,
        name: &'static str,
        f: impl Fn(&mut GameWorld, &mut TickContext) -> Result<(), GameError> + Send + 'static,
    ) {
        self.systems.push(RegisteredSystem { name, func: Box::new(f) });
    }

    /// Запустить все системы. Ошибки изолируются — одна упавшая система
    /// не останавливает остальные.
    pub fn run(
        &mut self,
        world: &mut GameWorld,
        ctx:   &mut TickContext,
        telemetry: &dyn TelemetrySink,
    ) {
        for system in &self.systems {
            if let Err(e) = (system.func)(world, ctx) {
                // Ошибка системы → телеметрия, движок продолжает работать
                tracing::warn!("System '{}' error: {}", system.name, e);
                telemetry.emit(EngineEvent::ErrorIsolated {
                    tick_id: ctx.tick_id.0,
                    context: system.name.to_string(),
                    error:   e.to_string(),
                });
            }
        }
    }
}

/// Макрос для объявления игровых систем без boilerplate.
///
/// # Примеры
///
/// Система без контекста тика:
/// ```rust
/// game_system!(pub fn regen_system(world) {
///     for (_e, stats) in world.query::<&mut Stats>().iter() {
///         if stats.hp < stats.max_hp { stats.hp += 1; }
///     }
///     Ok(())
/// });
/// ```
///
/// Система с доступом к RNG через контекст:
/// ```rust
/// game_system!(pub fn random_event(world, ctx) {
///     let roll = ctx.rng.next_u32() % 100;
///     Ok(())
/// });
/// ```
#[macro_export]
macro_rules! game_system {
    // Вариант с ctx (доступ к RNG и tick_id)
    (pub fn $name:ident($world:ident, $ctx:ident) $body:block) => {
        pub fn $name(
            $world: &mut $crate::game_world::GameWorld,
            $ctx:   &mut $crate::tick::TickContext,
        ) -> Result<(), $crate::game_error::GameError> {
            $body
        }
    };

    // Вариант без ctx (большинство систем)
    (pub fn $name:ident($world:ident) $body:block) => {
        pub fn $name(
            $world: &mut $crate::game_world::GameWorld,
            _ctx:   &mut $crate::tick::TickContext,
        ) -> Result<(), $crate::game_error::GameError> {
            $body
        }
    };
}