use bevy::ecs::message::Message;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::anatomy::{
    DamageResult, HitLocationType, PenetrationProfile, TissueType, WoundSeverity,
};

/// События, которые генерирует ядро анатомии при обработке урона или тике симуляции.
/// Движок (cd-engine) перехватывает эти события и транслирует их в визуал, звук,
/// дроп предметов, смерть или промпты для LLM.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub enum AnatomyEvent {
    /// Ткань была пробита или разрезана (для LLM: "прорубил кожу и мышцы")
    TissueDamaged {
        location: HitLocationType,
        tissue: TissueType,
        damage_ratio: f32, // От 0.0 до 1.0
    },
    /// Кость сломана (для движка: бросить оружие / упасть, для LLM: "хруст костей")
    BoneFractured { location: HitLocationType },
    /// Конечность отрублена (для движка: дроп всего экипа, для LLM: "рука отлетает в сторону")
    LimbSevered { location: HitLocationType },
    /// Пробита артерия или вена (для движка: обильное кровотечение)
    VesselRuptured {
        location: HitLocationType,
        bleed_rate: f32,
    },
    /// Кровь пролилась на землю (для движка: спавн декалей крови на карте)
    BloodSpilled {
        location: HitLocationType,
        amount_ml: f32,
    },
    /// Новая рана определенной тяжести (для UI: показать значок травмы)
    WoundInflicted {
        location: HitLocationType,
        severity: WoundSeverity,
    },
    /// Наступление болевого шока
    ShockInduced { shock_level: f32 },
    /// Потеря сознания
    ConsciousnessLost,
    /// Смерть
    Died { reason: DeathReason },
}

impl AnatomyEvent {
    /// Генерирует текстовые теги для контекста LLM.
    /// Вместо цифр LLM получает понятные описания.
    pub fn narrative_tags(&self) -> Vec<&'static str> {
        match self {
            AnatomyEvent::BoneFractured { .. } => {
                vec!["хруст_костей", "перелом", "структурный_урон"]
            }
            AnatomyEvent::VesselRuptured { bleed_rate, .. } => {
                if *bleed_rate > 3.0 {
                    vec![
                        "фонтан_крови",
                        "артериальное_кровотечение",
                        "опасность_смерти",
                    ]
                } else {
                    vec!["обильное_кровотечение", "глубокий_порез"]
                }
            }
            AnatomyEvent::LimbSevered { .. } => {
                vec!["расчлененка", "ампутация", "шок", "обилие_крови"]
            }
            AnatomyEvent::ShockInduced { shock_level } => {
                if *shock_level > 0.8 {
                    vec!["предобморочное_состояние", "бледность", "холодный_пот"]
                } else {
                    vec!["слабость", "головокружение"]
                }
            }
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeathReason {
    HpDepleted,
    BrainDestroyed,
    HeartDestroyed,
    Exsanguination, // Истекание кровью
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageInput {
    pub location: HitLocationType,
    pub raw_damage: i32,
    pub profile: PenetrationProfile,
    pub timestamp_secs: f64,
}

#[derive(Debug, Clone)]
pub struct SimulationOutput {
    pub damage_result: DamageResult,
    pub events: SmallVec<[AnatomyEvent; 8]>,
}
