# Оглавление файла
- [[#Выбор Профессии и Бюджет Профессиональных Навыков (Professional Skill Pool)]]
- [[#Лимиты Навыков по Уровню Силы (Skill Caps)]]
- [[#Бюджет Личных Навыков (Personal Skill Pool)]]
- [[#Итоговый Расчет Значений Навыков (Final Skill Calculation)]]
- [[#Опциональное правило: Влияние Характеристики Образования (Education / EDU)]]
- [[#Опциональное правило: Увеличенные Личные Навыки (Increased Personal Skill Points)]]

---

## Выбор Профессии и Бюджет Профессиональных Навыков (Professional Skill Pool)
Этот шаг определяет, чем персонаж занимался до начала игры, и предоставляет основной пул очков для распределения в профильные навыки.

### Механика / Концепция
Игрок выбирает профессию (`Profession`) из доступного списка. Профессия определяет фиксированный список навыков (обычно около 10), в которые персонаж может вкладывать профессиональные очки. В некоторых случаях профессия дает особые преимущества (например, доступ к магии). 

Размер пула профессиональных очков (`Professional Skill Pool`) зависит от [[Introduction and Power Level#Уровень Силы (Power Level)|уровня силы кампании]]:
- **Normal**: 250 очков.
- **Heroic**: 325 очков.
- **Epic**: 400 очков.
- **Superhuman**: 500 очков.

Эти очки могут быть потрачены **только** на навыки, входящие в список выбранной профессии.

### Архитектура Rust
```rust
pub enum PowerLevel { Normal, Heroic, Epic, Superhuman }

pub struct Profession {
    pub name: String,
    pub allowed_skills: Vec<String>,
    pub grants_powers: bool,
}

pub fn get_professional_pool_size(level: PowerLevel) -> u16 {
    match level {
        PowerLevel::Normal => 250,
        PowerLevel::Heroic => 325,
        PowerLevel::Epic => 400,
        PowerLevel::Superhuman => 500,
    }
}
```

---

## Лимиты Навыков по Уровню Силы (Skill Caps)
Система устанавливает жесткие ограничения на то, насколько высоким может быть рейтинг навыка (`Skill Rating`) на этапе создания персонажа.

### Механика / Концепция
Максимально допустимое значение навыка при генерации также зависит от `Power Level`:
- **Normal**: Не более `75%`.
- **Heroic**: Не более `90%`.
- **Epic**: Не более `101%`.
- **Superhuman**: Без ограничений (No limit).

Если рейтинг навыка превышает этот лимит до распределения очков (например, за счет суммы базового шанса и бонусов категорий/личности), игрок **не имеет права** тратить на этот навык ни профессиональные, ни личные очки. Любые излишки очков должны быть потрачены на другие навыки.

### Граничные случаи и Критические исходы
Существует критическое правило "естественного превышения": если сумма `Base Chance` + `Personality Bonus` + `Skill Category Bonus` *уже* превышает лимит (например, равна `78%` в Normal-игре), навык остается равным `78%`. Система не "обрезает" его до `75%`, но строго запрещает вкладывать в него очки из пулов профессии или личных навыков.

---

## Бюджет Личных Навыков (Personal Skill Pool)
Помимо профессиональных интересов, у персонажа есть хобби и сторонние знания.

### Механика / Концепция
После распределения профессиональных очков вычисляется пул личных очков навыков (`Personal Skill Pool`). 
- **Формула**: `INT × 10`.
Эти очки можно потратить на **любые** навыки в игре (с одобрения Гейммастера), включая те, что не входят в список профессии.

**Лимит на непрофильные навыки:**
Гейммастер может наложить ограничение на то, насколько высоко можно прокачать навык, не входящий в профессию (soft cap). Этот лимит составляет:
- **Normal**: `50%`.
- **Heroic**: `75%`.
- **Epic**: `90%`.
- **Superhuman**: `100%`.
Навыки, которые "естественным образом" (за счет бонусов) превышают этот лимит, также не могут быть улучшены личными очками.

### Архитектура Rust
```rust
pub fn get_personal_pool_size(int_stat: u8) -> u16 {
    int_stat as u16 * 10
}

pub fn get_skill_caps(level: PowerLevel) -> (u16, u16) {
    // Возвращает кортеж: (Абсолютный лимит, Лимит для непрофильных навыков)
    match level {
        PowerLevel::Normal => (75, 50),
        PowerLevel::Heroic => (90, 75),
        PowerLevel::Epic => (101, 90),
        PowerLevel::Superhuman => (u16::MAX, 100),
    }
}
```

---

## Итоговый Расчет Значений Навыков (Final Skill Calculation)
Алгоритм того, как все компоненты собираются в единый показатель навыка.

### Механика / Концепция
Во время генерации персонажа очки профессии и личные очки должны учитываться раздельно для валидации лимитов. Итоговый рейтинг навыка (`Skill Rating`) вычисляется как сумма:
1. `Base Chance` (Базовый шанс навыка).
2. `Personality Type Bonus` (Опционально: +20% из [[Step Six - Personality (Optional)]]).
3. `Skill Category Bonus` (Опционально: бонус от характеристик из Шага 5).
4. `Professional Skill Points` (Очки из пула профессии).
5. `Personal Skill Points` (Очки из личного пула).

### Архитектура Rust
```rust
pub struct SkillAllocation {
    pub skill_name: String,
    pub is_professional: bool,
    pub base_chance: u16,
    pub personality_bonus: u16,
    pub category_bonus: i16,
    pub allocated_prof_points: u16,
    pub allocated_personal_points: u16,
}

impl SkillAllocation {
    pub fn current_total(&self) -> u16 {
        let mut total: i32 = self.base_chance as i32 
                           + self.personality_bonus as i32 
                           + self.category_bonus as i32 
                           + self.allocated_prof_points as i32 
                           + self.allocated_personal_points as i32;
        if total < 0 { total = 0; }
        total as u16
    }

    // Валидация перед добавлением очков
    pub fn can_add_points(&self, target_cap: u16) -> bool {
        let natural_base: i32 = self.base_chance as i32 
                              + self.personality_bonus as i32 
                              + self.category_bonus as i32;
        // Если база уже выше или равна капу, очки добавлять нельзя
        natural_base < target_cap as i32 && self.current_total() < target_cap
    }
}
```

---

## Опциональное правило: Влияние Характеристики Образования (Education / EDU)
Если в игре используется опциональная характеристика `EDU`, она заменяет фиксированный пул профессиональных очков.

### Механика / Концепция
Вместо базовых `250 / 325 / 400 / 500` очков профессии, размер пула зависит от уровня образования персонажа:
- **Normal**: `EDU × 20`
- **Heroic**: `EDU × 25`
- **Epic**: `EDU × 30`
- **Superhuman**: `EDU × 40`

### Архитектура Rust
```rust
pub fn get_edu_professional_pool_size(level: PowerLevel, edu_stat: u8) -> u16 {
    let multiplier = match level {
        PowerLevel::Normal => 20,
        PowerLevel::Heroic => 25,
        PowerLevel::Epic => 30,
        PowerLevel::Superhuman => 40,
    };
    edu_stat as u16 * multiplier
}
```

---

## Опциональное правило: Увеличенные Личные Навыки (Increased Personal Skill Points)
Для кампаний, где персонажи должны быть выдающимися экспертами с множеством сторонних знаний.

### Механика / Концепция
Изменяет множитель для `Personal Skill Pool` в зависимости от уровня силы (вместо стандартного `INT × 10`):
- **Heroic**: `INT × 15`
- **Epic**: `INT × 20`
- **Superhuman**: `INT × 25`
*(Для Normal остается `INT × 10`).*

### Архитектура Rust
```rust
pub fn get_increased_personal_pool_size(level: PowerLevel, int_stat: u8) -> u16 {
    let multiplier = match level {
        PowerLevel::Normal => 10,
        PowerLevel::Heroic => 15,
        PowerLevel::Epic => 20,
        PowerLevel::Superhuman => 25,
    };
    int_stat as u16 * multiplier
}
```