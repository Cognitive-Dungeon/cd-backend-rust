use crate::anatomy::TissueType;

// BRP/DF константы для баланса
pub const BRP_MAX_PART_DAMAGE_MULTIPLIER: i32 = 2;
pub const DF_PAIN_UNCONSCIOUS_THRESHOLD: f32 = 150.0;
pub const DF_BLOOD_VOLUME_HUMAN_ML: f32 = 1000.0;
pub const DF_HEAL_RATE_PER_HOUR: f32 = 0.05;

/// Порядок тканей для расчёта проникновения урона (от внешних к внутренним)
pub const TISSUE_PENETRATION_ORDER: [TissueType; 10] = [
    TissueType::Skin,
    TissueType::Fat,
    TissueType::Muscle,
    TissueType::Tendon,
    TissueType::Ligament,
    TissueType::Bone,
    TissueType::Nerve,
    TissueType::Artery,
    TissueType::Vein,
    TissueType::OrganTissue,
];
