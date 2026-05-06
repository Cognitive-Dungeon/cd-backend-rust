use super::{define_id, parse_glyph};
use crate::depot::{FromDepotLine, Line};
use cd_common::Glyph;

define_id!(CreatureId);

/// Определение существа
#[derive(Debug, Clone)]
pub struct CreatureDef {
    pub id: CreatureId,
    pub slug: String,
    pub name: String,
    pub desc: String,
    pub glyph: Glyph,
    pub base_hp: i32,
    pub base_mp: i32,
    pub speed: i32,
}

impl FromDepotLine for CreatureDef {
    fn from_depot_line(line: &Line<'_>) -> Result<Self, String> {
        Ok(Self {
            id: line
                .id()
                .parse::<CreatureId>()
                .map_err(|e| format!("Failed to parse ID for creature '{}': {}", line.id(), e))?,
            slug: line.text("slug").to_string(),
            name: line.text("name").to_string(),
            desc: line.text("desc").to_string(),
            glyph: parse_glyph(line),
            base_hp: line.int_or("base_hp", 100) as i32,
            base_mp: line.int_or("base_mp", 0) as i32,
            speed: line.int_or("speed", 100) as i32,
        })
    }
}
