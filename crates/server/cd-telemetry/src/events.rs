use serde::Serialize;

/// Структурированные события движка.
/// Это единственный публичный контракт между движком и SDK.
/// Добавлять поля — ок. Менять существующие — breaking change.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum EngineEvent {
    /// Тик полностью выполнен
    TickCompleted {
        tick_id: u64,
        duration_us: u64,
        entity_count: u32,
        command_count: u32,
    },

    /// Сущность создана в мире
    EntitySpawned {
        tick_id: u64,
        guid: String, // String чтобы не тащить сложности сериализации ObjectGuid в SDK
        x: i32,
        y: i32,
    },

    /// Сущность переместилась
    EntityMoved {
        tick_id: u64,
        guid: String,
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
    },

    /// Сущность получила урон
    EntityDamaged {
        tick_id: u64,
        guid: String,
        amount: i32,
        hp_left: i32,
    },

    /// Сущность умерла
    EntityDied { tick_id: u64, guid: String },

    /// Изолированная ошибка (движок продолжает работать)
    ErrorIsolated {
        tick_id: u64,
        context: String,
        error: String,
    },
}
