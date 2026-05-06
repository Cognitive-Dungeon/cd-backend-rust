use super::{define_id, parse_glyph};
use crate::depot::{FromDepotLine, Line};
use cd_common::Glyph;

define_id!(FurnitureId);

/// Определение мебели
#[derive(Debug, Clone)]
pub struct FurnitureDef {
    pub id: FurnitureId,
    pub slug: String,
    pub name: String,
    pub glyph: Glyph,
    pub is_solid: bool,
    pub is_opaque: bool,
}

impl FromDepotLine for FurnitureDef {
    fn from_depot_line(line: &Line<'_>) -> Result<Self, String> {
        Ok(Self {
            id: line
                .id()
                .parse::<FurnitureId>()
                .map_err(|e| format!("Failed to parse ID for furniture '{}': {}", line.id(), e))?,
            slug: line.text("slug").to_string(),
            name: line.text("name").to_string(),
            glyph: parse_glyph(line),
            is_solid: line.bool("is_solid"),
            is_opaque: line.bool("is_opaque"),
        })
    }
}
