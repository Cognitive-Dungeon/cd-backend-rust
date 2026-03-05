use std::collections::HashMap;
use serde_json::Value;
use crate::defs::{CreatureDef, ItemDef, MaterialDef, SpellDef};
use crate::error::DepotError;
use crate::format::DepotFile;

/// Загруженные игровые данные.
/// Хранится в Engine, доступен через GameWorld::data().
/// При горячей перезагрузке заменяется целиком.
#[derive(Debug, Default, Clone)]
pub struct GameData {
    pub materials: HashMap<String, MaterialDef>,
    pub creatures: HashMap<String, CreatureDef>,
    pub items:     HashMap<String, ItemDef>,
    pub spells:    HashMap<String, SpellDef>,

    /// Индекс по guid для разрешения Line References:
    /// data.resolve_material_guid("abc-123") -> &MaterialDef
    pub materials_by_guid: HashMap<String, String>,  // guid -> id
    pub creatures_by_guid: HashMap<String, String>,

    /// Листы, которые движок не знает — хранятся как сырой JSON.
    /// Игровые системы могут читать их через data.raw_sheet("MyCustomSheet").
    pub raw: HashMap<String, Vec<Value>>,
}

impl GameData {
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, DepotError> {
        let bytes = std::fs::read(path)
            .map_err(|e| DepotError::Io(e.to_string()))?;
        Self::load_from_bytes(&bytes)
    }

    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self, DepotError> {
        let file: DepotFile = serde_json::from_slice(bytes)
            .map_err(|e| DepotError::Parse(e.to_string()))?;
        Self::from_depot(file)
    }

    fn from_depot(file: DepotFile) -> Result<Self, DepotError> {
        let mut data = Self::default();

        for sheet in file.sheets {
            if sheet.hidden { continue; }
            match sheet.name.as_str() {
                "Materials" => {
                    for line in &sheet.lines {
                        match parse_material(line) {
                            Ok(m) => {
                                data.materials_by_guid.insert(m.guid.clone(), m.id.clone());
                                data.materials.insert(m.id.clone(), m);
                            }
                            Err(e) => tracing::warn!("Materials: skip row: {}", e),
                        }
                    }
                }
                "Creatures" => {
                    for line in &sheet.lines {
                        match parse_creature(line) {
                            Ok(c) => {
                                data.creatures_by_guid.insert(c.guid.clone(), c.id.clone());
                                data.creatures.insert(c.id.clone(), c);
                            }
                            Err(e) => tracing::warn!("Creatures: skip row: {}", e),
                        }
                    }
                }
                "Items" => {
                    for line in &sheet.lines {
                        match parse_item(line) {
                            Ok(i)  => { data.items.insert(i.id.clone(), i); }
                            Err(e) => tracing::warn!("Items: skip row: {}", e),
                        }
                    }
                }
                "Spells" => {
                    for line in &sheet.lines {
                        match parse_spell(line) {
                            Ok(s)  => { data.spells.insert(s.id.clone(), s); }
                            Err(e) => tracing::warn!("Spells: skip row: {}", e),
                        }
                    }
                }
                other => {
                    tracing::debug!("Unknown sheet '{}' stored as raw", other);
                    data.raw.insert(other.to_string(), sheet.lines);
                }
            }
        }

        tracing::info!(
            "GameData: {} materials, {} creatures, {} items, {} spells, {} raw",
            data.materials.len(), data.creatures.len(),
            data.items.len(),     data.spells.len(),
            data.raw.len()
        );

        Ok(data)
    }

    // ── Line Reference helpers ──────────────────────────────────────────

    /// Разрешить LineRef на материал по GUID.
    pub fn resolve_material(&self, guid: &str) -> Option<&MaterialDef> {
        let id = self.materials_by_guid.get(guid)?;
        self.materials.get(id)
    }

    /// Разрешить LineRef на существо по GUID.
    pub fn resolve_creature(&self, guid: &str) -> Option<&CreatureDef> {
        let id = self.creatures_by_guid.get(guid)?;
        self.creatures.get(id)
    }

