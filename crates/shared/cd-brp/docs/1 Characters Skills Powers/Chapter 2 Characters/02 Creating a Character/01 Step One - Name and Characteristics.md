# Оглавление файла
- [[#Базовая генерация (Name and Characteristics)]]
- [[#Опциональное правило: Покупка за очки (Point-Based Character Creation)]]
- [[#Опциональные правила: Вариации бросков и Модификаторы]]

---

## Базовая генерация (Name and Characteristics)
Первый шаг в создании персонажа — выбор имени, пола и определение базовых числовых значений характеристик случайным образом. 

### Механика / Концепция
1. **Имя и Пол**: Игрок выбирает имя и пол персонажа, подходящие под сеттинг.
2. **Броски кубиков**:
   - Для характеристик `STR`, `CON`, `POW`, `DEX` и `CHA` бросается `roll(3, d6)`. (Диапазон: 3–18).
   - Для характеристик `INT` и `SIZ` бросается `roll(2, d6) + 6`. (Диапазон: 8–18). Таким образом, интеллект и физический размер человека не могут быть экстремально низкими при стандартной генерации.
3. **Перераспределение (Redistribute)**: После бросков игрок может перераспределить до `3` очков между любыми характеристиками (отнять от одних и прибавить к другим).
4. **Лимиты**: На данном этапе ни одна характеристика не может превышать `21`.
5. **Сброс**: С разрешения Гейммастера игрок может отменить результаты и начать процесс бросков заново.

Если в игре используются магические или иные силы, Гейммастер может разрешить увеличить стартовые характеристики сверх этих лимитов (детальная механика в [[Chapter 4: Powers]]).

### Архитектура Rust
Генерация персонажа хорошо ложится на паттерн `Builder`.

```rust
pub struct CharacterIdentity {
    pub name: String,
    pub gender: String,
}

pub struct BaseCharacteristics {
    pub str: u8,
    pub con: u8,
    pub siz: u8,
    pub int: u8,
    pub pow: u8,
    pub dex: u8,
    pub cha: u8,
}

pub struct CharacterBuilder {
    pub identity: Option<CharacterIdentity>,
    pub stats: Option<BaseCharacteristics>,
    pub free_points_to_redistribute: u8, // Изначально 3
}

impl CharacterBuilder {
    // Стандартная генерация 3d6 и 2d6+6
    pub fn roll_standard_stats(&mut self) {
        self.stats = Some(BaseCharacteristics {
            str: roll(3, d6),
            con: roll(3, d6),
            pow: roll(3, d6),
            dex: roll(3, d6),
            cha: roll(3, d6),
            int: roll(2, d6) + 6,
            siz: roll(2, d6) + 6,
        });
        self.free_points_to_redistribute = 3;
    }

    // Логика перемещения очков
    pub fn redistribute(
        &mut self, 
        from_stat: &mut u8, 
        to_stat: &mut u8, 
        amount: u8
    ) -> Result<(), &'static str> {
        if amount > self.free_points_to_redistribute {
            return Err("Not enough redistribution points");
        }
        if *from_stat <= amount {
            return Err("Cannot reduce stat below 1");
        }
        if *to_stat + amount > 21 {
            return Err("Stat cannot exceed 21 at creation");
        }

        *from_stat -= amount;
        *to_stat += amount;
        self.free_points_to_redistribute -= amount;
        
        Ok(())
    }
}
```

### Граничные случаи и Критические исходы
Максимальное базовое значение до перераспределения — 18. После перераспределения 3-х очков абсолютный жесткий лимит равен 21. `INT` и `SIZ` не могут упасть ниже 8 при начальном броске, но правила не запрещают опустить их ниже 8 при перераспределении (главное условие — не превышать 21 при прибавлении).

---

## Опциональное правило: Покупка за очки (Point-Based Character Creation)
Вместо бросков кубиков игроки могут "покупать" значения характеристик из фиксированного пула очков.

### Механика / Концепция
1. **Базовое значение**: Все характеристики (`STR`, `CON`, `SIZ`, `INT`, `POW`, `DEX`, `CHA`) изначально равны `10`.
2. **Пул очков**: Зависит от [[01 Power Level#Уровень Силы (Power Level)|Power Level]]:
   - `Normal`: 24 очка.
   - `Heroic`: 36 очков.
   - `Epic`: 48 очков.
   - `Superhuman`: 60 очков.
3. **Стоимость**: 
   - `STR`, `CON`, `SIZ`, `CHA` стоят `1` очко пула за `+1` к значению.
   - `DEX`, `INT`, `POW` стоят `3` очка пула за `+1` к значению.
4. **Уменьшение характеристик**: Игрок может опускать характеристики ниже `10` (вплоть до минимума `3`), чтобы получить дополнительные очки в пул. За каждый отнятый пункт физических характеристик дается `+1` в пул, за ментальные/ловкость дается `+3`.
5. **Лимиты**: Значения должны оставаться в пределах от `3` до `21`. Однако для уровней `Epic` и `Superhuman` ограничение в `21` игнорируется.
6. В играх со сверхспособностями неиспользованные очки характеристик можно перевести в бюджет способностей (powers budget).

### Архитектура Rust
Реализация алгоритма валидации покупки атрибутов:

```rust
pub enum PowerLevel { Normal, Heroic, Epic, Superhuman }

pub fn get_starting_points(level: PowerLevel) -> i32 {
    match level {
        PowerLevel::Normal => 24,
        PowerLevel::Heroic => 36,
        PowerLevel::Epic => 48,
        PowerLevel::Superhuman => 60,
    }
}

pub fn calculate_point_buy_cost(stats: &BaseCharacteristics) -> i32 {
    let mut total_cost = 0;
    
    // Стоимость 1 к 1
    total_cost += (stats.str as i32 - 10) * 1;
    total_cost += (stats.con as i32 - 10) * 1;
    total_cost += (stats.siz as i32 - 10) * 1;
    total_cost += (stats.cha as i32 - 10) * 1;

    // Стоимость 3 к 1
    total_cost += (stats.dex as i32 - 10) * 3;
    total_cost += (stats.int as i32 - 10) * 3;
    total_cost += (stats.pow as i32 - 10) * 3;

    total_cost
}

pub fn validate_point_buy(stats: &BaseCharacteristics, level: PowerLevel) -> Result<(), &'static str> {
    let pool = get_starting_points(level);
    let cost = calculate_point_buy_cost(stats);
    
    if cost > pool {
        return Err("Exceeded point buy budget");
    }
    
    let max_limit = match level {
        PowerLevel::Normal | PowerLevel::Heroic => 21,
        PowerLevel::Epic | PowerLevel::Superhuman => 255, // Нет лимита
    };
    
    let all_stats = [stats.str, stats.con, stats.siz, stats.int, stats.pow, stats.dex, stats.cha];
    for &stat in all_stats.iter() {
        if stat < 3 { return Err("Stat cannot be below 3"); }
        if stat > max_limit { return Err("Stat exceeded maximum limit"); }
    }
    
    Ok(())
}
```

### Граничные случаи и Критические исходы
Атрибуты могут уходить "в минус" по стоимости (давая сдачу), если игрок опускает значение ниже 10. Формула `(stat - 10) * cost` корректно обрабатывает возврат очков, уходя в отрицательные значения (например, INT 9 даст `-3` к общей стоимости).

---

## Опциональные правила: Вариации бросков и Модификаторы
Альтернативные подходы к случайной генерации и добавлению дополнительных метрик.

### Механика / Концепция
- **Выбор значений (Choosing Characteristic Values)**: Вместо бросков кубиков под конкретную характеристику, игрок кидает `roll(3, d6)` семь раз, а затем распределяет полученные результаты между характеристиками по своему усмотрению. Ограничение: значения для `SIZ` и `INT` не могут быть ниже `8` (если выпали только низкие числа, их нельзя поставить в эти характеристики). 3 очка перераспределения всё еще доступны.
- **Высокие стартовые характеристики (Higher Starting Characteristics)**: Для высокоуровневых игр (Heroic и выше), игрок бросает `roll(2, d6) + 6` для **всех** характеристик, а не только для `INT` и `SIZ`.
- **Характеристика Образования (Education / EDU)**: Добавляет 8-ю характеристику в игру (характерно для современных/футуристичных сеттингов с формальным обучением). 
  - Бросается как `roll(2, d6) + 6`.
  - Значение `12` эквивалентно окончанию старшей школы.
  - В системе Point-Based стоит `3` очка пула за каждый `+1`.
- **Культурные модификаторы (Cultural Modifiers / Non-human Characters)**: Стартовые значения кубиков, лимиты и финальные характеристики могут меняться в зависимости от расы, вида (эльфы, гномы) или культуры. Расовые модификаторы применяются **после** базовой генерации. Правила создания нелюдей описаны в [[Chapter 11: Creatures]].

### Архитектура Rust
Добавление опциональных флагов для модификации процесса `CharacterBuilder`:

```rust
pub struct GenerationConfig {
    pub use_point_buy: bool,
    pub allow_choose_values: bool,     // 7 бросков 3d6 с выбором
    pub use_high_starting_stats: bool, // Все статы как 2d6+6
    pub use_education_stat: bool,      // Добавляет EDU
}

pub struct OptionalCharacteristics {
    pub edu: Option<u8>,
}
```