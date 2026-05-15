use cd_core::Glyph;
use cd_map::{MaterialID, TileFlags};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MaterialDef {
    pub id: MaterialID,
    pub slug: String,
    pub name: String,
    pub desc: String,
    #[serde(deserialize_with = "crate::utils::deserialize_glyph")]
    pub glyph: Glyph,
    pub is_solid: bool,
    pub is_opaque: bool,
}

impl MaterialDef {
    pub fn flags(&self) -> TileFlags {
        let mut f = TileFlags::empty();
        if self.is_solid {
            f |= TileFlags::SOLID;
        }
        if self.is_opaque {
            f |= TileFlags::OPAQUE;
        }
        f
    }
}
