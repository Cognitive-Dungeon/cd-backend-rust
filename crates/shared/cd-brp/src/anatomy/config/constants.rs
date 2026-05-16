use crate::anatomy::TissueType;

// ============================================================================
// BRP Ruleset Constants
// ============================================================================
/// Максимальный урон, который может получить часть тела за один удар (BRP UGE)
pub const BRP_MAX_PART_DAMAGE_MULTIPLIER: i32 = 2;

// ============================================================================
// Vital & Shock Constants (Dwarf Fortress / Realistic inspired)
// ============================================================================
pub const DF_PAIN_UNCONSCIOUS_THRESHOLD: f32 = 150.0;
pub const DF_BLOOD_VOLUME_HUMAN_ML: f32 = 4940.0;
pub const DF_HEAL_RATE_PER_HOUR: f32 = 0.05;

/// Порог потери крови для начала гиповолемического шока (50%)
pub const SHOCK_BLOOD_LOSS_THRESHOLD: f32 = 0.5;

// ============================================================================
// Tissue Damage & Pain Constants
// ============================================================================
/// Базовый множитель генерации боли при повреждении тканей
pub const PAIN_BASE_MULTIPLIER: f32 = 10.0;

/// Множители кровотечения в зависимости от типа урона
pub const BLEED_MOD_CUTTING: f32 = 2.0;
pub const BLEED_MOD_PIERCING: f32 = 1.5;
pub const BLEED_MOD_BLUNT: f32 = 0.3;
pub const BLEED_MOD_DEFAULT: f32 = 1.0;

/// Шансы заражения в зависимости от типа раны
pub const INFECTION_RISK_BURNING: f32 = 0.0; // Огонь прижигает рану
pub const INFECTION_RISK_DEFAULT: f32 = 0.15; // 15% базовый шанс для открытых ран

/// Количество вытекающей крови (мл) для создания визуального события "BloodSpilled"
pub const BLOOD_SPILLED_VISUAL_MULTIPLIER: f32 = 2.0;

// ============================================================================
// Order of penetration
// ============================================================================
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
