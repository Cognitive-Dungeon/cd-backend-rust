use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};

/// Монотонный идентификатор тика
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TickId(pub u64);

impl TickId {
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl std::fmt::Display for TickId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tick#{}", self.0)
    }
}

/// Контекст одного тика — передаётся в системы
/// Гарантирует детерминизм: один seed → один результат
pub struct TickContext {
    pub tick_id: TickId,
    /// Детерминированный RNG: seed = world_seed XOR tick_id
    /// Системы используют ТОЛЬКО этот rng, никаких thread_local/SystemTime
    pub rng: ChaCha8Rng,
}

impl TickContext {
    pub fn new(world_seed: u64, tick_id: TickId) -> Self {
        Self {
            tick_id,
            rng: ChaCha8Rng::seed_from_u64(world_seed ^ tick_id.0),
        }
    }
}