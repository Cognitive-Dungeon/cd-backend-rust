use crate::depot::{FromDepotLine, Line};
use cd_common::Glyph;
use cd_map::{MaterialID, TileFlags};

/// Определение материала (тайла карты)
#[derive(Debug, Clone)]
pub struct MaterialDef {
    pub mat_id: MaterialID,
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

        let hex_color = line.color("color");
        let glyph =
            Glyph::from_json(line.text("glyph"), hex_color).unwrap_or(Glyph::new(0xFF00FF, b'?'));

        Ok(Self {
            // В Materials id это "0", "1", а slug это "void", "floor_stone".
            // Поэтому берем text("slug"). В будущем перенести слаги прямо в id.
            mat_id: line.id().parse::<MaterialID>().unwrap_or(0),
            slug: line.text("slug").to_string(),
            name: line.text("name").to_string(),
            desc: line.text("desc").to_string(),
            glyph,
            flags,
        })
    }
}

/// Определение существа
#[derive(Debug, Clone)]
pub struct CreatureDef {
    pub id: String, // В листе Creatures id = "human", "skeleton"
    pub name: String,
    pub desc: String,
    pub glyph: Glyph,
    pub base_hp: i32,
    pub base_mp: i32,
    pub speed: i32,
}

impl FromDepotLine for CreatureDef {
    fn from_depot_line(line: &Line<'_>) -> Result<Self, String> {
        let hex_color = line.color("color");
        let glyph =
            Glyph::from_json(line.text("glyph"), hex_color).unwrap_or(Glyph::new(0xFF00FF, b'?'));

        Ok(Self {
            id: line.id().to_string(),
            name: line.text("name").to_string(),
            desc: line.text("desc").to_string(),
            glyph,
            base_hp: line.int_or("base_hp", 100) as i32,
            base_mp: line.int_or("base_mp", 0) as i32,
            speed: line.int_or("speed", 100) as i32,
        })
    }
}

/// Определение мебели
#[derive(Debug, Clone)]
pub struct FurnitureDef {
    pub id: String,
    pub name: String,
    pub glyph: Glyph,
    pub is_solid: bool,
    pub is_opaque: bool,
}

impl FromDepotLine for FurnitureDef {
    fn from_depot_line(line: &Line<'_>) -> Result<Self, String> {
        let hex_color = line.color("color");
        let glyph =
            Glyph::from_json(line.text("glyph"), hex_color).unwrap_or(Glyph::new(0xFF00FF, b'?'));

        Ok(Self {
            id: line.id().to_string(),
            name: line.text("name").to_string(),
            glyph,
            is_solid: line.bool("is_solid"),
            is_opaque: line.bool("is_opaque"),
        })
    }
}
