use bevy::ecs::component::Component;
use enum_map::EnumMap;
use enum_map::enum_map;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    HitLocationType, Injury,
    anatomy::{HitLocationRoll, Organ, OrganType, TissueLayer, TissueType, Wound},
};

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct BodyPart {
    // Legacy BRP поля (для совместимости с action_points.rs)
    pub location: HitLocationType,
    pub hp: i32,
    pub max_hp: i32,
    pub armor: i32,
    pub injuries: Vec<Injury>,

    // Симулятивные поля
    pub tissues: EnumMap<TissueType, Option<TissueLayer>>,
    pub organs: EnumMap<OrganType, Option<Organ>>,
    pub wounds: Vec<Wound>,
    pub is_useless: bool,
    pub is_destroyed: bool,
}

impl BodyPart {
    pub fn new(total_hp: i32, location: HitLocationType, armor: i32) -> Self {
        let max_hp = (total_hp as f32 * location.hp_fraction()).ceil() as i32;

        let mut tissues = enum_map! { _ => None };

        tissues[TissueType::Skin] = Some(TissueLayer::default_skin());
        tissues[TissueType::Muscle] = Some(TissueLayer::default_muscle(max_hp));

        // Кости для конечностей/головы
        if !matches!(location, HitLocationType::Abdomen) {
            tissues[TissueType::Bone] = Some(TissueLayer {
                tissue_type: TissueType::Bone,
                thickness: 8.0,
                integrity: 1.0,
                max_integrity: 1.0,
                pain_receptors: 5.0,
                bleeding_rate: 0.0,
            });
        }

        Self {
            location,
            hp: max_hp,
            max_hp,
            armor,
            injuries: Vec::new(),
            tissues,
            organs: enum_map! { _ => None },
            wounds: Vec::new(),
            is_useless: false,
            is_destroyed: false,
        }
    }

    pub fn is_useless(&self) -> bool {
        self.is_useless || self.hp <= 0 || self.injuries.contains(&Injury::Severed)
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed || self.hp <= -self.max_hp || self.injuries.contains(&Injury::Severed)
    }
}
