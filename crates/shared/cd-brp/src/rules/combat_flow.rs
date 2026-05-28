//! Модуль управления боевым раундом и фазами (Combat Phases, стр. 48 BRP: UGE).
//!
//! Предоставляет:
//! - Детерминированную машину состояний с событиями переходов
//! - Очередь действий с приоритетами и модификаторами Strike Rank
//! - Контекстную проверку легальности действий с причинами отказа
//! - Сериализацию для сетевой синхронизации и VTT-интеграции
//! - Оптимизированные хелперы для горячих циклов
//!
//! # Пример использования
//! ```rust
//! use combat::{RoundState, CombatStateMachine, ActionQueue, check_action_legality};
//!
//! let mut state = RoundState::default();
//! let mut queue = ActionQueue::default();
//!
//! // Игрок заявляет атаку
//! queue.schedule(
//!     actor_id,
//!     CombatAction::Attack { weapon: WeaponClass::Sword, target },
//!     base_strike_rank: 3,
//!     &modifiers,
//!     is_held: false,
//! );
//!
//! // Сервер продвигает состояние боя
//! loop {
//!     let (next_state, transition) = state.advance(state);
//!     
//!     if let Some(event) = transition {
//!         // Рассылаем событие клиентам / логируем / триггерим эффекты
//!         broadcast_phase_event(&event);
//!     }
//!     
//!     // Обрабатываем действия текущего тика
//!     if next_state.current_phase == CombatPhase::MeleeMissileAndMagic {
//!         for action in queue.get_actions_for_sr(next_state.current_strike_rank.get()) {
//!             resolve_action(action);
//!         }
//!     }
//!     
//!     state = next_state;
//!     if state.current_phase == CombatPhase::ResolutionAndBookkeeping {
//!         break; // Конец раунда
//!     }
//! }
//! ```

use cd_core::ObjectGuid;
use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use smallvec::{SmallVec, smallvec};
use std::hash::{Hash, Hasher};

use crate::action::sync::NetworkRoundState;
use crate::domain::combat::{ActorCombatState, RoundState, ScheduledAction, TargetInfo};
use crate::error::{LegalityReason, SyncError};
use crate::{
    ActionLegality, ActionPriority, CombatAction, CombatActionCategory, CombatPhase,
    PhaseTransition, StrikeRank, StrikeRankModifier,
};

/// Трейт для детерминированного управления состоянием боя.
/// Реализация гарантирует, что переходы предсказуемы и воспроизводимы.
pub trait CombatStateMachine {
    type State: Copy + PartialEq + std::fmt::Debug;

    /// Возвращает следующее состояние и событие перехода (если переход произошёл).
    ///
    /// # Пример
    /// ```rust
    /// let (next_state, event) = state.advance(state);
    /// if let Some(PhaseTransition::StrikeRankAdvanced { to, .. }) = event {
    ///     println!("Ходим тик {}", to);
    /// }
    /// ```
    fn advance(&self, state: Self::State) -> (Self::State, SmallVec<[PhaseTransition; 2]>);

    /// Проверяет, является ли переход между фазами валидным согласно правилам.
    /// Полезно для санитизации входящих данных в сетевой игре.
    fn is_valid_transition(from: CombatPhase, to: CombatPhase) -> bool;
}

impl CombatStateMachine for RoundState {
    type State = RoundState;

    fn advance(&self, state: RoundState) -> (RoundState, SmallVec<[PhaseTransition; 2]>) {
        match self.current_phase {
            CombatPhase::StatementOfIntent => {
                let mut next = state;
                next.current_phase = CombatPhase::MovementAndNonCombat;
                (
                    next,
                    smallvec![PhaseTransition::PhaseChanged {
                        from: CombatPhase::StatementOfIntent,
                        to: CombatPhase::MovementAndNonCombat,
                    }],
                )
            }

            CombatPhase::MovementAndNonCombat => {
                let mut next = state;
                next.current_phase = CombatPhase::MeleeMissileAndMagic;
                next.current_strike_rank = StrikeRank::new(1);
                (
                    next,
                    smallvec![PhaseTransition::PhaseChanged {
                        from: CombatPhase::MovementAndNonCombat,
                        to: CombatPhase::MeleeMissileAndMagic,
                    }],
                )
            }

            CombatPhase::MeleeMissileAndMagic => {
                let current_sr = self.current_strike_rank.get();
                if current_sr < StrikeRank::MAX {
                    let mut next = state;
                    next.current_strike_rank = StrikeRank::new(current_sr + 1);
                    (
                        next,
                        smallvec![PhaseTransition::StrikeRankAdvanced {
                            from: current_sr,
                            to: current_sr + 1,
                        }],
                    )
                } else {
                    let mut next = state;
                    next.current_phase = CombatPhase::ResolutionAndBookkeeping;
                    (
                        next,
                        smallvec![PhaseTransition::PhaseChanged {
                            from: CombatPhase::MeleeMissileAndMagic,
                            to: CombatPhase::ResolutionAndBookkeeping,
                        }],
                    )
                }
            }

            CombatPhase::ResolutionAndBookkeeping => {
                let ended_round = state.current_round;
                let new_round = state.current_round.saturating_add(1);

                let mut next = state;
                next.current_phase = CombatPhase::StatementOfIntent;
                next.current_round = new_round;
                next.current_strike_rank = StrikeRank::new(1);

                (
                    next,
                    smallvec![
                        PhaseTransition::RoundEnded(ended_round), // ← старый раунд
                        PhaseTransition::RoundStarted(new_round), // ← новый раунд
                    ],
                )
            }
        }
    }

