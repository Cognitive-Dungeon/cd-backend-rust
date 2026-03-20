use serde_json::Value;

/// Accessor для одной строки листа.
/// Позволяет читать поля без определения структуры.
///
/// ```rust
/// let hp = depot.sheet("Creatures")?.line("Dragon")?.int("Base Damage");
/// let flying = depot.sheet("Creatures")?.line("Dragon")?.bool("Flying");
/// ```
pub struct LineAccessor<'a> {
    pub data: &'a Value,
}

impl<'a> LineAccessor<'a> {
    /// GUID строки (первичный ключ Depot)
    pub fn guid(&self) -> &str {
        self.data["guid"].as_str().unwrap_or("")
    }

    /// ID строки (человекочитаемый ключ)
    pub fn id(&self) -> &str {
        self.data["id"].as_str().unwrap_or("")
    }

    pub fn int(&self, field: &str) -> i64 {
        self.data[field].as_i64().unwrap_or(0)
    }

    pub fn int_or(&self, field: &str, default: i64) -> i64 {
        self.data[field].as_i64().unwrap_or(default)
    }

    pub fn float(&self, field: &str) -> f64 {
        self.data[field].as_f64().unwrap_or(0.0)
    }

    pub fn float_or(&self, field: &str, default: f64) -> f64 {
        self.data[field].as_f64().unwrap_or(default)
    }

    pub fn bool(&self, field: &str) -> bool {
        self.data[field].as_bool().unwrap_or(false)
    }

    pub fn text(&self, field: &str) -> &str {
        self.data[field].as_str().unwrap_or("")
    }

    /// Возвращает hex-строку цвета. Если цвета нет, возвращает белый ("#ffffff")
    pub fn color(&self, field: &str) -> &str {
        let s = self.data[field].as_str().unwrap_or("");
        if s.is_empty() { "#ffffff" } else { s }
    }

    /// Возвращает hex-строку цвета с кастомным значением по умолчанию
    pub fn color_or<'b>(&'b self, field: &str, default: &'b str) -> &'b str {
        let s = self.data[field].as_str().unwrap_or("");
        if s.is_empty() { default } else { s }
    }

    pub fn char(&self, field: &str) -> char {
        self.data[field]
            .as_str()
            .and_then(|s| s.chars().next())
            .unwrap_or('?')
    }

    /// Enum / SingleSelect — возвращает выбранное значение
    pub fn enum_val(&self, field: &str) -> &str {
        self.data[field].as_str().unwrap_or("")
    }

    /// Multiple / MultiSelect — возвращает выбранные значения
    pub fn multiple(&self, field: &str) -> Vec<&str> {
        self.data[field]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// LineReference — возвращает GUID целевой строки
    pub fn line_ref(&self, field: &str) -> Option<&str> {
        let s = self.data[field].as_str()?;
        if s.is_empty() { None } else { Some(s) }
    }

    /// List — возвращает вложенные строки как Vec<LineAccessor>
    pub fn list(&self, field: &str) -> Vec<LineAccessor<'_>> {
        self.data[field]
            .as_array()
            .map(|arr| arr.iter().map(|v| LineAccessor { data: v }).collect())
            .unwrap_or_default()
    }

    /// Сырой JSON — для нестандартных колонок (props, grid)
    pub fn raw(&self, field: &str) -> &Value {
        &self.data[field]
    }

    /// Десериализовать строку в типизированную структуру через serde.
    /// Работает если имена полей в структуре совпадают с именами в .dpo
    pub fn deserialize<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.data.clone())
    }
}

/// Accessor для листа — итерация и поиск строк.
pub struct SheetAccessor<'a> {
    pub(crate) lines: &'a [Value],
}

impl<'a> SheetAccessor<'a> {
    /// Найти строку по id колонке
    pub fn line(&self, id: &str) -> Option<LineAccessor<'_>> {
        self.lines
            .iter()
            .find(|v| v["id"].as_str() == Some(id))
            .map(|data| LineAccessor { data })
    }

    /// Найти строку по guid
    pub fn by_guid(&self, guid: &str) -> Option<LineAccessor<'_>> {
        self.lines
            .iter()
            .find(|v| v["guid"].as_str() == Some(guid))
            .map(|data| LineAccessor { data })
    }

    /// Итерация по всем строкам
    pub fn iter(&self) -> impl Iterator<Item = LineAccessor<'_>> {
        self.lines.iter().map(|data| LineAccessor { data })
    }

    /// Количество строк
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Загрузить все строки в типизированный Vec через FromDepotLine
    pub fn load_all<T: FromDepotLine>(&self) -> Vec<T> {
        self.lines
            .iter()
            .filter_map(|v| {
                let acc = LineAccessor { data: v };
                match T::from_depot_line(&acc) {
                    Ok(t) => Some(t),
                    Err(e) => {
                        tracing::warn!(
                            "FromDepotLine failed for id={:?}: {}",
                            v["id"].as_str().unwrap_or("?"),
                            e
                        );
                        None
                    }
                }
            })
            .collect()
    }

    /// Загрузить все строки в HashMap<id, T>
    pub fn load_as_map<T: FromDepotLine>(&self) -> std::collections::HashMap<String, T> {
        self.lines
            .iter()
            .filter_map(|v| {
                let acc = LineAccessor { data: v };
                let id = v["id"].as_str()?.to_string();
                match T::from_depot_line(&acc) {
                    Ok(t) => Some((id, t)),
                    Err(e) => {
                        tracing::warn!("FromDepotLine failed for id={:?}: {}", id, e);
                        None
                    }
                }
            })
            .collect()
    }
}

/// Trait для типизированного маппинга строки Depot → твоя структура.
///
/// Реализуй для своих игровых структур:
/// ```rust
/// struct CreatureDef { id: String, base_damage: i32, flying: bool }
///
/// impl FromDepotLine for CreatureDef {
///     fn from_depot_line(line: &LineAccessor) -> Result<Self, String> {
///         Ok(Self {
///             id:          line.id().to_string(),
///             base_damage: line.int("Base Damage") as i32,
///             flying:      line.bool("Flying"),
///         })
///     }
/// }
/// ```
pub trait FromDepotLine: Sized {
    fn from_depot_line(line: &LineAccessor<'_>) -> Result<Self, String>;
}
