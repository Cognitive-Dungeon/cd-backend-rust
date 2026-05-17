# Оглавление файла
- [[#Последние штрихи (Finishing Touches)]]
- [[#Опциональное правило: Генерация за очки (Point-Based Character Creation)]]
- [[#Опциональное правило: Высокие стартовые характеристики (Higher Starting Characteristics)]]
- [[#Опциональное правило: Характеристика Образования (Education)]]
- [[#Опциональное правило: Культурные Модификаторы (Cultural Modifiers or Non-Human Characters)]]

---

## Последние штрихи (Finishing Touches)
Финальный этап создания персонажа, посвященный нарративным, косметическим и биографическим деталям, которые не имеют прямого числового выражения в механике игры.

### Механика / Концепция
На этом этапе игрок заполняет оставшиеся пустые поля на листе персонажа. Если имя не было выбрано в [[Step One - Name and Characteristics|Шаге 1]], оно выбирается сейчас. 
Остальные аспекты могут быть придуманы до начала игры или раскрываться в процессе:
- **Внешность:** Цвет волос, кожи и глаз. Стиль одежды.
- **Особенности поведения:** Интересные манеры, девизы, часто используемые фразы, репутация.
- **Предыстория (Background):** Происхождение (откуда персонаж родом), образование (где учился), отношения с семьей, членство в значимых организациях, важные события из прошлого, религиозные или политические убеждения.

Эти элементы используются `Gamemaster` (Гейммастером) для интеграции персонажа в сеттинг.

### Архитектура Rust
Поскольку эти данные не участвуют в математических расчетах системы, они хранятся как простые опциональные текстовые строки (метаданные).

```rust
pub struct CharacterBackground {
    pub hair_color: Option<String>,
    pub skin_color: Option<String>,
    pub eye_color: Option<String>,
    pub clothing_style: Option<String>,
    pub mannerisms: Option<String>,
    pub motto: Option<String>,
    pub reputation: Option<String>,
    pub origin: Option<String>,
    pub schooling: Option<String>,
    pub family_ties: Option<String>,
    pub organizations: Vec<String>,
    pub beliefs: Option<String>,
    pub backstory: Option<String>,
}
```

---

## Опциональное правило: Генерация за очки (Point-Based Character Creation)
Альтернативная система создания характеристик, заменяющая случайные броски кубиков из [[Step One - Name and Characteristics|Шага 1]] на точное распределение баллов.

### Механика / Концепция
Вместо бросков кубиков игрок "покупает" значения характеристик из фиксированного бюджета.
1. **Базовое значение**: Все характеристики (`STR`, `CON`, `SIZ`, `INT`, `POW`, `DEX`, `CHA`) начинают со значения `10`.
2. **Бюджет**: Игрок получает `24` очка для распределения (эквивалентно уровню силы `Normal`).
3. **Лимит**: Ни одна стартовая характеристика не может быть выше `21`.
4. **Стоимость покупки**:
   - 1 очко `STR`, `CON`, `SIZ` или `CHA` стоит **1 очко** из бюджета.
   - 1 очко `DEX`, `INT` или `POW` стоит **3 очка** из бюджета.
5. **Снижение характеристик**: Игрок может опустить значение ниже стартовых `10` (вплоть до минимума `3`), чтобы получить дополнительные очки в бюджет. За каждую единицу `STR`, `CON`, `SIZ`, `CHA` ниже 10 возвращается **1 очко**, за `DEX`, `INT`, `POW` возвращается **3 очка**.
6. **Выход за лимиты**: Повышение выше 21 или понижение ниже 3 возможно только с разрешения Гейммастера.
7. **Интеграция с силами**: В играх со способностями ([[Step Two - Powers|Powers]]), неиспользованные очки характеристик можно перевести в бюджет способностей (powers budget). Однако это может быть невыгодно по курсу обмена. В этом случае применяется альтернативный Шаг 2 из [[Chapter 4: Powers]].

### Архитектура Rust
```rust
pub struct PointBuySystem;

impl PointBuySystem {
    pub const BASE_STAT: u8 = 10;
    pub const MIN_STAT: u8 = 3;
    pub const MAX_NORMAL_STAT: u8 = 21;

    // Вычисляет стоимость отклонения от базовой 10
    pub fn get_stat_cost(stat_name: &str, current_value: u8) -> i32 {
        let diff = current_value as i32 - Self::BASE_STAT as i32;
        match stat_name {
            "STR" | "CON" | "SIZ" | "CHA" => diff * 1,
            "DEX" | "INT" | "POW" => diff * 3,
            _ => 0,
        }
    }

    pub fn calculate_total_cost(stats: &Characteristics) -> i32 {
        Self::get_stat_cost("STR", stats.str) +
        Self::get_stat_cost("CON", stats.con) +
        Self::get_stat_cost("SIZ", stats.siz) +
        Self::get_stat_cost("CHA", stats.cha) +
        Self::get_stat_cost("DEX", stats.dex) +
        Self::get_stat_cost("INT", stats.int) +
        Self::get_stat_cost("POW", stats.pow)
    }
}
```

---

## Опциональное правило: Высокие стартовые характеристики (Higher Starting Characteristics)
Модификация системы Point-Based для игр с более высоким уровнем силы (где кубиками кидалось бы `2D6+6` вместо `3D6`).

### Механика / Концепция
Увеличивает стартовый бюджет в зависимости от уровня силы кампании:
- **Heroic**: `36` очков.
- **Epic**: `48` очков.
- **Superhuman**: `60` очков.
Для уровней `Epic` и `Superhuman` стандартный лимит характеристик в `21` единицу должен игнорироваться.

### Архитектура Rust
```rust
pub fn get_point_buy_budget(power_level: PowerLevel) -> i32 {
    match power_level {
        PowerLevel::Normal => 24,
        PowerLevel::Heroic => 36,
        PowerLevel::Epic => 48,
        PowerLevel::Superhuman => 60,
    }
}

pub fn get_point_buy_limit(power_level: PowerLevel) -> u8 {
    match power_level {
        PowerLevel::Normal | PowerLevel::Heroic => 21,
        PowerLevel::Epic | PowerLevel::Superhuman => u8::MAX, // Нет лимита
    }
}
```

---

## Опциональное правило: Характеристика Образования (Education)
Интеграция характеристики `EDU` в систему Point-Based.

### Механика / Концепция
Если в игре используется `EDU`, Гейммастер назначает базовое значение `EDU`, основываясь на возрасте персонажа (из [[Step Three - Age|Шага 3]]) и его предыстории. Игрок может изменить это значение, используя очки из пула Point-Based.
- **Стоимость**: Каждое очко `EDU` стоит **3 очка** из бюджета.

### Архитектура Rust
```rust
impl PointBuySystem {
    pub fn get_edu_cost(assigned_base_edu: u8, target_edu: u8) -> i32 {
        let diff = target_edu as i32 - assigned_base_edu as i32;
        diff * 3
    }
}
```

---

## Опциональное правило: Культурные Модификаторы (Cultural Modifiers or Non-Human Characters)
Использование расовых и культурных шаблонов при генерации.

### Механика / Концепция
Если Гейммастер разрешает использование культурных модификаторов (см. [[Introduction and Power Level|Cultural Modifiers]]), они должны применяться к характеристикам **после** того, как закончена базовая генерация (или покупка за очки).
Если в игре доступны нечеловеческие персонажи (non-human characters) со своими модификаторами стартовых характеристик, Гейммастер должен соответствующим образом скорректировать их стартовые очки или начальные характеристики. Рекомендации по нечеловеческим расам находятся в [[Chapter 11: Creatures]].