    fn is_valid_transition(from: CombatPhase, to: CombatPhase) -> bool {
        matches!(
            (from, to),
            // Стандартные переходы по фазам
            (CombatPhase::StatementOfIntent, CombatPhase::MovementAndNonCombat) |
            (CombatPhase::MovementAndNonCombat, CombatPhase::MeleeMissileAndMagic) |
            (CombatPhase::MeleeMissileAndMagic, CombatPhase::ResolutionAndBookkeeping) |
            (CombatPhase::ResolutionAndBookkeeping, CombatPhase::StatementOfIntent) |
            // Разрешаем "застревание" на фазе атак для инкремента тиков
            (CombatPhase::MeleeMissileAndMagic, CombatPhase::MeleeMissileAndMagic)
        )
    }
}

/// Менеджер очереди действий на текущий раунд.
/// Хранит действия, сгруппированные по тикам инициативы (1..=10),
/// а также отдельную очередь для реакций.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActionQueue {
    /// Действия, сгруппированные по тикам: индекс 0 = SR 1, индекс 9 = SR 10.
    #[serde(with = "array_serde")]
    pub(crate) actions_by_sr: [Vec<ScheduledAction>; StrikeRank::MAX as usize],

    /// Глобальная очередь реакций — обрабатывается вне зависимости от тика.
    pub(crate) reactions: Vec<ScheduledAction>,
}

impl ActionQueue {
    /// Создаёт пустую очередь действий.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавляет действие в очередь с учётом модификаторов к инициативе.
    ///
    /// # Параметры
    /// * `actor_id` — идентификатор актёра
    /// * `action` — само действие
    /// * `base_sr` — базовый тик инициативы (до модификаторов)
    /// * `modifiers` — список модификаторов (магия, черты, состояние)
    /// * `is_held` — флаг отложенного действия (Hold Action)
    pub fn schedule(
        &mut self,
        actor_id: ObjectGuid,
        action: CombatAction,
        base_sr: StrikeRank,
        modifiers: &[StrikeRankModifier],
        is_held: bool,
    ) {
        let is_reaction = action.is_reaction();
        // Применяем модификаторы к базовому SR
        let final_sr = modifiers.iter().fold(base_sr, |sr, m| m.apply(sr));

        let priority = if is_reaction {
            ActionPriority::Interrupt
        } else if is_held {
            ActionPriority::Delayed
        } else {
            ActionPriority::Normal
        };

        let scheduled = ScheduledAction {
            actor_id,
            action,
            base_strike_rank: final_sr,
            priority,
            is_held,
        };

        // Реакции — в отдельную очередь (всегда доступны)
        if is_reaction {
            self.reactions.push(scheduled);
        } else {
            // Индекс в массиве: SR 1 -> индекс 0, SR 10 -> индекс 9
            let bucket = &mut self.actions_by_sr[(final_sr.get() - 1) as usize];

            // partition_point — бинарный поиск позиции вставки: O(log n)
            // Vec::insert сдвигает хвост: O(n)
            let pos = bucket.partition_point(|existing| existing.priority_cmp(&scheduled).is_lt());
            bucket.insert(pos, scheduled);
        }
    }

    /// Возвращает действия для текущего тика, отсортированные по приоритету.
    ///
    /// Порядок сортировки: Interrupt > Normal > Delayed, затем по базовому SR.
    #[inline]
    #[must_use]
    pub fn get_actions_for_sr(&self, sr: StrikeRank) -> &[ScheduledAction] {
        let sr_val = sr.get();
        if !(1..=StrikeRank::MAX).contains(&sr_val) {
            return &[];
        }
        &self.actions_by_sr[(sr_val - 1) as usize]
    }

    /// Возвращает все доступные реакции (независимо от тика).
    #[must_use]
    pub fn get_pending_reactions(&self) -> Vec<&ScheduledAction> {
        self.reactions.iter().collect()
    }

