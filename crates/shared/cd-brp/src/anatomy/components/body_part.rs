use bevy::ecs::component::Component;
use bevy::reflect::Reflect;
use enum_map::EnumMap;
use enum_map::enum_map;
use serde::{Deserialize, Serialize};

use crate::anatomy::types::location::calculate_location_hp;
use crate::{
    HitLocationType, Injury,
    anatomy::{Organ, OrganType, TissueLayer, TissueType, Wound},
};

#[derive(Debug, Clone, Component, Serialize, Deserialize, Reflect)]
pub struct BodyPart {
    // Legacy BRP поля (для совместимости с action_points.rs)
    pub location: HitLocationType,
    pub hp: i32,
    pub max_hp: i32,
    pub armor: i32,
    pub injuries: Vec<Injury>,

    // Симулятивные поля
    #[reflect(ignore)]
    pub tissues: EnumMap<TissueType, Option<TissueLayer>>,
    #[reflect(ignore)]
    pub organs: EnumMap<OrganType, Option<Organ>>,
    pub wounds: Vec<Wound>,
    pub is_useless: bool,
    pub is_destroyed: bool,
}

impl Default for BodyPart {
    fn default() -> Self {
        Self::new(10, crate::HitLocationType::Chest, 0)
    }
}

impl BodyPart {
    pub fn new(total_hp: i32, location: HitLocationType, armor: i32) -> Self {
        let max_hp = calculate_location_hp(total_hp, location);

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
