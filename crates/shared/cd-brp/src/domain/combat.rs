use serde::{Deserialize, Serialize};

use crate::{
    ActionPriority, CombatAction, CombatPhase, CombatRounds, DefId, Meters, StrikeRank, WeaponClass,
};

/// Действие, запланированное на выполнение в конкретный тик инициативы.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduledAction {
    /// Идентификатор актёра, выполняющего действие.
    pub actor_id: cd_core::ObjectGuid,

    /// Само действие (атака, заклинание, использование предмета и т.д.).
    /// Адаптируйте тип под вашу систему действий.
    pub action: CombatAction,

    /// Базовый тик инициативы после применения всех модификаторов (1..=10).
    pub base_strike_rank: StrikeRank,

    /// Приоритет для разрешения коллизий в одном тике.
    pub priority: ActionPriority,

    /// Флаг отложенного действия (игрок выбрал "Hold Action").
    pub is_held: bool,
}

impl ScheduledAction {
    /// Детерминированный порядок: Interrupt > Normal > Delayed,
    /// затем меньший SR (быстрее), затем actor_id для стабильности.
    pub(crate) fn priority_cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .priority // Interrupt первым (обратный порядок)
            .cmp(&self.priority)
            .then_with(|| self.base_strike_rank.cmp(&other.base_strike_rank))
            .then_with(|| self.actor_id.cmp(&other.actor_id))
    }
}

/// Источник модификатора инициативы.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModifierSource {
    // TODO: Избавиться от строк в ModifierSource
    /// Заклинание с указанным названием.
    Spell(String),
    /// Состояние персонажа (ранен, оглушён и т.д.).
    Condition(String),
    /// Предмет экипировки по ID.
    Equipment(DefId),
    /// Черта / способность персонажа.
    Feat(String),
    /// Кастомный источник (для модов).
    Custom(String),
}

/// Контекст текущего состояния боевого раунда.
/// Сервер хранит эту структуру в глобальном ресурсе (Bevy Resource / Arc<Mutex> / etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct RoundState {
    /// Номер текущего раунда (начинается с 1).
    pub current_round: CombatRounds,

    /// Текущая фаза боя.
    pub current_phase: CombatPhase,

    /// Текущий тик инициативы. Имеет смысл только в фазе `MeleeMissileAndMagic`.
    /// Значение от 1 до 10 включительно.
    pub current_strike_rank: StrikeRank,
}

impl Default for RoundState {
    fn default() -> Self {
        Self {
            current_round: CombatRounds::new(1),
            current_phase: CombatPhase::StatementOfIntent,
            current_strike_rank: StrikeRank::new(1),
        }
    }
}

/// Состояние актёра в бою — для контекстной проверки действий.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorCombatState {
    /// Застигнут ли персонаж врасплох.
    pub is_surprised: bool,
    /// Совершил ли персонаж движение в текущей фазе.
    pub has_moved: bool,
    /// Готовое оружие (если есть).
    pub weapon_ready: Option<WeaponClass>,
    /// Количество действий, уже совершённых в этом раунде.
    pub actions_this_round: u8,
    /// Отложенное действие (если игрок выбрал Hold).
    pub held_action: Option<CombatAction>,
    /// Максимальное количество действий в раунде (по правилам / чертам).
    pub max_actions_per_round: u8,
}

impl Default for ActorCombatState {
    fn default() -> Self {
        Self {
            is_surprised: false,
            has_moved: false,
            weapon_ready: None,
            actions_this_round: 0,
            held_action: None,
            max_actions_per_round: 1, // BRP: обычно 1 действие + защита
        }
    }
}

/// Информация о цели действия (опционально).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    pub target_id: cd_core::ObjectGuid,
    pub distance: Meters,
    pub cover: bool,
    pub is_surprised: bool,
}