    /// Очищает очередь реакций после их обработки.
    pub fn clear_reactions(&mut self) {
        self.reactions.clear();
    }

    /// Очищает всю очередь (при старте нового раунда).
    pub fn clear(&mut self) {
        for bucket in &mut self.actions_by_sr {
            bucket.clear();
        }
        self.reactions.clear();
    }

    /// Возвращает общее количество запланированных действий.
    #[must_use]
    pub fn len(&self) -> usize {
        self.actions_by_sr.iter().map(Vec::len).sum::<usize>() + self.reactions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Проверка легальности действий
// ─────────────────────────────────────────────────────────────────────────────

/// Контекст для проверки легальности действия.
#[derive(Debug, Clone)]
pub struct ActionContext<'a> {
    /// Текущая фаза боя.
    pub current_phase: CombatPhase,
    /// Состояние актёра, выполняющего действие.
    pub actor_state: &'a ActorCombatState,
    /// Информация о цели (если действие таргетированное).
    pub target_info: Option<&'a TargetInfo>,
    /// Ранее заявленное намерение (из фазы StatementOfIntent).
    pub declared_intent: Option<&'a CombatAction>,
}

/// Проверяет, легально ли выполнить указанное действие в текущем контексте.
///
/// Возвращает структурированный результат с причиной отказа (если есть).
#[must_use]
pub fn check_action_legality(action: &CombatAction, ctx: &ActionContext) -> ActionLegality {
    use CombatActionCategory::*;
    use CombatPhase::*;

    // ── Глобальные блокировки ─────────────────────────────────────────────

    // Застигнут врасплох: нельзя действовать, кроме реакций
    if ctx.actor_state.is_surprised && !action.is_reaction() {
        return ActionLegality {
            allowed: false,
            reason: Some(LegalityReason::Surprised),
        };
    }

    // Лимит действий в раунде
    if ctx.actor_state.actions_this_round >= ctx.actor_state.max_actions_per_round
        && !matches!(action.category(), Defense)
    {
        return ActionLegality {
            allowed: false,
            reason: Some(LegalityReason::ActionLimitExceeded),
        };
    }

    // ── Проверка по категориям действий ───────────────────────────────────

    match action.category() {
        Movement | NonCombat => {
            if ctx.current_phase != MovementAndNonCombat {
                return ActionLegality {
                    allowed: false,
                    reason: Some(LegalityReason::WrongPhase {
                        expected: MovementAndNonCombat,
                        actual: ctx.current_phase,
                    }),
                };
            }

            // Движение: нельзя двигаться дважды за фазу
            if matches!(action, CombatAction::Move) && ctx.actor_state.has_moved {
                return ActionLegality {
                    allowed: false,
                    reason: Some(LegalityReason::ActionAlreadyDeclared),
                };
            }
        }

        Attack | Magic => {
            if ctx.current_phase != MeleeMissileAndMagic {
                return ActionLegality {
                    allowed: false,
                    reason: Some(LegalityReason::WrongPhase {
                        expected: MeleeMissileAndMagic,
                        actual: ctx.current_phase,
                    }),
                };
            }

            // Атака оружием: оно должно быть готово
            if ctx.actor_state.weapon_ready.is_none() {
                return ActionLegality {
                    allowed: false,
                    reason: Some(LegalityReason::WeaponNotReady),
                };
            }

            // Заклинание: проверка на наличие компонента / фокуса (опционально)
            // if let CombatAction::CastSpell { spell_id } = action { ... }
        }

        Defense => {
            // Парирование: нельзя, если застигнут врасплох
            if ctx.actor_state.is_surprised && matches!(action, CombatAction::Parry) {
                return ActionLegality {
                    allowed: false,
                    reason: Some(LegalityReason::Surprised),
                };
            }

            // Уклонение: обычно требует возможности движения
            // (опциональное правило, можно отключить через конфиг)
            if matches!(action, CombatAction::Dodge) && !ctx.actor_state.has_moved {
                // Можно вернуть предупреждение, но не блокировать:
                // return ActionLegality { allowed: true, reason: Some(LegalityReason::InsufficientMovement) };
            }
        }
    }

    // ── Проверка заявленного намерения (опционально) ──────────────────────

    if let Some(declared) = ctx.declared_intent {
        // Если игрок заявлял одно действие, а пытается сделать другое:
        // (строгие правила) можно запретить, (мягкие) — выдать штраф
        if declared.category() != action.category() {
            // Пример мягкой проверки: разрешить, но отметить для ГМа
            // return ActionLegality { allowed: true, reason: Some(LegalityReason::IntentMismatch) };
        }
    }

    ActionLegality {
        allowed: true,
        reason: None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Оптимизация: битовые маски для быстрых проверок
// ─────────────────────────────────────────────────────────────────────────────

/// Битовая маска разрешённых категорий действий для фазы.
/// Позволяет проверять легальность действия за O(1) без match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseActionMask(u16);

impl PhaseActionMask {
    /// Пустая маска (ничего не разрешено).
    pub const NONE: Self = Self(0);

    /// Возвращает маску разрешённых действий для указанной фазы.
    #[must_use]
    pub const fn for_phase(phase: CombatPhase) -> Self {
        match phase {
            CombatPhase::StatementOfIntent => Self::NONE, // Только декларация

            CombatPhase::MovementAndNonCombat => {
                Self(CombatActionCategory::Movement.bit() | CombatActionCategory::NonCombat.bit())
            }
            CombatPhase::MeleeMissileAndMagic => Self(
                CombatActionCategory::Attack.bit() | CombatActionCategory::Magic.bit(), // Defense обрабатывается отдельно — всегда разрешена как реакция
            ),
            CombatPhase::ResolutionAndBookkeeping => Self::NONE, // Только авто-разрешение
        }
    }

    /// Проверяет, разрешена ли категория действий для этой маски.
    #[inline]
    #[must_use]
    pub const fn allows(&self, category: CombatActionCategory) -> bool {
        (self.0 & category.bit()) != 0
    }
}

/// Быстрая проверка легальности действия (без контекста, только фаза).
///
/// ⚠️ Не учитывает состояние актёра, оружие, сюрприз и т.д.
/// Используйте для предварительной фильтрации в горячих циклах.
#[inline]
#[must_use]
pub fn is_action_legal_fast(
    action_category: CombatActionCategory,
    current_phase: CombatPhase,
) -> bool {
    // Defense всегда разрешена как реакция (проверяется отдельно)
    if matches!(action_category, CombatActionCategory::Defense) {
        return true;
    }
    PhaseActionMask::for_phase(current_phase).allows(action_category)
}

// ─────────────────────────────────────────────────────────────────────────────
// Сетевая синхронизация и детерминизм
// ─────────────────────────────────────────────────────────────────────────────

impl RoundState {
    /// Конвертирует локальное состояние в сетевое представление.
    ///
    /// # Параметры
    /// * `round_seed` — уникальный сид раунда (для генерации детерминированного checksum)
    #[must_use]
    pub fn to_network(&self, round_seed: u64, server_tick: u64) -> NetworkRoundState {
        let mut hasher = SeaHasher::new();
        self.hash(&mut hasher);
        hasher.write_u64(round_seed);
        hasher.write_u64(server_tick);

        NetworkRoundState {
            round_id: self.current_round.get(),
            phase: self.current_phase,
            strike_rank: if matches!(self.current_phase, CombatPhase::MeleeMissileAndMagic) {
                Some(self.current_strike_rank)
            } else {
                None
            },
            checksum: hasher.finish(),
            server_tick,
        }
    }

    /// Проверяет консистентность с серверным состоянием.
    ///
    /// # Возвращает
    /// * `Ok(())` — состояния синхронизированы
    /// * `Err(SyncError)` — обнаружен рассинхрон
    pub fn verify_consistency(&self, network: &NetworkRoundState) -> Result<(), SyncError> {
        if self.current_round.get() != network.round_id {
            return Err(SyncError::RoundMismatch);
        }
        if self.current_phase != network.phase {
            return Err(SyncError::PhaseMismatch);
        }
        if matches!(self.current_phase, CombatPhase::MeleeMissileAndMagic)
            && network.strike_rank != Some(self.current_strike_rank)
        {
            return Err(SyncError::StrikeRankMismatch);
        }
        // Checksum проверяется на стороне сервера
        Ok(())
    }
}

// Хелпер для сериализации фиксированных массивов векторов
pub(crate) mod array_serde {
    use serde::de::{SeqAccess, Visitor};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::fmt;
    use std::marker::PhantomData;

    pub fn serialize<S, T>(data: &[Vec<T>; 10], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        use serde::ser::SerializeTuple;
        let mut tup = serializer.serialize_tuple(10)?;
        for item in data {
            tup.serialize_element(item)?;
        }
        tup.end()
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<[Vec<T>; 10], D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        struct ArrayVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for ArrayVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = [Vec<T>; 10];

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an array of 10 vectors")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut arr = std::array::from_fn(|_| Vec::new());
                for item in arr.iter_mut() {
                    *item = seq.next_element()?.unwrap_or_default();
                }
                Ok(arr)
            }
        }

        deserializer.deserialize_tuple(10, ArrayVisitor(PhantomData))
    }
}
