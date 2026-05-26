use crate::{
    time::BrpDuration,
    types::{DamageType, DefId, DiceExpression, PowerCost, PowerDefense, PowerRange, PowerType},
};
use serde::{Deserialize, Serialize};

/// Механический эффект способности (что она делает)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "effect_type")]
pub enum PowerMechanic {
    /// Наносит урон в области или по цели
    Damage {
        base_dice: DiceExpression,
        damage_type: DamageType,
        radius_meters: u16, // 0 для одиночной цели
    },
    /// Лечит HP
    Healing { dice: DiceExpression },
    /// Накладывает бафф/дебафф или меняет правила
    StatusEffect {
        // Здесь можно хранить ID статуса для ECS
        effect_id: DefId,
    },
    // В будущем сюда добавятся Summoning, Teleport и т.д.
}

/// Статический чертеж Способности (Заклинания, Мутации, Псионики).
/// Загружается в память сервера один раз при старте.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerBlueprint {
    pub id: DefId,
    pub power_type: PowerType,
    pub cost: PowerCost,
    pub range: PowerRange,
    pub duration: BrpDuration,
    pub defense: PowerDefense,

    pub mechanic: PowerMechanic,

    // --- Теги/Флаги для особых правил (Опционально) ---
    pub ignores_armour: bool,
    pub ignores_countermagic: bool,
}
