use super::define_id;
use cd_core::Glyph;
use serde::Deserialize;

define_id!(CreatureId);

#[derive(Debug, Clone, Deserialize)]
pub struct CreatureDef {
    pub id: CreatureId,
    pub slug: String,
    pub name: String,
    pub desc: String,
    #[serde(deserialize_with = "crate::utils::deserialize_glyph")]
    pub glyph: Glyph,
    pub base_hp: i32,
    pub base_mp: i32,
    pub speed: i32,
}
