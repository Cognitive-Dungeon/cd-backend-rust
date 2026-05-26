use serde::{Deserialize, Serialize};

use crate::{DefId, DiseaseSeverity, DiseaseType, time::BrpDuration};

/// Статус/Чертеж болезни или яда (Стр. 96, 111)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToxinBlueprint {
    pub id: DefId,
    pub disease_type: DiseaseType,
    pub severity: DiseaseSeverity,

    // Как быстро начинает действовать (Например: BrpDuration::Hours(1))
    pub speed_of_effect: BrpDuration,

    // Как долго длится (Например: BrpDuration::Indefinite)
    pub duration: BrpDuration,
}
