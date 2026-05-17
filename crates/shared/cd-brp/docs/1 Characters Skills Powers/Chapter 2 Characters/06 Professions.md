# Оглавление файла
- [[#Профессии (Professions)]]
- [[#Адаптация профессий под сеттинг (Settings and Skills)]]
- [[#Опциональное правило: Бонусы категорий навыков (Skill Category Bonuses)]]
- [[#Опциональное правило: Упрощенные бонусы навыков (Simpler Skill Bonuses)]]

---

## Профессии (Professions)
Профессия определяет род занятий, которым персонаж владел до начала игры. Это отправная точка, которая направляет распределение стартовых очков навыков.

### Концепция
В дополнение к характеристикам (`Characteristics`), персонаж описывается его навыками (`Skills`). Профессия — это шаблон (пакет), предоставляющий список подходящих навыков (обычно 10 штук), в которые игрок может вложить очки из бюджета профессиональных навыков (Professional Skill Pool). 

Профессия не является классом: она не ограничивает то, кем персонаж может стать в будущем, а лишь определяет его стартовые знания и средний уровень богатства (`Wealth`).

### Архитектура Rust
Поскольку профессии зависят от жанра и сеттинга, их не следует хардкодить как перечисления (`enum`). Лучший подход — использовать data-driven архитектуру, загружая список профессий из конфигурационных файлов (например, JSON или TOML).

```rust
pub struct ProfessionTemplate {
    pub id: String,                 // Уникальный идентификатор (напр. "warrior")
    pub name: String,               // Локализованное название (напр. "Samurai")
    pub base_wealth: String,        // Уровень богатства (напр. "Average")
    pub skill_identifiers: Vec<String>, // Список ID доступных навыков
    pub allows_magic: bool,         // Дает ли профессия доступ к магии/силам
}

pub struct CharacterProfession {
    pub template_id: String,
    // В процессе игры профессия может поменяться или стать неактуальной,
    // но она сохраняется как элемент предыстории.
}
```

---

## Адаптация профессий под сеттинг (Settings and Skills)
Правила по изменению профессий в зависимости от игрового мира.

### Концепция
Базовые списки профессий являются обобщенными (generic). В зависимости от сеттинга `Gamemaster` (Гейммастер) может:
1. **Переименовывать профессии**: Адаптировать названия под реалии мира.
> For example, in campaign set in ancient Japan, your gamemaster tells you that the warrior profession is called samurai, assassin is a ninja, the thief is a bandit, the criminal is a yakuza, and the noble is a courtier.
2. **Заменять навыки**: Если какой-либо навык в шаблоне профессии не имеет смысла в текущей эре или сеттинге (например, `Computer Use` в Средневековье), он заменяется на эквивалентный навык или другую специальность.

---

## Опциональное правило: Бонусы категорий навыков (Skill Category Bonuses)
Продвинутая система, в которой значения базовых характеристик (`Characteristics`) влияют на стартовые шансы всех навыков в игре.

### Механика / Концепция
Каждый навык принадлежит к одной из 6 категорий. Для каждой категории вычисляется уникальный бонус, основанный на одной **первичной** (Primary), одной или нескольких **вторичных** (Secondary) и, в одном случае, **негативной** (Negative) характеристиках.

Этот вычисленный бонус прибавляется к [[Terms Used in Basic Roleplaying#Base Chance|базовому шансу]] (Base Chance) каждого навыка в соответствующей категории.

**Формула расчета:**
- **Primary Characteristic**: `+1%` за каждое очко свыше `10`; `-1%` за каждое очко ниже `10`.
- **Secondary Characteristic**: `+1%` за каждые полные `2` очка свыше `10`; `-1%` за каждые полные `2` очка ниже `10` (округление вниз).
- **Negative Characteristic**: `-1%` за каждое очко свыше `10`; `+1%` за каждое очко ниже `10`.

**Матрица привязки характеристик к категориям (Skill Category Modifiers):**
- **Combat skills** (Боевые): Primary = `DEX` | Secondary = `INT`, `STR` | Negative = Нет
- **Communication skills** (Социальные): Primary = `INT` | Secondary = `POW`, `CHA` | Negative = Нет
- **Manipulation skills** (Манипуляция): Primary = `DEX` | Secondary = `INT`, `STR` | Negative = Нет
- **Mental skills** (Ментальные): Primary = `INT` | Secondary = `POW`, `EDU` (если используется) | Negative = Нет
- **Perception skills** (Восприятие): Primary = `INT` | Secondary = `POW`, `CON` | Negative = Нет
- **Physical skills** (Физические): Primary = `DEX` | Secondary = `STR`, `CON` | Negative = `SIZ`

> For example, your character has the following characteristics: STR 14, CON 13, INT 8, SIZ 12, POW 10, DEX 12, and CHA 8. Their skill category bonuses are:
> - Combat: +3% (+2 for DEX, +2 for STR, –1 for INT)
> - Communication: –3% (–2 for INT, 0 for POW, –1 for CHA)
> - Manipulation: +3% (+2 for DEX, –1 for INT, +2 for STR)
> - Mental: –2% (–2 from INT, 0 for POW, EDU is not used in this campaign)
> - Perception: –1% (–2 for INT, 0 for POW, +1 for CON)
> - Physical: +3% (+2 for DEX, +2 for STR, +1 for CON, –2 for SIZ)

### Архитектура Rust
Целочисленная арифметика в Rust со знаковыми типами (`i16` или `i32`) идеально подходит для вычисления вторичного бонуса, так как деление на 2 автоматически отбрасывает остаток в сторону нуля (truncation), что полностью соответствует правилу "за каждые *полные* 2 очка" как для плюса, так и для минуса.

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
    pub str: u8,
    pub con: u8,
    pub siz: u8,
    pub int: u8,
    pub pow: u8,
    pub dex: u8,
    pub cha: u8,
    pub edu: Option<u8>,
}

impl Characteristics {
    // Вспомогательные математические функции
    fn primary_mod(stat: u8) -> i16 {
        stat as i16 - 10
    }

    fn secondary_mod(stat: u8) -> i16 {
        (stat as i16 - 10) / 2
    }

    fn negative_mod(stat: u8) -> i16 {
        -(stat as i16 - 10)
    }

    // Расчет бонуса для конкретной категории
    pub fn get_category_bonus(&self, category: &SkillCategory) -> i16 {
        match category {
            SkillCategory::Combat => {
                Self::primary_mod(self.dex) 
                + Self::secondary_mod(self.int) 
                + Self::secondary_mod(self.str)
            }
            SkillCategory::Communication => {
                Self::primary_mod(self.int) 
                + Self::secondary_mod(self.pow) 
                + Self::secondary_mod(self.cha)
            }
            SkillCategory::Manipulation => {
                Self::primary_mod(self.dex) 
                + Self::secondary_mod(self.int) 
                + Self::secondary_mod(self.str)
            }
            SkillCategory::Mental => {
                let mut base = Self::primary_mod(self.int) + Self::secondary_mod(self.pow);
                if let Some(edu_val) = self.edu {
                    base += Self::secondary_mod(edu_val);
                }
                base
            }
            SkillCategory::Perception => {
                Self::primary_mod(self.int) 
                + Self::secondary_mod(self.pow) 
                + Self::secondary_mod(self.con)
            }
            SkillCategory::Physical => {
                Self::primary_mod(self.dex) 
                + Self::secondary_mod(self.str) 
                + Self::secondary_mod(self.con) 
                + Self::negative_mod(self.siz)
            }
        }
    }
}
```

### Граничные случаи и Критические исходы
- Если характеристика равна ровно `10`, модификатор от нее (как первичный, так вторичный и негативный) равен `0`.
- Для значения `9` вторичный модификатор равен `0` (так как разница `-1` поделенная на `2` в целочисленной логике равна `0`), а для значения `8` он равен `-1`. 
- Результат вычисления бонуса категории может быть отрицательным (штраф). В этом случае базовый шанс навыка уменьшается, но не может опуститься ниже `0%` при финальном расчете значения навыка.

## Опциональное правило: Упрощенные бонусы навыков (Simpler Skill Bonuses)
Альтернативный, математически более легкий подход к вычислению бонусов для категорий навыков, заменяющий сложную систему из предыдущего раздела.

### Механика / Концепция
Если базовая система вычисления `Skill Category Bonuses` (учитывающая первичные, вторичные и негативные характеристики) кажется слишком перегруженной, `Gamemaster` (Гейммастер) может использовать упрощенную формулу. 

В этой версии бонус каждой категории равен **половине от одной конкретной характеристики**, с округлением вверх (`ceil`).
        
Особенности упрощенного метода:
- Он всегда дает только положительные бонусы.
- Полностью исключает возможность получения штрафов (категорийных пенальти), даже если характеристики персонажа очень низкие.
- Обычно приводит к более высоким стартовым значениям навыков на этапе создания персонажа.

**Таблица привязки характеристик (Simple Skill Category Modifiers):**
- **Combat skills** (Боевые навыки): `DEX / 2`
- **Communication skills** (Социальные навыки): `CHA / 2`
- **Manipulation skills** (Навыки манипуляции): `DEX / 2`
- **Mental skills** (Ментальные навыки): `INT / 2`
- **Perception skills** (Навыки восприятия): `POW / 2`
- **Physical skills** (Физические навыки): `STR / 2`

*Важное отличие:* Обратите внимание, что набор базовых характеристик для некоторых категорий в этой упрощенной модели отличается от тех, что используются в сложной. Например, для `Communication` здесь используется `CHA` (вместо `INT`), а для `Perception` используется `POW` (вместо `INT`).

### Архитектура Rust
Поскольку формула всегда возвращает положительное число, тип возвращаемого значения может быть `u16` (в отличие от `i16` в сложной версии, где могли быть штрафы).

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
    pub str: u8,
    pub con: u8, // Не влияет на навыки в упрощенной модели
    pub siz: u8, // Не влияет на навыки в упрощенной модели
    pub int: u8,
    pub pow: u8,
    pub dex: u8,
    pub cha: u8,
}

impl Characteristics {
    // Функция расчета бонуса по упрощенной формуле
    pub fn get_simple_category_bonus(&self, category: &SkillCategory) -> u16 {
        let primary_stat = match category {
            SkillCategory::Combat => self.dex,
            SkillCategory::Communication => self.cha,
            SkillCategory::Manipulation => self.dex,
            SkillCategory::Mental => self.int,
            SkillCategory::Perception => self.pow,
            SkillCategory::Physical => self.str,
        };
        
        // Делим на 2 и округляем вверх
        (primary_stat as f32 / 2.0).ceil() as u16
    }
}
```

### Граничные случаи и Критические исходы
В отличие от стандартной системы, где бонус мог уйти в минус (если характеристики падали ниже 10), здесь бонус всегда строго больше нуля. Минимально возможное значение характеристики по базовым правилам (после травм или штрафов) равно `1`, что в упрощенной модели все равно даст бонус `+1%` (`1 / 2.0 = 0.5 -> ceil = 1`). Для обычного созданного персонажа с минимальным статом `3`, минимальный бонус составит `+2%`.