use super::define_id;
use cd_common::Glyph;
use serde::Deserialize;

define_id!(FurnitureId);

#[derive(Debug, Clone, Deserialize)]
pub struct FurnitureDef {
    pub id: FurnitureId,
    pub slug: String,
    pub name: String,
    #[serde(deserialize_with = "crate::utils::deserialize_glyph")]
    pub glyph: Glyph,
    pub is_solid: bool,
    pub is_opaque: bool,
}
