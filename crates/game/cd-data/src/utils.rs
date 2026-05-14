use cd_common::Glyph;
use serde::Deserialize;

/// Позволяет десериализовать кортеж ('A', "#FFFFFF") напрямую в Glyph
pub fn deserialize_glyph<'de, D>(deserializer: D) -> Result<Glyph, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let (ch, color): (char, String) = Deserialize::deserialize(deserializer)?;
    Glyph::from_json(&ch.to_string(), &color).map_err(serde::de::Error::custom)
}
