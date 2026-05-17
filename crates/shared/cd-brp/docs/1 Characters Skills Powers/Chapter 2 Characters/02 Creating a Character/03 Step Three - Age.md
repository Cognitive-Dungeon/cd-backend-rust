# Оглавление файла
- [[#Базовый возраст (Starting Age)]]
- [[#Влияние возраста на очки навыков (Age and Skill Points)]]
- [[#Опциональное правило: Старение и Бездействие (Ageing and Inaction)]]
- [[#Опциональное правило: Характеристика Образования (Education / EDU)]]

---

## Базовый возраст (Starting Age)
Возраст персонажа определяет его жизненный опыт, доступные профессиональные навыки и физическое состояние. 

### Механика / Концепция
1. **Возраст по умолчанию**: Базовый стартовый возраст персонажа равен `17 + 1D6` (от 18 до 23 лет). Гейммастер может изменить этот диапазон в зависимости от сеттинга.
2. **Изменение возраста**: Игрок может сделать персонажа старше или моложе базового, если Гейммастер это одобрит. Изменение возраста напрямую влияет на количество очков профессиональных навыков (Professional Skill Points) в [[07 Step Seven - Profession and Skills|Шаге 7]].

### Архитектура Rust
```rust
pub struct AgeOptions {
    pub base_age: u16, // Обычно вычисляется как 17 + roll(1, d6)
    pub min_allowed_age: u16, // Обычно 18, но GM может переопределить
}

pub fn roll_default_starting_age() -> u16 {
    17 + roll(1, d6) as u16
}
```

---

## Влияние возраста на очки навыков (Age and Skill Points)
Отклонение от базового возраста модифицирует количество профессиональных очков навыков. Модификатор зависит от `Power Level` кампании.

### Механика / Концепция
1. **Увеличение возраста (Старше)**: За каждые **полные 10 лет**, добавленные к базовому (выброшенному на кубиках) стартовому возрасту, персонаж получает бонус к профессиональным очкам навыков:
   - `Normal`: +0 очков.
   - `Heroic`: +20 очков.
   - `Epic`: +30 очков.
   - `Superhuman`: +40 очков.
   *Дробные части от 10 лет не учитываются (добавление 9 лет дает +0).*
2. **Уменьшение возраста (Моложе)**: За каждый **один год** ниже минимального возраста (по умолчанию 18 лет), из профессиональных очков навыков вычитается:
   - `Normal`: -0 очков.
   - `Heroic`: -20 очков.
   - `Epic`: -30 очков.
   - `Superhuman`: -40 очков.
   *Гейммастер может ограничивать доступные профессии для персонажей младше 18 лет.*

### Архитектура Rust
```rust
pub fn calculate_age_skill_modifier(
    rolled_base_age: u16, 
    chosen_age: u16, 
    min_age: u16, 
    power_level: PowerLevelTarget
) -> i32 {
    let mut modifier = 0;

    // Бонус за каждые полные 10 лет старше базового броска
    if chosen_age > rolled_base_age {
        let decades_older = (chosen_age - rolled_base_age) / 10;
        let bonus_per_decade = match power_level {
            PowerLevelTarget::Normal => 0,
            PowerLevelTarget::Heroic => 20,
            PowerLevelTarget::Epic => 30,
            PowerLevelTarget::Superhuman => 40,
            PowerLevelTarget::None => 0,
        };
        modifier += (decades_older as i32) * bonus_per_decade;
    }

    // Штраф за каждый год младше минимального (обычно 18)
    if chosen_age < min_age {
        let years_younger = min_age - chosen_age;
        let penalty_per_year = match power_level {
            PowerLevelTarget::Normal => 0,
            PowerLevelTarget::Heroic => 20,
            PowerLevelTarget::Epic => 30,
            PowerLevelTarget::Superhuman => 40,
            PowerLevelTarget::None => 0,
        };
        modifier -= (years_younger as i32) * penalty_per_year;
    }

    modifier
}
```

### Граничные случаи и Критические исходы
Формула для "постарше" использует `rolled_base_age` (то, что выпало на `17 + 1D6`), а формула для "помоложе" использует фиксированный лимит `min_age` (18 лет). Если игрок выбросил 20 лет, а выбрал возраст 25, он не получит бонус (так как разница меньше 10 лет).

---

## Опциональное правило: Старение и Бездействие (Ageing and Inaction)
Правила физической деградации персонажей со временем или в связи с ранним возрастом. Гейммастер может полностью игнорировать эти правила в зависимости от уровня силы кампании (например, чтобы позволить играть за детей-гениев или сверхсильных стариков).

### Механика / Концепция
1. **Деградация в старости**:
   - По достижении **50 лет**, и за каждые **полные 10 лет** после этого: уменьшите одну из характеристик (`STR`, `CON`, `DEX` или `CHA` — на выбор игрока) на `-1`.
   - По достижении **80 лет**, и за каждые **полные 10 лет** после этого: уменьшите **три** из этих характеристик на `-1`.
2. **Незрелость (Дети)**:
   - За каждый **один год** ниже выброшенного базового возраста (`rolled base age`), уменьшите любую характеристику (кроме `EDU`) на `-1` (на выбор игрока).
   - Гейммастер может потребовать, чтобы одной из пониженных характеристик обязательно был `SIZ` (Размер).
   - Эти потерянные очки могут быть восстановлены в процессе игры через опыт, тренировки или естественный рост со временем.

### Архитектура Rust
```rust
pub struct AgeingResult {
    pub stats_to_reduce_by_one: u8,
    pub is_child_penalty: u16, // Сколько статов нужно понизить на 1
}

pub fn calculate_ageing_effects(rolled_base_age: u16, current_age: u16) -> AgeingResult {
    let mut stats_to_reduce_by_one = 0;
    let mut is_child_penalty = 0;

    // Штрафы старости
    if current_age >= 50 {
        // За 50, 60, 70 дается 1 снижение. За 80, 90+ дается 3 снижения.
        let mut age_iter = 50;
        while age_iter <= current_age {
            if age_iter >= 80 {
                stats_to_reduce_by_one += 3;
            } else {
                stats_to_reduce_by_one += 1;
            }
            age_iter += 10;
        }
    }

    // Штрафы детства
    if current_age < rolled_base_age {
        is_child_penalty = rolled_base_age - current_age;
    }

    AgeingResult {
        stats_to_reduce_by_one,
        is_child_penalty,
    }
}
```

---

## Опциональное правило: Характеристика Образования (Education / EDU)
Если в игре используется опциональная характеристика `EDU` (введенная в [[Step One - Name and Characteristics|Шаге 1]]), возраст напрямую связан со временем, потраченным на учебу.

### Механика / Концепция
1. **Минимальный возраст с EDU**: Стартовый возраст персонажа должен быть **не меньше**, чем значение `EDU + 5` (это симулирует годы, проведенные в обучении). 
2. **Рост EDU от возраста**: За каждые **полные 10 лет**, добавленные к стартовому возрасту, характеристика `EDU` увеличивается на `+1`. 
   - *Важно: При увеличении EDU необходимо не забыть пересчитать очки навыков в Шаге 7, если пул навыков зависит от EDU.*

### Архитектура Rust
```rust
pub fn validate_edu_age(edu_stat: u8, chosen_age: u16) -> Result<(), &'static str> {
    if chosen_age < (edu_stat as u16 + 5) {
        return Err("Age must be at least EDU + 5");
    }
    Ok(())
}

pub fn apply_age_to_edu(edu_stat: &mut u8, rolled_base_age: u16, chosen_age: u16) {
    if chosen_age > rolled_base_age {
        let decades_older = (chosen_age - rolled_base_age) / 10;
        *edu_stat = edu_stat.saturating_add(decades_older as u8);
    }
}
```