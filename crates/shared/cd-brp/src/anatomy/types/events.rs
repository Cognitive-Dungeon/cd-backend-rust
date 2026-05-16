use serde::{Deserialize, Serialize};

use crate::anatomy::{HitLocationType, TissueType, WoundSeverity};

/// События, которые генерирует ядро анатомии при обработке урона или тике симуляции.
/// Движок (cd-engine) перехватывает эти события и транслирует их в визуал, звук,
/// дроп предметов, смерть или промпты для LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeathReason {
    HpDepleted,
    BrainDestroyed,
    HeartDestroyed,
    Exsanguination, // Истекание кровью
}
