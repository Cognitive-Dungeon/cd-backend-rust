use cd_core::{ObjectGuid, WorldPos};
use cd_map::Chunk;
use serde::{Deserialize, Serialize};
use crate::error::DataError;

// ----- WorldRepository -----

/// Port для работы с геометрией мира.
/// Движок зависит только от этого trait — не от файлов, не от БД.
pub trait WorldRepository: Send + Sync + 'static {
    /// Загрузить чанк по его chunk-координатам.
    fn load_chunk(&self, chunk_key: WorldPos) -> Result<Option<Chunk>, DataError>;

    /// Сохранить чанк.
    fn save_chunk(&self, chunk_key: WorldPos, chunk: &Chunk) -> Result<(), DataError>;
}

// ----- EntityRepository -----

/// Плоское представление сущности для персистентности.
/// Отдельно от EntitySnapshot (который для сети) — разные контракты.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedEntity {
    /// Сырое u64 значение GUID — не тащим сложности ObjectGuid в storage layer
    pub guid_raw: u64,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    /// Компоненты в свободном JSON — адаптер не знает их типов
    pub components: serde_json::Value,
}

pub trait EntityRepository: Send + Sync + 'static {
    fn load_entity(&self, guid: ObjectGuid) -> Result<Option<PersistedEntity>, DataError>;
    fn save_entity(&self, entity: &PersistedEntity) -> Result<(), DataError>;
    fn delete_entity(&self, guid: ObjectGuid) -> Result<(), DataError>;
}