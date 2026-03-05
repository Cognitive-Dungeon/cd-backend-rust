use serde::Deserialize;
use serde_json::Value;

/// Корень .cdb файла
#[derive(Debug, Deserialize)]
pub struct DepotFile {
    pub sheets: Vec<Sheet>,
}

/// Один лист
#[derive(Debug, Deserialize)]
pub struct Sheet {
    pub name:    String,
    pub columns: Vec<Column>,
    pub lines:   Vec<Value>,
    pub guid:    String,
    #[serde(default)]
    pub description: String,
    /// Скрытые листы — внутренние для list/props колонок
    #[serde(default)]
    pub hidden:  bool,
    #[serde(rename = "isProps", default)]
    pub is_props: bool,
    #[serde(default)]
    pub separators: Vec<Value>,
}

/// Тип колонки — точные строки из реального Depot файла
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub enum ColumnKind {
    #[serde(rename = "int")]             Int,
    #[serde(rename = "float")]           Float,
    #[serde(rename = "bool")]            Bool,
    #[serde(rename = "text")]            Text,
    #[serde(rename = "longtext")]        LongText,
    #[serde(rename = "image")]           Image,
    #[serde(rename = "file")]            File,
    /// Single select ("enum" в реальном файле)
    #[serde(rename = "enum")]            Enum,
    /// Multi select ("multiple" в реальном файле)
    #[serde(rename = "multiple")]        Multiple,
    #[serde(rename = "sheetReference")]  SheetReference,
    #[serde(rename = "lineReference")]   LineReference,
    #[serde(rename = "list")]            List,
    /// Properties ("props" в реальном файле)
    #[serde(rename = "props")]           Props,
    #[serde(rename = "grid")]            Grid,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct Column {
    pub name:    String,
    #[serde(rename = "typeStr")]
    pub kind:    ColumnKind,
    pub guid:    String,
    #[serde(default)]
    pub description: String,
    /// Для enum/multiple — опции через запятую: "fire, ice, lightning"
    #[serde(default)]
    pub options: Option<String>,
    /// Для lineReference/sheetReference/list/props — GUID целевого листа
    #[serde(default)]
    pub sheet:   Option<String>,
    /// Для grid — типы каждой ячейки
    #[serde(default)]
    pub schema:  Vec<String>,
    #[serde(default)]
    pub length:  Option<u32>,
}

impl Column {
    /// Парсит строку опций ("fire, ice") в Vec<String>
    pub fn parsed_options(&self) -> Vec<String> {
        self.options.as_deref()
            .unwrap_or("")
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}