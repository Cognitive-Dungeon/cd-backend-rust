# Оглавление файла
- [[#Последние штрихи (Finishing Touches)]]
- [[#Опциональное правило: Культура и Персонажи (Culture and Characters)]]
- [[#Создание новых профессий (Creating New Professions)]]

---

## Последние штрихи (Finishing Touches)
Финальный этап создания персонажа (после распределения всех очков навыков и выбора сверхъестественных сил, если они предусмотрены правилами). Этот этап посвящен не поддающимся количественной оценке элементам опыта и идентичности.

### Механика / Концепция
Игрок заполняет оставшиеся пустые поля на листе персонажа, определяя предысторию и мотивы. Система предлагает ответить на ряд вопросов, чтобы найти "голос" персонажа:
- Какие трагедии и успехи он пережил?
- Где он вырос?
- Кто оказал наибольшее влияние на его жизнь?
- Кто его семья и живы ли родители?
- Влюблен ли он? Был ли или состоит ли в браке? Есть ли дети?
- Счастлив ли он в жизни?
- Много ли у него друзей? А как насчет врагов?
- Чего он хочет достичь?

Не обязательно глубоко прописывать характер до начала игры — многие черты могут сформироваться естественным образом во время приключения.

### Архитектура Rust
Для разработчика этот блок представляет собой набор текстовых (String) полей в объекте персонажа, которые могут использоваться для UI (карточки персонажа), но не участвуют в расчетах игровой логики.

```rust
pub struct CharacterBackground {
    pub tragedies_and_successes: Option<String>,
    pub origin_location: Option<String>,
    pub influential_person: Option<String>,
    pub family_status: Option<String>,
    pub romantic_status: Option<String>,
    pub happiness_level: Option<String>,
    pub friends_and_enemies: Option<String>,
    pub goals: Option<String>,
}
```

---

## Опциональное правило: Культура и Персонажи (Culture and Characters)
Для исторических или фэнтезийных сеттингов Гейммастер может вводить шаблоны культур или рас (например, эльфы, жители конкретной планеты или нации), которые влияют на стартовые параметры персонажа.

### Механика / Концепция
Культурное происхождение (Cultural background) не является обязательным, но может добавлять ценные элементы в сеттинг. Описание культуры обычно содержит следующие атрибуты:
- **Leader (Лидер)**: Правитель, президент или орган управления регионом.
- **Culture (Культура)**: Доминирующая раса или вид.
- **Appearance (Внешность)**: Общие отличительные черты (Distinctive features), такие как цвет кожи или черты лица.
- **Demeanour (Поведение)**: Общие черты характера (Personality traits), если применимо.
- **Language(s) (Языки)**: Языки, на которых говорят коренные жители.
- **Occupations (Профессии)**: Наиболее распространенные профессии в регионе (обычно выделяют 3 основных).
- **Religions (Религии)**: Какие боги почитаются (влияет на систему `Allegiance`).
- **Arms and Armour (Оружие и Броня)**: Типичные стили вооружения для данной культуры.
- **Cultural Skills (Культурные навыки)**: Навыки, которым обучают всех членов общества в процессе социализации. По усмотрению Гейммастера, эти навыки получают небольшой бонус к стартовому значению. Рекомендуется выдавать всем культурам бонус одинакового размера, чтобы сохранить баланс.
- **Items (Предметы)**: Значимая вещь, общая для выходцев из этой культуры (религиозная или социальная реликвия, не обязательно дорогая).

### Архитектура Rust
Шаблон культуры можно реализовать как модификатор, применяемый к персонажу на этапе создания (до или после выбора профессии).

```rust
pub struct CulturalTemplate {
    pub name: String,
    pub leader_description: String,
    pub dominant_species: String,
    pub typical_appearance: Vec<String>,
    pub typical_demeanour: Vec<String>,
    pub native_languages: Vec<String>,
    pub common_professions: Vec<String>,
    pub religions: Vec<String>,
    pub typical_equipment: Vec<String>,
    pub cultural_skill_bonuses: std::collections::HashMap<String, u16>, // Навык -> Бонус (%)
    pub cultural_item: Option<String>,
}

impl CharacterBuilder {
    pub fn apply_cultural_template(&mut self, culture: &CulturalTemplate) {
        // Применяем бонусы к навыкам (добавляются к Base Chance)
        for (skill_name, bonus) in &culture.cultural_skill_bonuses {
            self.add_skill_bonus(skill_name, *bonus);
        }
        
        // Записываем языки и предметы в предысторию/инвентарь
        if let Some(item) = &culture.cultural_item {
            self.inventory.push(item.clone());
        }
    }
}
```

---

## Создание новых профессий (Creating New Professions)
Инструкция для Гейммастера по внедрению пользовательских профессий.

### Механика / Концепция
Если список из базовых профессий не содержит нужной, ее можно создать двумя путями:

**1. Модификация существующей (Adaptation)**:
Находится наиболее близкая по смыслу профессия, и в ней заменяется несколько навыков.
> For example, you decide that ‘taxi driver’ is a new profession. Your gamemaster recognizes that this is basically a land-bound version of the Pilot profession. From here, you and your gamemaster choose to modify your character’s wealth level from Poor to Average, and amend the skills list to: Bargain, Drive (Automobile), Knowledge (Accounting), Knowledge (Region: the City), Listen, Navigate, Language (Other), Repair (Mechanical), and Spot. Voila, a taxi driver profession!

**2. Создание с нуля (Creation)**:
Требуется определить следующие параметры:
- **Title (Название)**: Имя профессии.
- **Description (Описание)**: Чем занимается персонаж, его обучение и положение в обществе.
- **Wealth level (Уровень богатства)**: Установить один уровень или диапазон.
- **10 Skills (10 Навыков)**: Выбрать ровно 10 навыков, которые являются обязательными или наиболее часто используемыми в этой профессии. Сюда можно включать опции "выбери один (или более) из следующих", при условии, что общее итоговое количество доступных слотов для навыков останется равно 10.

### Архитектура Rust
Функция-валидатор, которую можно использовать в административной панели или парсере конфигураций для проверки того, что созданная Гейммастером профессия соответствует правилам системы.

```rust
pub enum SkillRequirement {
    Specific(String),               // Строго заданный навык
    ChoiceFromList(u8, Vec<String>), // Выбор N навыков из списка
}

pub struct CustomProfession {
    pub title: String,
    pub description: String,
    pub wealth_range: Vec<WealthLevel>,
    pub skill_requirements: Vec<SkillRequirement>,
}

impl CustomProfession {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.wealth_range.is_empty() {
            return Err("Profession must have at least one wealth level.");
        }

        let mut total_skill_slots = 0;

        for req in &self.skill_requirements {
            match req {
                SkillRequirement::Specific(_) => total_skill_slots += 1,
                SkillRequirement::ChoiceFromList(count, list) => {
                    if *count as usize > list.len() {
                        return Err("Choice count cannot exceed list size.");
                    }
                    total_skill_slots += *count;
                }
            }
        }

        if total_skill_slots != 10 {
            return Err("A profession must provide exactly 10 skill slots.");
        }

        Ok(())
    }
}
```