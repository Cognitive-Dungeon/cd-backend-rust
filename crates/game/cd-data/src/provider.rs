use crate::defs::{CreatureDef, FurnitureDef, MaterialDef, SpellDef};
use crate::error::DataError;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Абстрактный источник данных (может быть RON-файлы, может быть БД)
pub trait DataProvider: Send + Sync + 'static {
    fn load_creatures(&self) -> Result<HashMap<String, CreatureDef>, DataError>;
    fn load_materials(&self) -> Result<HashMap<String, MaterialDef>, DataError>;
    fn load_furniture(&self) -> Result<HashMap<String, FurnitureDef>, DataError>;
    fn load_spells(&self) -> Result<HashMap<String, SpellDef>, DataError>;
}

/// Реализация провайдера для локальных RON файлов
pub struct RonDataProvider {
    base_path: PathBuf,
}

impl RonDataProvider {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    fn load_file<T: serde::de::DeserializeOwned>(
        &self,
        filename: &str,
    ) -> Result<HashMap<String, T>, DataError> {
        let path = self.base_path.join(filename);
        let content = fs::read_to_string(&path).map_err(DataError::Io)?;
        ron::from_str(&content)
            .map_err(|e| DataError::Deserialize(format!("RON error in {}: {}", filename, e)))
    }
}

impl DataProvider for RonDataProvider {
    fn load_creatures(&self) -> Result<HashMap<String, CreatureDef>, DataError> {
        self.load_file("creatures.ron")
    }

    fn load_materials(&self) -> Result<HashMap<String, MaterialDef>, DataError> {
        self.load_file("materials.ron")
    }

    fn load_furniture(&self) -> Result<HashMap<String, FurnitureDef>, DataError> {
        self.load_file("furniture.ron")
    }

    fn load_spells(&self) -> Result<HashMap<String, SpellDef>, DataError> {
        self.load_file("spells.ron")
    }
}
