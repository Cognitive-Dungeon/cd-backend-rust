use crate::types::HitPoints;

/// Тип попадания по самому защитнику (его телу).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetHitType {
    /// Защитник полностью избежал урона по телу (успешный Dodge или Parry).
    Evaded,
    /// Обычное попадание (броня работает штатно).
    Normal,
    /// Особое попадание (срабатывает эффект оружия: Impale, Bleed и т.д., броня работает).
    Special,
    /// Критическое попадание (макс. урон, обычно игнорирует броню).
    Critical,
}

/// Итоговый результат столкновения (Zero Allocations - 100% на стеке).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExchangeOutcome {
    /// Эффект, примененный к самому персонажу-защитнику.
    pub target_hit: TargetHitType,
    /// Урон, который получило оружие или щит при парировании.
    /// Равен 0, если персонаж уворачивался (Dodge) или парирование не повредило предмет.
    pub parry_item_damage: HitPoints,
}

impl ExchangeOutcome {
    /// Удобный конструктор для случаев, когда урон получает только персонаж (или уворачивается)
    pub const fn new(hit: TargetHitType) -> Self {
        Self {
            target_hit: hit,
            parry_item_damage: HitPoints::ZERO, // Используем константу ZERO из вашей реализации HitPoints
        }
    }

    /// Удобный конструктор для случаев успешного парирования, когда ломается щит/оружие
    pub const fn with_item_damage(hit: TargetHitType, damage: i16) -> Self {
        Self {
            target_hit: hit,
            parry_item_damage: HitPoints::new(damage),
        }
    }
}
