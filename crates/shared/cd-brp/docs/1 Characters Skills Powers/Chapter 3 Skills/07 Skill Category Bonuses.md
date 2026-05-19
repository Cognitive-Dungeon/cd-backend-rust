**Путь в Obsidian:** `📁 Chapter 3: Skills / 📄 Skill Category Bonuses and Base Chances.md`

# Оглавление файла
- [[#Опциональное правило: Бонусы категорий навыков (Skill Category Bonuses)]]
- [[#Базовые шансы навыков (Base Chances)]]

---

## Опциональное правило: Бонусы категорий навыков (Skill Category Bonuses)
Детальное описание системы модификации навыков на основе характеристик.

### Механика / Концепция
Если `Gamemaster` (Гейммастер) и игроки решают использовать эту опциональную систему, характеристики персонажа (`Characteristics`) начинают напрямую влиять на стартовые значения навыков (`Skill Ratings`). 

Каждый навык сгруппирован в одну из 6 категорий (Combat, Communication, Manipulation, Mental, Perception, Physical). Каждая категория связана с одной **первичной (Primary)**, одной или несколькими **вторичными (Secondary)** и, в некоторых случаях, **негативной (Negative)** характеристикой. 

Алгоритм вычисления бонуса (или штрафа) для конкретной категории:
1. **Primary (Первичная)**: Прибавляет `+1%` за каждое очко характеристики больше `10`, и отнимает `-1%` за каждое очко меньше `10`.
2. **Secondary (Вторичная)**: Прибавляет `+1%` за каждые **полные 2 очка** больше `10`, и отнимает `-1%` за каждые **полные 2 очка** меньше `10`. Округление происходит вниз (отбрасывание остатка).
3. **Negative (Негативная)**: Отнимает `-1%` за каждое очко больше `10`, и прибавляет `+1%` за каждое очко меньше `10`.

Этот итоговый бонус (который может быть как положительным, так и отрицательным) прибавляется к [[#Базовые шансы навыков (Base Chances)|базовому шансу]] (Base Chance) **всех** навыков, входящих в данную категорию.

**Матрица модификаторов категорий (Skill Category Modifiers):**

| Категория (Category) | Первичная (Primary) | Вторичная (Secondary) | Негативная (Negative) |
| :--- | :--- | :--- | :--- |
| **Combat skills** | DEX | INT, STR | — |
| **Communication skills** | INT | POW, CHA | — |
| **Manipulation skills** | DEX | INT, STR | — |
| **Mental skills** | INT | POW, EDU | — |
| **Perception skills** | INT | POW, CON | — |
| **Physical skills** | DEX | STR, CON | SIZ |

> For example, your character has the following characteristics: STR 14, CON 13, INT 8, SIZ 12, POW 10, DEX 12, and CHA 8. Their skill category bonuses are:
> - Combat: +3% (+2 for DEX, +2 for STR, –1 for INT)
> - Communication: –3% (–2 for INT, 0 for POW, –1 for CHA)
> - Manipulation: +3% (+2 for DEX, –1 for INT, +2 for STR)
> - Mental: –2% (–2 from INT, 0 for POW, EDU is not used in this campaign)
> - Perception: –1% (–2 for INT, 0 for POW, +1 for CON)
> - Physical: +3% (+2 for DEX, +2 for STR, +1 for CON, –2 for SIZ)

**Сводная таблица (Skill Bonus Table) для быстрого расчета одной характеристики:**

| Значение Характеристики | Primary Mod | Secondary Mod | Negative Mod |
| :--- | :--- | :--- | :--- |
| **1** | -9% | -4% | +9% |
| **2** | -8% | -4% | +8% |
| **3** | -7% | -3% | +7% |
| ... | ... | ... | ... |
| **8** | -2% | -1% | +2% |
| **9** | -1% | -0% | +1% |
| **10** | +0% | +0% | -0% |
| **11** | +1% | +0% | -1% |
| **12** | +2% | +1% | -2% |
| **13** | +3% | +1% | -3% |
| ... | ... | ... | ... |
| **20** | +10% | +5% | -10% |
| **21** | +11% | +5% | -11% |
| **Далее (Etc.)** | +1% за очко | +1% за 2 очка | -1% за очко |

### Опциональное правило: Упрощенные бонусы (Simpler Skill Bonuses)
Если базовый метод кажется слишком сложным, система предлагает упрощенную альтернативу.
Вместо учета нескольких характеристик и вычисления штрафов, бонус категории равен: `Primary Characteristic / 2` (с округлением вверх `ceil`).
- **Combat**: `DEX / 2`
- **Communication**: `CHA / 2` (Обратите внимание, Primary изменен с INT на CHA)
- **Manipulation**: `DEX / 2`
- **Mental**: `INT / 2`
- **Perception**: `POW / 2` (Обратите внимание, Primary изменен с INT на POW)
- **Physical**: `STR / 2` (Обратите внимание, Primary изменен с DEX на STR)

### Архитектура Rust
```rust
pub enum SkillCategory {
    Combat,
    Communication,
    Manipulation,
    Mental,
    Perception,
    Physical,
}

pub struct Characteristics {
    pub str: u8, pub con: u8, pub siz: u8, pub int: u8,
    pub pow: u8, pub dex: u8, pub cha: u8, pub edu: Option<u8>,
}

impl Characteristics {
    // Вспомогательные функции для расчета сложных бонусов
    fn primary_mod(stat: u8) -> i16 { stat as i16 - 10 }
    fn secondary_mod(stat: u8) -> i16 { (stat as i16 - 10) / 2 }
    fn negative_mod(stat: u8) -> i16 { -(stat as i16 - 10) }

    pub fn calculate_complex_category_bonus(&self, category: &SkillCategory) -> i16 {
        match category {
            SkillCategory::Combat => Self::primary_mod(self.dex) + Self::secondary_mod(self.int) + Self::secondary_mod(self.str),
            SkillCategory::Communication => Self::primary_mod(self.int) + Self::secondary_mod(self.pow) + Self::secondary_mod(self.cha),
            SkillCategory::Manipulation => Self::primary_mod(self.dex) + Self::secondary_mod(self.int) + Self::secondary_mod(self.str),
            SkillCategory::Mental => {
                let mut base = Self::primary_mod(self.int) + Self::secondary_mod(self.pow);
                if let Some(edu_val) = self.edu { base += Self::secondary_mod(edu_val); }
                base
            },
            SkillCategory::Perception => Self::primary_mod(self.int) + Self::secondary_mod(self.pow) + Self::secondary_mod(self.con),
            SkillCategory::Physical => Self::primary_mod(self.dex) + Self::secondary_mod(self.str) + Self::secondary_mod(self.con) + Self::negative_mod(self.siz),
        }
    }

    pub fn calculate_simple_category_bonus(&self, category: &SkillCategory) -> u16 {
        let primary_stat = match category {
            SkillCategory::Combat => self.dex,
            SkillCategory::Communication => self.cha,
            SkillCategory::Manipulation => self.dex,
            SkillCategory::Mental => self.int,
            SkillCategory::Perception => self.pow,
            SkillCategory::Physical => self.str,
        };
        (primary_stat as f32 / 2.0).ceil() as u16
    }
}
```

---

## Базовые шансы навыков (Base Chances)
Определение врожденной или общекультурной способности персонажа совершать действия.

### Механика / Концепция
Большинство здоровых и физически развитых людей могут размахнуться дубиной, забраться на дерево или говорить на родном языке. Поэтому каждый навык имеет ассоциированный с ним **Базовый Шанс** (Base Chance). Это то значение (рейтинг в процентах), которое есть у персонажа изначально, **до** того как в навык будут вложены какие-либо очки (профессиональные или личные).

Если в игре используются бонусы категорий (Skill Category Bonuses), они прибавляются (или отнимаются) именно от этого Базового Шанса.

**Влияние сеттинга на базовые шансы:**
Базовый шанс сильно зависит от эпохи или сеттинга кампании. Гейммастер имеет полное право изменять базовые шансы навыков, чтобы они соответствовали духу мира.
- Персонажи в средневековой Европе будут иметь более высокий базовый шанс в навыке `Knowledge (Religion)`, чем современные граждане США.
- Граждане США будут иметь преимущество (базовый шанс) в медицинских навыках благодаря основам первой помощи (`First Aid`), преподаваемым в школах и на рабочих местах.

В описании каждого навыка (в секции словаря навыков) обычно указывается стандартный базовый шанс, а иногда и несколько вариантов для разных эпох.

### Архитектура Rust
```rust
pub struct SkillDefinition {
    pub id: String,
    pub name: String,
    pub category: SkillCategory,
    pub base_chance: u16, // Может быть переопределен конфигурацией сеттинга
}

pub struct CharacterSkill {
    pub definition_id: String,
    pub allocated_points: u16,
}

impl CharacterSkill {
    pub fn get_total_rating(
        &self, 
        def: &SkillDefinition, 
        category_bonus: i16
    ) -> u16 {
        let mut total = def.base_chance as i32 + category_bonus as i32 + self.allocated_points as i32;
        if total < 0 { total = 0; }
        total as u16
    }
}
```

### Граничные случаи и Критические исходы
Если `Skill Category Bonus` является отрицательным числом (штрафом), он может опустить сумму `Base Chance + Bonus` ниже нуля. В этом случае эффективный базовый шанс становится равен `0%` (персонаж настолько плох в характеристиках, что лишается даже врожденного общекультурного шанса на успех в этой области). Значение навыка не может быть отрицательным.