use super::define_id;
use serde::Deserialize;

define_id!(SpellId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpellTarget {
    Self_,
    Entity,
    Object,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageType {
    Physical,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpellEffect {
    Damage {
        amount: i32,
        damage_type: DamageType,
    },
    Heal {
        amount: i32,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpellDef {
    pub id: SpellId,
    pub slug: String,
    pub name: String,
    pub target: SpellTarget,
    pub range: i32,
    pub effect: SpellEffect,
}
