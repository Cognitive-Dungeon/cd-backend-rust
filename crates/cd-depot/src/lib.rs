mod format;
pub mod accessor;
pub mod error;

use std::collections::HashMap;
use accessor::{SheetAccessor, LineAccessor};
use error::DepotError;
use format::DepotFile;

pub use accessor::{FromDepotLine, SheetAccessor as Sheet, LineAccessor as Line};
pub use error::DepotError as Error;

/// Загруженный .dpo файл.
/// Не знает про CreatureDef, MaterialDef и т.д. — это забота вызывающего кода.
///
/// ```rust
/// let depot = Depot::load("./data/game.dpo")?;
///
/// // Builder API — быстрый доступ без структур:
/// let hp = depot.sheet("Creatures")?.line("Dragon")?.int("Base Damage");
///
/// // Typed API — загрузить весь лист в HashMap:
/// let creatures: HashMap<String, CreatureDef> =
///     depot.sheet("Creatures")?.load_as_map();
/// ```
pub struct Depot {
    /// Только видимые листы (hidden: false)
    /// Ключ — name листа
    sheets: HashMap<String, Vec<serde_json::Value>>,
    /// Индекс guid → (sheet_name, line_index) для разрешения lineReference
    guid_index: HashMap<String, (String, usize)>,
}

impl Depot {
    pub fn load(path: &std::path::Path) -> Result<Self, DepotError> {
        let bytes = std::fs::read(path)
            .map_err(|e| DepotError::Io(e.to_string()))?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DepotError> {
        let file: DepotFile = serde_json::from_slice(bytes)
            .map_err(|e| DepotError::Parse(e.to_string()))?;

        let mut sheets: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
        let mut guid_index: HashMap<String, (String, usize)> = HashMap::new();

        for sheet in file.sheets {
            // Строим GUID индекс для ВСЕХ листов (включая hidden — они нужны для lineRef)
            for (idx, line) in sheet.lines.iter().enumerate() {
                if let Some(guid) = line["guid"].as_str() {
                    guid_index.insert(
                        guid.to_string(),
                        (sheet.name.clone(), idx),
                    );
                }
            }

            // В основной индекс кладём только видимые листы
            if !sheet.hidden {
                sheets.insert(sheet.name, sheet.lines);
            }
        }

        tracing::info!(
            "Depot loaded: {} sheets, {} guids indexed",
            sheets.len(),
            guid_index.len()
        );

        Ok(Self { sheets, guid_index })
    }

    /// Получить accessor для листа по имени.
    pub fn sheet(&self, name: &str) -> Option<SheetAccessor<'_>> {
        self.sheets.get(name).map(|lines| SheetAccessor { lines })
    }

    /// Разрешить lineReference: guid → LineAccessor любого листа.
    ///
    /// ```rust
    /// let creature_guid = spawn.line_ref("Data").unwrap();
    /// let creature_line = depot.resolve(creature_guid)?;
    /// let name = creature_line.text("TooltipText");
    /// ```
    pub fn resolve(&self, guid: &str) -> Option<LineAccessor<'_>> {
        let (sheet_name, idx) = self.guid_index.get(guid)?;
        let lines = self.sheets.get(sheet_name)?;
        lines.get(*idx).map(|data| LineAccessor { data })
    }

    /// Список всех видимых листов
    pub fn sheet_names(&self) -> Vec<&str> {
        self.sheets.keys().map(|s| s.as_str()).collect()
    }
}