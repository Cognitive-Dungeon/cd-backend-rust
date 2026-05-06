use super::define_id;
use crate::depot::{FromDepotLine, Line};
use std::str::FromStr;

define_id!(SpellId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellTarget {
    Self_,
    Entity,
    Object,
}

#[derive(Debug, thiserror::Error)]
#[error("invalid spell target")]
pub struct ParseSpellTargetError;

impl FromStr for SpellTarget {
    type Err = ParseSpellTargetError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "self" => Ok(Self::Self_),
            "entity" => Ok(Self::Entity),
            "object" => Ok(Self::Object),
            _ => Err(ParseSpellTargetError),
        }
    }
}

impl SpellTarget {
    pub fn parse_or_default(s: &str) -> Self {
        s.parse().unwrap_or(Self::Self_)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Physical,
}

#[derive(Debug, thiserror::Error)]
#[error("invalid damage type")]
pub struct ParseDamageTypeError;

impl FromStr for DamageType {
    type Err = ParseDamageTypeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "physical" => Ok(Self::Physical),
            _ => Err(ParseDamageTypeError),
        }
    }
}

impl DamageType {
    pub fn parse_or_default(s: &str) -> Self {
        s.parse().unwrap_or(Self::Physical)
    }
}

#[derive(Debug, Clone)]
pub enum SpellEffect {
    Damage {
        amount: i32,
        damage_type: DamageType,
    },
    Heal {
        amount: i32,
    },
}

impl SpellEffect {
    pub fn from_depot(effect_type: &str, amount: i32, damage_type: &str) -> Self {
        match effect_type {
            "heal" => Self::Heal { amount },
            _ => Self::Damage {
                amount,
                damage_type: DamageType::parse_or_default(damage_type),
            },
        }
    }
}

/// Определение спелла из Depot.
#[derive(Debug, Clone)]
pub struct SpellDef {
    pub id: SpellId,
    pub slug: String,
    pub name: String,
    pub target: SpellTarget,
    pub range: i32,
    pub effect: SpellEffect,
}

impl FromDepotLine for SpellDef {
    fn from_depot_line(line: &Line<'_>) -> Result<Self, String> {
        Ok(Self {
            id: line
                .id()
                .parse::<SpellId>()
                .map_err(|e| format!("Failed to parse ID for spell '{}': {}", line.id(), e))?,
            slug: line.text("slug").to_string(),
            name: line.text("name").to_string(),
            target: SpellTarget::parse_or_default(line.text("target")),
            range: line.int("range") as i32,
            effect: SpellEffect::from_depot(
                line.text("effect_type"),
                line.int("effect_amount") as i32,
                line.text("damage_type"),
            ),
        })
    }
}