    /// Сырой лист для кастомной логики.
    pub fn raw_sheet(&self, name: &str) -> &[Value] {
        self.raw.get(name).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

// ──────────────────────────────────── helpers ─────────────────────────────

/// Обязательное строковое поле
fn req_str<'a>(v: &'a Value, field: &str) -> Result<&'a str, String> {
    v[field].as_str().ok_or_else(|| format!("missing string '{}'", field))
}
/// Необязательное строковое поле (возвращает "" если отсутствует)
fn opt_str<'a>(v: &'a Value, field: &str) -> &'a str {
    v[field].as_str().unwrap_or("")
}
fn bool_f(v: &Value, field: &str) -> bool {
    v[field].as_bool().unwrap_or(false)
}
fn int_f(v: &Value, field: &str, default: i64) -> i64 {
    v[field].as_i64().unwrap_or(default)
}
fn f32_f(v: &Value, field: &str, default: f32) -> f32 {
    v[field].as_f64().unwrap_or(default as f64) as f32
}
fn char_f(v: &Value, field: &str) -> char {
    v[field].as_str().and_then(|s| s.chars().next()).unwrap_or('?')
}
/// MultiSelect хранится как массив строк: ["tag1", "tag2"]
fn multiple_f(v: &Value, field: &str) -> Vec<String> {
    v[field].as_array()
        .map(|arr| arr.iter()
            .filter_map(|x| x.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect())
        .unwrap_or_default()
}
/// LineRef хранится как строка-GUID. None если пустая строка.
fn lineref_f(v: &Value, field: &str) -> Option<String> {
    let s = v[field].as_str().unwrap_or("");
    if s.is_empty() { None } else { Some(s.to_string()) }
}

fn parse_material(v: &Value) -> Result<MaterialDef, String> {
    Ok(MaterialDef {
        guid:     req_str(v, "guid")?.to_string(),
        id:       req_str(v, "id")?.to_string(),
        name:     opt_str(v, "name").to_string(),
        solid:    bool_f(v, "solid"),
        opaque:   bool_f(v, "opaque"),
        liquid:   bool_f(v, "liquid"),
        walkable: bool_f(v, "walkable"),
        tile_id:  int_f(v, "tile_id", 0) as u16,
        color:    v["color"].as_str().unwrap_or("#888888").to_string(),
    })
}

fn parse_creature(v: &Value) -> Result<CreatureDef, String> {
    Ok(CreatureDef {
        guid:             req_str(v, "guid")?.to_string(),
        id:               req_str(v, "id")?.to_string(),
        name:             opt_str(v, "name").to_string(),
        glyph:            char_f(v, "glyph"),
        color:            v["color"].as_str().unwrap_or("#ffffff").to_string(),
        max_hp:           int_f(v, "max_hp",   10) as i32,
        max_mana:         int_f(v, "max_mana",  0) as i32,
        speed:            int_f(v, "speed",   100) as i32,
        loot_table_guid:  lineref_f(v, "loot_table"),
    })
}

fn parse_item(v: &Value) -> Result<ItemDef, String> {
    Ok(ItemDef {
        guid:      req_str(v, "guid")?.to_string(),
        id:        req_str(v, "id")?.to_string(),
        name:      opt_str(v, "name").to_string(),
        glyph:     char_f(v, "glyph"),
        weight:    f32_f(v, "weight", 1.0),
        value:     int_f(v, "value", 0) as i32,
        stackable: bool_f(v, "stackable"),
        tags:      multiple_f(v, "tags"),
    })
}

fn parse_spell(v: &Value) -> Result<SpellDef, String> {
    Ok(SpellDef {
        guid:        req_str(v, "guid")?.to_string(),
        id:          req_str(v, "id")?.to_string(),
        name:        opt_str(v, "name").to_string(),
        damage:      int_f(v, "damage",    0) as i32,
        mana_cost:   int_f(v, "mana_cost", 10) as i32,
        radius:      int_f(v, "radius",    1) as i32,
        range:       int_f(v, "range",     5) as i32,
        damage_type: opt_str(v, "damage_type").to_string(),
    })
}