use std::str::FromStr;

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

/// Числовой ID спелла — используется в рантайме везде где нужна скорость.
/// Назначается при загрузке Depot, не персистентен между перезапусками.
pub type SpellId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellTarget {
    /// Применяется к соседним сущностям (melee)
    Self_,
    /// Требует явного GUID цели
    Entity,
    /// Требует позиции на карте (AoE, projectile)
    Object,
}

#[derive(Debug)]
pub struct ParseSpellTargetError;

impl std::str::FromStr for SpellTarget {
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

#[derive(Debug)]
pub struct ParseDamageTypeError;

impl std::str::FromStr for DamageType {
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
/// Строковый slug нужен только для логов и отладки — в симуляции ходит SpellId.
#[derive(Debug, Clone)]
pub struct SpellDef {
    pub id: SpellId,  // числовой — назначается при загрузке
    pub slug: String, // "melee_attack" — только для логов
    pub name: String,
    pub target: SpellTarget,
    pub range: i32,
    pub effect: SpellEffect,
}

impl FromDepotLine for SpellDef {
    fn from_depot_line(line: &Line<'_>) -> Result<Self, String> {
        Ok(Self {
            id: line.id().parse::<SpellId>().unwrap_or(0),
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
