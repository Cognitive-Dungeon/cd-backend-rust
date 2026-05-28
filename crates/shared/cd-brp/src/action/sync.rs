// crates/shared/cd-brp/src/action/sync.rs

use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use crate::domain::combat::{ActorCombatState, RoundState};
use crate::rules::combat_flow::ActionQueue;
use crate::types::{CombatPhase, StrikeRank};
use cd_core::ObjectGuid;

/// Сериализуемое представление состояния для передачи по сети.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRoundState {
    /// Монотонный идентификатор раунда (для детекта рассинхрона).
    pub round_id: u32,
    /// Текущая фаза.
    pub phase: CombatPhase,
    /// Текущий тик инициативы (если в фазе атак).
    pub strike_rank: Option<StrikeRank>,
    /// Контрольная сумма состояния (для детекта рассинхрона).
    pub checksum: u64,
    /// Серверный таймстамп (для компенсации лагов).
    pub server_tick: u64,
}

impl NetworkRoundState {
    pub fn from_state(state: &RoundState, round_seed: u64, server_tick: u64) -> Self {
        let mut hasher = SeaHasher::new();
        state.hash(&mut hasher);
        hasher.write_u64(round_seed);
        hasher.write_u64(server_tick);

        Self {
            round_id: state.current_round.get(),
            phase: state.current_phase,
            strike_rank: if matches!(state.current_phase, CombatPhase::MeleeMissileAndMagic) {
                Some(state.current_strike_rank)
            } else {
                None
            },
            checksum: hasher.finish(),
            server_tick,
        }
    }
}

/// Снимок состояния боя для механизмов rollback / анти-чита.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatSnapshot {
    /// Серверный тик, на котором сделан снимок.
    pub timestamp: u64,
    /// Состояние раунда.
    pub round_state: RoundState,
    /// Хэш очереди действий (для детекта несанкционированных изменений).
    pub action_queue_hash: u64,
    /// Состояния всех актёров (для детерминированного воспроизведения).
    pub actor_states: BTreeMap<ObjectGuid, ActorCombatState>,
}

impl CombatSnapshot {
    /// Создаёт снимок текущего состояния.
    #[must_use]
    pub fn new(
        timestamp: u64,
        round_state: RoundState,
        action_queue: &ActionQueue,
        actor_states: BTreeMap<ObjectGuid, ActorCombatState>,
    ) -> Self {
        // Вычисляем хэш очереди для детекта изменений
        let mut hasher = SeaHasher::new();
        for bucket in &action_queue.actions_by_sr {
            for action in bucket {
                action.actor_id.hash(&mut hasher);
                action.base_strike_rank.hash(&mut hasher);
            }
        }
        for reaction in &action_queue.reactions {
            reaction.actor_id.hash(&mut hasher);
        }

        Self {
            timestamp,
            round_state,
            action_queue_hash: hasher.finish(),
            actor_states,
        }
    }
}

/// Представление состояния боя для VTT-фронтенда (JSON API).
#[derive(Debug, Clone, Serialize)]
pub struct VTTCombatState {
    /// Номер раунда.
    pub round: u32,
    /// Код фазы для фронтенда.
    pub phase: &'static str,
    /// Текущий тик инициативы (если применимо).
    pub strike_rank: Option<StrikeRank>,
    /// Список активных актёров в текущем тике.
    pub active_actors: Vec<ObjectGuid>,
    /// Количество ожидающих реакций.
    pub pending_reactions: usize,
    /// Подсказка для анимации (опционально).
    pub animation_hint: Option<&'static str>,
}

impl From<&RoundState> for VTTCombatState {
    fn from(state: &RoundState) -> Self {
        Self {
            round: state.current_round.get(),
            phase: match state.current_phase {
                CombatPhase::StatementOfIntent => "intent",
                CombatPhase::MovementAndNonCombat => "movement",
                CombatPhase::MeleeMissileAndMagic => "actions",
                CombatPhase::ResolutionAndBookkeeping => "cleanup",
            },
            strike_rank: if matches!(state.current_phase, CombatPhase::MeleeMissileAndMagic) {
                Some(state.current_strike_rank)
            } else {
                None
            },
            active_actors: Vec::new(), // Заполняется из других ресурсов
            pending_reactions: 0,      // Заполняется из ActionQueue
            animation_hint: None,      // Можно добавить логику выбора
        }
    }
}
