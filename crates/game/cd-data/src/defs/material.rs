use super::parse_glyph;
use crate::depot::{FromDepotLine, Line};
use cd_common::Glyph;
use cd_map::{MaterialID, TileFlags};

/// Определение материала (тайла карты)
#[derive(Debug, Clone)]
pub struct MaterialDef {
    pub id: MaterialID, // u16 (из cd-map)
    pub slug: String,
    pub name: String,
    pub desc: String,
    pub glyph: Glyph,
    pub flags: TileFlags,
}

impl FromDepotLine for MaterialDef {
    fn from_depot_line(line: &Line<'_>) -> Result<Self, String> {
        let mut flags = TileFlags::empty();
        if line.bool("is_solid") {
            flags |= TileFlags::SOLID;
        }
        if line.bool("is_opaque") {
            flags |= TileFlags::OPAQUE;
        }

        Ok(Self {
            id: line.id().parse::<MaterialID>().unwrap_or(0),
            slug: line.text("slug").to_string(),
            name: line.text("name").to_string(),
            desc: line.text("desc").to_string(),
            glyph: parse_glyph(line),
            flags,
        })
    }
}
