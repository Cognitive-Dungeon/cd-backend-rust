use std::sync::Arc;
use cd_data::{WorldRepository, EntityRepository};
use cd_telemetry::{TelemetrySink, NullSink};
use crate::engine::Engine;
use crate::tick::TickId;

/// Builder для Engine — единственное место, где собираются зависимости.
/// Добавить новую зависимость = добавить поле + метод здесь.
#[derive(Default)]
pub struct EngineBuilder {
    world_seed: Option<u64>,
    telemetry: Option<Arc<dyn TelemetrySink>>,
    world_repo: Option<Arc<dyn WorldRepository>>,
    entity_repo: Option<Arc<dyn EntityRepository>>,
}

impl EngineBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn world_seed(mut self, seed: u64) -> Self {
        self.world_seed = Some(seed);
        self
    }

    pub fn telemetry(mut self, sink: Arc<dyn TelemetrySink>) -> Self {
        self.telemetry = Some(sink);
        self
    }

    pub fn world_repo(mut self, repo: Arc<dyn WorldRepository>) -> Self {
        self.world_repo = Some(repo);
        self
    }

    pub fn entity_repo(mut self, repo: Arc<dyn EntityRepository>) -> Self {
        self.entity_repo = Some(repo);
        self
    }

    pub fn build(self) -> Engine {
        Engine::from_builder(
            self.world_seed.unwrap_or(0xDEAD_CAFE_BABE_1337),
            self.telemetry.unwrap_or_else(|| Arc::new(NullSink)),
            self.world_repo,
            self.entity_repo,
        )
    }
}