use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    #[error("D100 roll must be between {min} and {max}, got {value}")]
    InvalidD100Roll { value: u16, min: u16, max: u16 },

    #[error("Growth roll must be between {min} and {max}, got {value}")]
    InvalidGrowthRoll { value: u8, min: u8, max: u8 },

    #[error("Value cannot be negative")]
    NegativeValue,
}

/// Причины, по которым действие может быть запрещено.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Error)]
pub enum LegalityReason {
    /// Действие разрешено только в другой фазе.
    #[error("Действие недоступно в фазе {actual:?}, ожидается {expected:?}")]
    WrongPhase {
        /// Ожидаемая фаза.
        expected: super::CombatPhase,
        /// Текущая фаза.
        actual: super::CombatPhase,
    },
    /// Персонаж застигнут врасплох и не может действовать.
    #[error("Персонаж застигнут врасплох")]
    Surprised,
    /// Оружие не готово к использованию (не вынуто, сломано и т.д.).
    #[error("Оружие не готово к использованию")]
    WeaponNotReady,
    /// Недостаточно движения для выполнения действия (например, для уклонения).
    #[error("Недостаточно очков движения для выполнения действия")]
    InsufficientMovement,
    /// Действие уже было заявлено в этом раунде.
    #[error("Действие уже было заявлено в этом раунде")]
    ActionAlreadyDeclared,
    /// Превышен лимит действий в раунде.
    #[error("Превышен лимит действий в раунде")]
    ActionLimitExceeded,
    /// Кастомная причина (для модов и расширений).
    #[error("{0}")]
    Custom(String),
}

/// Ошибки синхронизации состояния боя.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Error)]
#[serde(rename_all = "snake_case")]
pub enum SyncError {
    /// Номер раунда не совпадает.
    #[error("Номер раунда не совпадает")]
    RoundMismatch,
    /// Фаза боя не совпадает.
    #[error("Фаза боя не совпадает")]
    PhaseMismatch,
    /// Тик инициативы не совпадает.
    #[error("Тик инициативы не совпадает")]
    StrikeRankMismatch,
    /// Контрольная сумма состояния не совпадает (рассинхрон).
    #[error("Контрольная сумма состояния не совпадает")]
    ChecksumFailed,
    /// Слишком большой дрейф таймстампов (лаг / чит).
    #[error("Слишком большой дрейф времени")]
    TimestampDrift,
}
