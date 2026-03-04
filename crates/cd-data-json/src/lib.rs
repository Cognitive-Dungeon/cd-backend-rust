mod dto;

use cd_core::{ObjectGuid, WorldPos};
use cd_data::{DataError, EntityRepository, PersistedEntity, WorldRepository};
use cd_map::Chunk;
use dto::ChunkDto;
use std::path::{Path, PathBuf};

// ----- JsonWorldRepository -----

pub struct JsonWorldRepository {
    base_path: PathBuf,
}

impl JsonWorldRepository {
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self, DataError> {
        let path = base_path.as_ref().join("chunks");
        std::fs::create_dir_all(&path)?;
        Ok(Self { base_path: path })
    }

    /// Имя файла: `chunk_{x}_{y}_{z}.json`
    fn chunk_path(&self, key: WorldPos) -> PathBuf {
        self.base_path
            .join(format!("chunk_{}_{}_{}.json", key.x(), key.y(), key.z()))
    }
}

impl WorldRepository for JsonWorldRepository {
    fn load_chunk(&self, chunk_key: WorldPos) -> Result<Option<Chunk>, DataError> {
        let path = self.chunk_path(chunk_key);

        match std::fs::read(&path) {
            Ok(bytes) => {
                let dto: ChunkDto = serde_json::from_slice(&bytes)
                    .map_err(|e| DataError::Deserialize(e.to_string()))?;
                let chunk = dto.into_chunk().map_err(|e| DataError::Deserialize(e))?;
                Ok(Some(chunk))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(DataError::Io(e)),
        }
    }

    fn save_chunk(&self, chunk_key: WorldPos, chunk: &Chunk) -> Result<(), DataError> {
        let dto = ChunkDto::from_chunk(chunk);
        let bytes =
            serde_json::to_vec_pretty(&dto).map_err(|e| DataError::Serialize(e.to_string()))?;
        std::fs::write(self.chunk_path(chunk_key), bytes)?;
        Ok(())
    }
}

// ----- JsonEntityRepository -----

pub struct JsonEntityRepository {
    base_path: PathBuf,
}

impl JsonEntityRepository {
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self, DataError> {
        let path = base_path.as_ref().join("entities");
        std::fs::create_dir_all(&path)?;
        Ok(Self { base_path: path })
    }

    fn entity_path(&self, guid: ObjectGuid) -> PathBuf {
        self.base_path
            .join(format!("entity_{}.json", guid.as_u64()))
    }
}

impl EntityRepository for JsonEntityRepository {
    fn load_entity(&self, guid: ObjectGuid) -> Result<Option<PersistedEntity>, DataError> {
        let path = self.entity_path(guid);
        match std::fs::read(&path) {
            Ok(bytes) => {
                let entity = serde_json::from_slice(&bytes)
                    .map_err(|e| DataError::Deserialize(e.to_string()))?;
                Ok(Some(entity))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(DataError::Io(e)),
        }
    }

    fn save_entity(&self, entity: &PersistedEntity) -> Result<(), DataError> {
        let bytes =
            serde_json::to_vec_pretty(entity).map_err(|e| DataError::Serialize(e.to_string()))?;
        use cd_core::ObjectGuid as G;
        let guid = G::from_raw(entity.guid_raw);
        std::fs::write(self.entity_path(guid), bytes)?;
        Ok(())
    }

    fn delete_entity(&self, guid: ObjectGuid) -> Result<(), DataError> {
        let path = self.entity_path(guid);
        match std::fs::remove_file(&path) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(DataError::Io(e)),
        }
    }
}
