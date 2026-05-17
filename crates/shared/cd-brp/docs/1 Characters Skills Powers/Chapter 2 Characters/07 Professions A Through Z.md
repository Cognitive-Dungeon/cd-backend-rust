# Оглавление файла
- [[#Общая концепция (Professions A Through Z)]]
- [[#Профессиональные навыки (Skills)]]
- [[#Особые преимущества (Special)]]
- [[#Богатство и Уровень Жизни (Wealth)]]
- [[#Архитектура данных профессий (Справочник)]]
- [[#Artist (Художник / Творец)]]
- [[#Assassin (Убийца)]]
- [[#Athlete (Атлет)]]
- [[#Beggar (Нищий)]]
- [[#Clerk (Клерк / Офисный работник)]]
- [[#Computer Tech (Компьютерный техник / Хакер)]]
- [[#Crafter (Ремесленник)]]
- [[#Criminal (Преступник)]]
- [[#Detective (Детектив)]]
- [[#Doctor (Врач)]]
- [[#Engineer (Инженер / Механик)]]
- [[#Entertainer (Артист / Исполнитель)]]
- [[#Explorer (Исследователь)]]
- [[#Farmer (Фермер)]]
- [[#Gambler (Азартный игрок / Шулер)]]
- [[#Herder (Пастух / Скотовод)]]
- [[#Hunter (Охотник)]]
- [[#Journalist (Журналист)]]
- [[#Labourer (Разнорабочий / Рабочий)]]
- [[#Lawkeeper (Страж порядка / Полицейский)]]
- [[#Lawyer (Юрист / Адвокат)]]
- [[#Mechanic (Механик)]]
- [[#Merchant (Торговец / Купец)]]
- [[#Noble (Дворянин / Аристократ)]]
- [[#Occultist (Оккультист / Эзотерик)]]
- [[#Pilot (Пилот / Капитан)]]
- [[#Politician (Политик / Чиновник)]]
- [[#Priest (Священник / Жрец)]]
- [[#Sailor (Моряк)]]
- [[#Scholar (Ученый / Исследователь)]]
- [[#Scientist (Ученый-естествоиспытатель)]]
- [[#Servant (Слуга / Помощник)]]
- [[#Shaman (Шаман)]]
- [[#Slave (Раб / Пленник)]]
- [[#Soldier (Солдат)]]
- [[#Spy (Шпион / Агент)]]
- [[#Student (Студент / Ученик)]]
- [[#Teacher (Учитель / Инструктор)]]
- [[#Technician (Техник / Системный инженер)]]
- [[#Thief (Вор / Грабитель)]]
- [[#Tribesperson (Племенной житель / Дикарь)]]
- [[#Warrior (Воин / Мастер боя)]]
- [[#Wizard (Маг / Волшебник)]]
- [[#Writer (Писатель / Сценарист)]]
- [[#Создание новых профессий (Creating New Professions)]]


---

## Общая концепция (Professions A Through Z)
Основы использования профессий для формирования стартового шаблона персонажа.

### Механика / Концепция
Представленные в системе профессии (`Professions`) являются обобщенными и подходят для множества сеттингов и эпох. Они не являются исчерпывающими: `Gamemaster` (Гейммастер) может создавать новые или адаптировать существующие. 

Гейммастер может ограничивать выбор профессий или запрашивать случайную генерацию, однако игроку обычно позволяется выбрать желаемую профессию. Главное правило: профессия определяет лишь то, с чем персонаж **начинает игру**, она не является жестким классом и не ограничивает то, чему персонаж может научиться в будущем.

### Архитектура Rust
```rust
// Идентификатор профессии, используемый для поиска в базе данных/конфиге
pub type ProfessionId = String;

pub struct ProfessionTemplate {
    pub id: ProfessionId,
    pub name: String,
    pub description: String,
    pub allowed_skills: Vec<String>,
    pub base_wealth_range: Vec<WealthLevel>,
    pub has_powers: bool,
}
```

---

## Профессиональные навыки (Skills)
Список навыков, ассоциированных с конкретной профессией.

### Механика / Концепция
Каждая профессия предоставляет список основных навыков (`Skills`), на которые персонаж будет тратить очки из своего пула профессиональных навыков (Professional Skill Pool). Персонажу не обязательно прокачивать все навыки из этого списка, он служит как "меню" доступных опций.

**Замена навыков (Substitution):**
С разрешения Гейммастера игрок может заменить любой навык из списка профессии на другой, если оригинальный навык не вписывается в концепт персонажа, сеттинг или историческую эпоху.

> For example, you wish to play a constable with a penchant for deduction in a campaign set in 12th century England. You look at the professions list and see that the Detective template lists the Firearms (Pistol or Revolver) skill. It would be wholly reasonable to switch this with Melee Weapon (Sword) for that setting.

### Архитектура Rust
Система должна поддерживать гибкое переопределение списка навыков на этапе билдера персонажа.

```rust
pub struct CharacterProfessionBuilder {
    pub base_template: ProfessionTemplate,
    pub custom_skill_substitutions: std::collections::HashMap<String, String>, // Старый навык -> Новый навык
}

impl CharacterProfessionBuilder {
    pub fn get_final_skill_list(&self) -> Vec<String> {
        self.base_template.allowed_skills.iter().map(|skill| {
            self.custom_skill_substitutions
                .get(skill)
                .unwrap_or(skill)
                .clone()
        }).collect()
    }
}
```

---

## Особые преимущества (Special)
Индикатор наличия сверхъестественных способностей.

### Механика / Концепция
Если в описании профессии присутствует тег `Special`, это означает, что представитель данной профессии, скорее всего, имеет доступ к силам (`Powers`), таким как магия, псионика или суперспособности (описывается в [[Chapter 4: Powers]]).

---

## Богатство и Уровень Жизни (Wealth)
Экономический статус персонажа на старте игры.

### Механика / Концепция
`Wealth` (Уровень богатства) представляет экономический статус персонажа на момент начала игры и то, к какому уровню жизни он привык. Это определяет, каким стартовым имуществом (`Possessions`) он владеет. 

Для некоторых профессий указан не один уровень, а диапазон. Гейммастер и игрок могут выбрать подходящий уровень в зависимости от концепта.
**Альтернативный метод:** Игрок начинает с самого низкого уровня богатства в указанном диапазоне и повышает его на одну ступень за каждый успешный бросок навыка `Status` (выполняется после завершения создания персонажа).

**Градация уровней богатства:**
- **Destitute (Нищий)**: Нет денег, полагается на сбор отбросов или благотворительность. Нет дома, спит где придется. Имеет только те немногочисленные и малоценные вещи, которые может унести на себе. Часто страдает от социальных предрассудков.
- **Poor (Бедный)**: Есть немного денег, не голодает и имеет крышу над головой (скромное жилье в бедном районе), но живет без роскоши и не имеет свободных наличных. Обычно частично занят или тяжело работает за гроши.
- **Average (Средний)**: Комфортный доход без серьезных трудностей, но крупные покупки требуют тщательного планирования. Среднее жилье, есть сбережения. Позволяет себе редкие предметы роскоши. Средний класс.
- **Affluent (Состоятельный)**: Значительный доход, позволяющий жить роскошно. Элитное жилье, нет необходимости дважды думать перед крупными покупками. Жизнь в избытке без негативных финансовых последствий.
- **Wealthy (Богатый)**: Огромное материальное состояние из почти неисчерпаемого источника. Покупки колоссальной стоимости совершаются не задумываясь. Высший уровень жизни, предоставляющий недоступные другим социальные и деловые возможности.

### Архитектура Rust
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WealthLevel {
    Destitute,
    Poor,
    Average,
    Affluent,
    Wealthy,
}

impl WealthLevel {
    // Вспомогательная функция для альтернативного метода генерации
    pub fn upgrade_tier(&self) -> Self {
        match self {
            WealthLevel::Destitute => WealthLevel::Poor,
            WealthLevel::Poor => WealthLevel::Average,
            WealthLevel::Average => WealthLevel::Affluent,
            WealthLevel::Affluent => WealthLevel::Wealthy,
            WealthLevel::Wealthy => WealthLevel::Wealthy, // Достигнут максимум
        }
    }
}

pub fn resolve_starting_wealth(
    base_range: &[WealthLevel], 
    status_successes: u8
) -> WealthLevel {
    if base_range.is_empty() {
        return WealthLevel::Average; // Фолбэк по умолчанию
    }
    
    // Начинаем с минимального значения в доступном диапазоне
    let mut current_wealth = *base_range.iter().min().unwrap();
    let max_possible = *base_range.iter().max().unwrap();

    // Повышаем за каждый успех навыка Status, но не выше максимума диапазона
    for _ in 0..status_successes {
        if current_wealth < max_possible {
            current_wealth = current_wealth.upgrade_tier();
        }
    }
    
    current_wealth
}
```


## Архитектура данных профессий (Справочник)
Поскольку все профессии используют общую логику выбора навыков (фиксированные навыки, выбор из категории или выбор $N$ из предложенного списка), для всех последующих профессий будет применяться следующая базовая структура данных в блоках Rust:

### Архитектура Rust
```rust
pub enum WealthLevel { Destitute, Poor, Average, Affluent, Wealthy, Any }

pub enum SkillReq {
    /// Конкретный навык, например "Dodge"
    Specific(String),
    /// Выбор N специализаций из указанной категории (например, 2 навыка "Art")
    AnyOfCategory(String, u8),
    /// Выбор N навыков из жестко заданного списка
    ChooseFrom(u8, Vec<String>),
    /// Выбор ровно одного навыка из вариантов (например, "Drive" ИЛИ "Ride")
    OneOf(Vec<String>),
}

pub struct ProfessionTemplate {
    pub id: String,
    pub description: String,
    pub wealth_range: Vec<WealthLevel>,
    pub skills: Vec<SkillReq>,
}
```

---

## Artist (Художник / Творец)
Люди, зарабатывающие на жизнь искусством (рисование, скульптура, дизайн, фотография и т.д.). Исполнители (актеры/музыканты) относятся к другой профессии (`Entertainer`).

### Механика / Концепция
- **Богатство (Wealth)**: Любое, но обычно `Poor` (Бедный) или `Average` (Средний).
- **Навыки (Skills)**: Требует выбора специализаций для искусств и ремесел, а также соответствующих знаний.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "artist".into(),
    description: "Makes a living through creating art in physical or digital mediums.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average, WealthLevel::Any],
    skills: vec![
        SkillReq::AnyOfCategory("Art".into(), 2),
        SkillReq::AnyOfCategory("Craft".into(), 1),
        SkillReq::Specific("Insight".into()),
        SkillReq::AnyOfCategory("Knowledge".into(), 1),
        SkillReq::Specific("Language (Other)".into()),
        SkillReq::Specific("Language (Own)".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::Specific("Spot".into()),
    ],
}
```

---

## Assassin (Убийца)
Хладнокровный профессионал, специализирующийся на скрытном устранении целей.

### Механика / Концепция
- **Богатство (Wealth)**: `Average` (Средний) или `Affluent` (Состоятельный). Может владеть широким арсеналом оружия и фальшивых личностей.
- **Навыки (Skills)**: Смесь обязательных навыков скрытности и восприятия, а также большой пул опциональных боевых и транспортных навыков, из которых нужно выбрать 5.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "assassin".into(),
    description: "Professional killer skilled in termination, usually in secrecy.".into(),
    wealth_range: vec![WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::Specific("Dodge".into()),
        SkillReq::Specific("Hide".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::Specific("Spot".into()),
        SkillReq::Specific("Stealth".into()),
        SkillReq::ChooseFrom(5, vec![
            "Brawl".into(), "Disguise".into(), "Drive".into(), 
            "Electronics".into(), "Grapple".into(), "Firearm (any)".into(), 
            "Fine Manipulation".into(), "Martial Arts".into(), 
            "Melee Weapon (any)".into(), "Missile Weapon (any)".into(), 
            "Ride".into(), "Throw".into(), "Track".into()
        ]),
    ],
}
```

---

## Athlete (Атлет)
Профессиональный спортсмен или любитель, чье тело натренировано для соревнований.

### Механика / Концепция
- **Богатство (Wealth)**: От `Poor` (Бедный) до `Wealthy` (Богатый), но обычно `Average` (Средний) или `Affluent` (Состоятельный).
- **Навыки (Skills)**: Обязательные базовые физические навыки и выбор из 5 дополнительных боевых или соревновательных навыков в зависимости от вида спорта.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "athlete".into(),
    description: "Excels in sports or exercise, honing body for competition.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average, WealthLevel::Affluent, WealthLevel::Wealthy],
    skills: vec![
        SkillReq::Specific("Climb".into()),
        SkillReq::Specific("Dodge".into()),
        SkillReq::Specific("Jump".into()),
        SkillReq::Specific("Stealth".into()),
        SkillReq::Specific("Throw".into()),
        SkillReq::ChooseFrom(5, vec![
            "Brawl".into(), "First Aid".into(), "Grapple".into(), 
            "Insight".into(), "Listen".into(), "Martial Arts".into(), 
            "Spot".into(), "Ride".into(), "Swim".into()
        ]),
    ],
}
```

---

## Beggar (Нищий)
Бродяга, выживающий за счет подаяний и сбора еды.

### Механика / Концепция
- **Богатство (Wealth)**: `Destitute` (Нищий). Некоторые могут притворяться и фактически иметь статус `Poor` (Бедный).
- **Навыки (Skills)**: Жестко заданный список навыков уличного выживания, социальной манипуляции и воровства. Включает специализированный `Knowledge` локального региона.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "beggar".into(),
    description: "Survives by begging for money and necessities, or wandering.".into(),
    wealth_range: vec![WealthLevel::Destitute, WealthLevel::Poor],
    skills: vec![
        SkillReq::Specific("Bargain".into()),
        SkillReq::Specific("Fast Talk".into()),
        SkillReq::Specific("Hide".into()),
        SkillReq::Specific("Insight".into()),
        SkillReq::Specific("Knowledge (Region: local area)".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::Specific("Sleight of Hand".into()),
        SkillReq::Specific("Spot".into()),
        SkillReq::Specific("Stealth".into()),
    ],
}
```

---

## Clerk (Клерк / Офисный работник)
Сотрудник за столом: бухгалтер, чиновник, банковский служащий.

### Механика / Концепция
- **Богатство (Wealth)**: `Average` (Средний).
- **Навыки (Skills)**: Знания в области права и финансов, бюрократические навыки. Выбор между `Computer Use` (для современных сеттингов) и `Literacy` (Грамотность, для исторических, где не все умеют читать).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "clerk".into(),
    description: "Desk worker, accountant, or salaried employee dealing with finances/records.".into(),
    wealth_range: vec![WealthLevel::Average],
    skills: vec![
        SkillReq::Specific("Bargain".into()),
        SkillReq::Specific("Etiquette".into()),
        SkillReq::Specific("Knowledge (Accounting)".into()),
        SkillReq::Specific("Knowledge (Law)".into()),
        SkillReq::AnyOfCategory("Knowledge".into(), 1),
        SkillReq::Specific("Language (Own)".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::Specific("Status".into()),
        SkillReq::OneOf(vec!["Technical Skill (Computer Use)".into(), "Literacy".into()]),
    ],
}
```

---

## Computer Tech (Компьютерный техник / Хакер)
Разработчик ПО, инженер корпорации или нелегальный хакер.

### Механика / Концепция
- **Богатство (Wealth)**: От `Average` (Средний) до `Affluent` (Состоятельный).
- **Навыки (Skills)**: Жесткая техническая база и языки программирования (отражаются через `Language (Other)`). Плюс 1 навык на выбор, определяющий легальность (Бухгалтерия, Право) или нелегальность (Скрытность/Убежище - `Hide`) деятельности.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "computer_tech".into(),
    description: "Software engineer, hacker, or IT specialist.".into(),
    wealth_range: vec![WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::Specific("Craft (Computer Hardware or Code)".into()),
        SkillReq::AnyOfCategory("Knowledge".into(), 1),
        SkillReq::Specific("Language (Other) [Programming Language]".into()),
        SkillReq::Specific("Repair (Electrical)".into()),
        SkillReq::Specific("Repair (Electronics)".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::Specific("Science (Mathematics)".into()),
        SkillReq::Specific("Status".into()),
        SkillReq::Specific("Technical (Computer Use)".into()),
        SkillReq::ChooseFrom(1, vec![
            "Knowledge (Accounting)".into(), 
            "Hide".into(), 
            "Knowledge (Law)".into()
        ]),
    ],
}
```

---

## Crafter (Ремесленник)
Кузнец, стеклодув, механик. Человек, создающий товары вручную на продажу.

### Механика / Концепция
- **Богатство (Wealth)**: От `Poor` (Бедный) до `Affluent` (Состоятельный), обычно `Average` (Средний).
- **Навыки (Skills)**: Фокус на ремеслах (`Craft`) и искусствах (`Art`). Выбор из 2 дополнительных технических специальностей, подходящих под тип ремесла.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "crafter".into(),
    description: "Maker of trade goods by hand (blacksmith, glass-blower, watchmaker, etc.).".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::Specific("Appraise".into()),
        SkillReq::AnyOfCategory("Art".into(), 1),
        SkillReq::Specific("Bargain".into()),
        SkillReq::AnyOfCategory("Craft".into(), 2),
        SkillReq::Specific("Spot".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::Specific("Status".into()),
        SkillReq::ChooseFrom(2, vec![
            "Fine Manipulation".into(), "Heavy Machine".into(), 
            "Repair (Electrical)".into(), "Repair (Electronics)".into(), 
            "Repair (Mechanical)".into()
        ]),
    ],
}
```

---

## Criminal (Преступник)
Нарушитель закона: от карманника до участника организованной преступности.

### Механика / Концепция
- **Богатство (Wealth)**: От `Poor` (Бедный) до `Affluent` (Состоятельный), иногда `Wealthy` (Богатый), обычно `Average` (Средний).
- **Навыки (Skills)**: Требует выбора транспортного навыка (`Drive` или `Ride`) и предоставляет большой список из 6 навыков на выбор в зависимости от криминальной специализации (рэкет, кражи, мошенничество).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "criminal".into(),
    description: "Makes way through the world by breaking the law (theft, organized crime, etc.).".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average, WealthLevel::Affluent, WealthLevel::Wealthy],
    skills: vec![
        SkillReq::Specific("Bargain".into()),
        SkillReq::Specific("Hide".into()),
        SkillReq::Specific("Stealth".into()),
        SkillReq::OneOf(vec!["Drive".into(), "Ride".into()]),
        SkillReq::ChooseFrom(6, vec![
            "Appraise".into(), "Brawl".into(), "Climb".into(), 
            "Fast Talk".into(), "Fine Manipulation".into(), 
            "Firearm (any)".into(), "Gaming".into(), "Grapple".into(), 
            "Insight".into(), "Jump".into(), "Knowledge (Law)".into(), 
            "Listen".into(), "Martial Arts".into(), 
            "Melee Weapon (any)".into(), "Persuade".into(), 
            "Spot".into(), "Throw".into()
        ]),
    ],
}
```

---

## Detective (Детектив)
Полицейский или частный сыщик. Работает на основе наблюдений, дедукции и криминалистики.

### Механика / Концепция
- **Богатство (Wealth)**: `Average` (Средний) или `Affluent` (Состоятельный).
- **Навыки (Skills)**: Огнестрельное оружие (`Handgun`), знания законов и следственные навыки по умолчанию. Плюс выбор 4 навыков из широкого спектра для определения профиля (судмедэксперт, оперативник, следователь).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "detective".into(),
    description: "Employed by police or working privately, uses observation and deduction.".into(),
    wealth_range: vec![WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::Specific("Firearm (Handgun)".into()),
        SkillReq::Specific("Knowledge (Law)".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::Specific("Spot".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::ChooseFrom(4, vec![
            "Art".into(), "Brawl".into(), "Disguise".into(), 
            "Dodge".into(), "Drive".into(), "Fast Talk".into(), 
            "Firearm (any)".into(), "Grapple".into(), "Hide".into(), 
            "Insight".into(), "Knowledge (any)".into(), 
            "Language (Other)".into(), "Language (Own)".into(), 
            "Medicine".into(), "Ride".into(), "Science (any)".into(), 
            "Technical (Computer Use)".into(), "Stealth".into(), "Track".into()
        ]),
    ],
}
```

---

## Doctor (Врач)
Медицинский работник, чье призвание — лечить травмы и болезни с помощью длительного обучения и диагностических навыков.

### Механика / Концепция
- **Богатство (Wealth)**: `Average` (Средний) до `Affluent` (Состоятельный).
- **Навыки (Skills)**: Базируется на первой помощи (`First Aid`) и медицине (`Medicine`). Для работы с терминами или иностранным персоналом требуется `Language (Other)`. Включает выбор из 4 академических или психологических навыков.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "doctor".into(),
    description: "Treats the injured, infirm, sick, and unhealthy using diagnostic skills and medicine.".into(),
    wealth_range: vec![WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::Specific("First Aid".into()),
        SkillReq::Specific("Language (Other)".into()),
        SkillReq::Specific("Medicine".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::Specific("Spot".into()),
        SkillReq::ChooseFrom(4, vec![
            "Insight".into(), "Language (any)".into(), 
            "Psychotherapy".into(), "Science (any)".into(), "Status".into()
        ]),
    ],
}
```

## Engineer (Инженер / Механик)
Специалист, который строит, ремонтирует или обслуживает механизмы, от простой каменной архитектуры до продвинутых двигателей космических кораблей или осадных орудий.

### Механика / Концепция
- **Богатство (Wealth)**: От `Poor` (Бедный) до `Affluent` (Состоятельный), обычно `Average` (Средний).
- **Навыки (Skills)**: Обязательные профильные навыки ремонта (`Mechanical`, `Structural`). Выбор из 5 дополнительных навыков для специализации (например, черчение (`Art`), управление тяжелой техникой (`Heavy Machine`) или электроника).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "engineer".into(),
    description: "Builds, repairs, or maintains machines, architecture, or complex vehicles.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::AnyOfCategory("Craft".into(), 1),
        SkillReq::Specific("Repair (Mechanical)".into()),
        SkillReq::Specific("Repair (Structural)".into()),
        SkillReq::Specific("Spot".into()),
        SkillReq::Specific("Status".into()),
        SkillReq::ChooseFrom(5, vec![
            "Art (Drafting)".into(), "Drive".into(), "Heavy Machine".into(),
            "Knowledge (any)".into(), "Pilot (any)".into(),
            "Repair (Electrical)".into(), "Repair (Electronics)".into(),
            "Science (any)".into(), "Technical (Computer Use)".into()
        ]),
    ],
}
```

---

## Entertainer (Артист / Исполнитель)
Человек, использующий свой исполнительский талант для развлечения аудитории (актер, певец, танцор). В отличие от `Artist`, который создает статичные произведения, `Entertainer` работает с публикой вживую или через трансляции.

### Механика / Концепция
- **Богатство (Wealth)**: От `Destitute` (Нищий) до `Wealthy` (Богатый), обычно `Average` (Средний).
- **Навыки (Skills)**: Жестко заданный список навыков без возможности выбора. Фокус на социальных взаимодействиях, исполнительском мастерстве (`Perform`) и внимании. Обратите внимание, что навык `Language (Other)` берется дважды (видимо, для имитации исполнения на разных языках).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "entertainer".into(),
    description: "Uses performing talent to entertain audiences, improvisational or scripted.".into(),
    wealth_range: vec![WealthLevel::Destitute, WealthLevel::Poor, WealthLevel::Average, WealthLevel::Affluent, WealthLevel::Wealthy],
    skills: vec![
        SkillReq::AnyOfCategory("Art".into(), 1),
        SkillReq::Specific("Disguise".into()),
        SkillReq::Specific("Fast Talk".into()),
        SkillReq::Specific("Fine Manipulation".into()),
        SkillReq::Specific("Insight".into()),
        SkillReq::Specific("Language (Other)".into()),
        SkillReq::Specific("Language (Other)".into()), // По правилам берется второй раз
        SkillReq::Specific("Listen".into()),
        SkillReq::AnyOfCategory("Perform".into(), 1),
        SkillReq::Specific("Persuade".into()),
    ],
}
```
### Граничные случаи и Критические исходы
Навык `Language (Other)` указан в списке дважды. Для разработчика это означает, что игрок должен выбрать две *разные* специальности для этого навыка (например, Испанский и Французский), либо вложить очки в один и тот же язык дважды, если система генератора не объединяет одинаковые записи на этапе выбора.

---

## Explorer (Исследователь)
Человек, чья цель жизни — искать неизведанные уголки мира, приносить знания во имя славы или науки.

### Механика / Концепция
- **Богатство (Wealth)**: `Affluent` (Состоятельный) или `Wealthy` (Богатый) — экспедиции требуют спонсирования или личного капитала.
- **Навыки (Skills)**: База из языков, исследований и выживания. Плюс выбор 4 навыков, определяющих среду исследования (академические знания, пилотирование, выживание в дикой природе).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "explorer".into(),
    description: "Seeks out the unknown corners of the world to bring back knowledge.".into(),
    wealth_range: vec![WealthLevel::Affluent, WealthLevel::Wealthy],
    skills: vec![
        SkillReq::Specific("Climb".into()),
        SkillReq::Specific("Language (Other)".into()),
        SkillReq::Specific("Language (Own)".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::Specific("Spot".into()),
        SkillReq::ChooseFrom(4, vec![
            "Knowledge (Anthropology)".into(), "Knowledge (Group)".into(),
            "Knowledge (History)".into(), "Knowledge (Natural World)".into(),
            "Knowledge (Region)".into(), "Drive".into(), "Fast Talk".into(),
            "Firearm (Pistol)".into(), "Firearm (Revolver)".into(), "Firearm (Rifle)".into(),
            "Navigate".into(), "Pilot (Aircraft)".into(), "Pilot (Boat)".into(),
            "Ride".into(), "Science (Geology)".into(), "Swim".into(), "Track".into()
        ]),
    ],
}
```

---

## Farmer (Фермер)
Житель сельской или полудикой местности, зарабатывающий на жизнь возделыванием земли или уходом за скотом. Тяжелая физическая работа.

### Механика / Концепция
- **Богатство (Wealth)**: От `Poor` (Бедный) до `Affluent` (Состоятельный), обычно `Average` (Средний).
- **Навыки (Skills)**: База из торговли, знаний природы и восприятия. Плюс выбор из 5 навыков: транспорт, оружие для защиты (винтовка/дробовик) или академические знания (биология/геология).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "farmer".into(),
    description: "Dwells in a rural area, coaxing a living out of the land or herds.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::Specific("Bargain".into()),
        SkillReq::AnyOfCategory("Craft".into(), 1),
        SkillReq::Specific("Knowledge (Natural History)".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::Specific("Spot".into()),
        SkillReq::ChooseFrom(5, vec![
            "Brawl".into(), "Drive".into(), 
            "Firearm (Rifle)".into(), "Firearm (Shotgun)".into(), 
            "First Aid".into(), "Heavy Machine".into(), 
            "Knowledge (History)".into(), "Repair (Mechanical)".into(), 
            "Ride".into(), "Science (Biology)".into(), 
            "Science (Botany)".into(), "Science (Geology)".into(), 
            "Track".into()
        ]),
    ],
}
```

## Gambler (Азартный игрок / Шулер)
Человек, выживающий по воле случая или обманывающий судьбу в азартных играх. Часто путешествует с места на место, скрываясь от закона или кредиторов.

### Механика / Концепция
- **Богатство (Wealth)**: От `Poor` (Бедный) до `Affluent` (Состоятельный), обычно `Average` (Средний).
- **Навыки (Skills)**: Жесткий список из 10 навыков, сфокусированных на социальной манипуляции, играх (`Gaming`), ловкости рук (`Sleight of Hand`) и умении постоять за себя в драке (`Brawl`, `Dodge`).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "gambler".into(),
    description: "Survives by chance or cheating fate in games of luck and skill.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::Specific("Bargain".into()),
        SkillReq::Specific("Brawl".into()),
        SkillReq::Specific("Dodge".into()),
        SkillReq::Specific("Fast Talk".into()),
        SkillReq::Specific("Gaming".into()),
        SkillReq::Specific("Insight".into()),
        SkillReq::Specific("Knowledge (Accounting)".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::Specific("Sleight of Hand".into()),
        SkillReq::Specific("Spot".into()),
    ],
}
```

---

## Herder (Пастух / Скотовод)
Зарабатывает на жизнь уходом за стадами животных на открытых пастбищах, ищет отбившихся животных и отвозит их на рынок для продажи.

### Механика / Концепция
- **Богатство (Wealth)**: От `Poor` (Бедный) до `Average` (Средний), редко `Affluent` (Состоятельный).
- **Навыки (Skills)**: Фиксированный набор из 10 навыков выживания на открытом воздухе, включая верховую езду (`Ride`), использование узлов (`Craft`), навигацию и стрельбу из винтовки.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "herder".into(),
    description: "Makes a living tending herd animals and riding the open range.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::Specific("Craft (Knots)".into()), // usually knots
        SkillReq::Specific("Firearm (Rifle)".into()),
        SkillReq::Specific("Knowledge (Natural History)".into()),
        SkillReq::Specific("Knowledge (Region: the Range)".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::Specific("Navigate".into()),
        SkillReq::Specific("Ride".into()),
        SkillReq::Specific("Spot".into()),
        SkillReq::Specific("Throw".into()),
        SkillReq::Specific("Track".into()),
    ],
}
```

---

## Hunter (Охотник)
Выживальщик или спортсмен, специализирующийся на выслеживании, установке ловушек или убийстве диких животных (или иных существ).

### Механика / Концепция
- **Богатство (Wealth)**: `Poor` (Бедный) или `Average` (Средний). Если это охотник за крупной дичью или спортсмен, богатство может быть `Wealthy` (Богатый).
- **Навыки (Skills)**: 7 обязательных навыков, связанных с выживанием, маскировкой и восприятием в дикой природе. Плюс выбор 3 специфических навыков (оружие, региональные знания).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "hunter".into(),
    description: "Specializes in tracking and trapping or killing wild animals.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average, WealthLevel::Wealthy],
    skills: vec![
        SkillReq::Specific("Climb".into()),
        SkillReq::Specific("Hide".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::Specific("Navigate".into()),
        SkillReq::Specific("Spot".into()),
        SkillReq::Specific("Stealth".into()),
        SkillReq::Specific("Track".into()),
        SkillReq::ChooseFrom(3, vec![
            "Firearm (Rifle)".into(), "Firearm (Shotgun)".into(),
            "Knowledge (Natural History)".into(), "Knowledge (Region)".into(),
            "Language (Other)".into(), "Melee Weapon (Spear)".into(),
            "Missile Weapon (any)".into(), "Ride".into()
        ]),
    ],
}
```

---

## Journalist (Журналист)
Колумнист, фотожурналист, телеведущий или аналитик. Зарабатывает освещением событий. 

### Механика / Концепция
- **Богатство (Wealth)**: От `Average` (Средний) до `Affluent` (Состоятельный).
- **Навыки (Skills)**: 7 обязательных социальных и исследовательских навыков. Плюс выбор 3 навыков, отражающих медиум работы (фотография, маскировка для шпионажа, компьютерная техника).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "journalist".into(),
    description: "Makes a living from the coverage and analysis of events.".into(),
    wealth_range: vec![WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::Specific("Fast Talk".into()),
        SkillReq::Specific("Insight".into()),
        SkillReq::Specific("Language (Own)".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::Specific("Spot".into()),
        SkillReq::ChooseFrom(3, vec![
            "Art (Photography)".into(), "Craft (Photography)".into(),
            "Disguise".into(), "Hide".into(), "Knowledge (any)".into(),
            "Language (Other)".into(), "Status".into(), "Stealth".into(),
            "Technical (Computer Use)".into()
        ]),
    ],
}
```

---

## Labourer (Разнорабочий / Рабочий)
Синий воротничок (работник фабрики, грузчик на складе). Для этой профессии мышцы и способность выполнять монотонную работу важнее интеллектуальных способностей.

### Механика / Концепция
- **Богатство (Wealth)**: `Poor` (Бедный) или `Average` (Средний).
- **Навыки (Skills)**: 6 обязательных навыков, сфокусированных на физическом труде (`Climb`, `Heavy Machine`) и силе (`Brawl`, `Grapple`). Плюс выбор 4 навыков, определяющих конкретную среду работы (ремонт, мелкая моторика, компьютерная техника).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "labourer".into(),
    description: "Blue-collar worker where muscle and repetition are paramount.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average],
    skills: vec![
        SkillReq::Specific("Climb".into()),
        SkillReq::AnyOfCategory("Craft".into(), 1),
        SkillReq::Specific("Drive".into()),
        SkillReq::Specific("Brawl".into()),
        SkillReq::Specific("Grapple".into()),
        SkillReq::Specific("Heavy Machine".into()),
        SkillReq::ChooseFrom(4, vec![
            "Appraise".into(), "Fine Manipulation".into(),
            "Language (Other)".into(), "Literacy".into(),
            "Repair (Mechanical)".into(), "Repair (Structural)".into(),
            "Technical (Computer Use)".into()
        ]),
    ],
}
```

---

## Lawkeeper (Страж порядка / Полицейский)
Представитель власти, чья юрисдикция — поддерживать и защищать закон (в идеале, ради защиты обычных людей). Поддерживается силовыми структурами своего общества.

### Механика / Концепция
- **Богатство (Wealth)**: Обычно `Average` (Средний). Коррумпированные стражи порядка могут иметь уровень `Affluent` (Состоятельный).
- **Навыки (Skills)**: База из 6 обязательных навыков ближнего боя, социальных проверок и восприятия. Плюс выбор 4 навыков, отражающих специфику региона, экипировку (стрельба, вождение) или специализацию (следопытство, статус).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "lawkeeper".into(),
    description: "Has the authority and jurisdiction to uphold and defend the law.".into(),
    wealth_range: vec![WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::Specific("Brawl".into()),
        SkillReq::Specific("Dodge".into()),
        SkillReq::Specific("Fast Talk".into()),
        SkillReq::Specific("Knowledge (Law)".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::Specific("Spot".into()),
        SkillReq::ChooseFrom(4, vec![
            "Drive".into(), "Firearm (any)".into(), "First Aid".into(),
            "Grapple".into(), "Insight".into(), "Knowledge (Region)".into(),
            "Knowledge (Group)".into(), "Language (Other)".into(),
            "Martial Arts".into(), "Melee Weapon (any)".into(),
            "Missile Weapon (any)".into(), "Pilot (any)".into(),
            "Ride".into(), "Status".into(), "Technical (Computer Use)".into(),
            "Track".into()
        ]),
    ],
}
```

---

## Lawyer (Юрист / Адвокат)
Обучен законам и использует правовую систему для защиты или обвинения, либо представляет юридические интересы лиц и организаций. Не обладает властью выше обычного гражданина, но имеет значительное влияние за счет знания системы.

### Механика / Концепция
- **Богатство (Wealth)**: Любой уровень (от `Destitute` до `Wealthy`).
- **Навыки (Skills)**: Фиксированный набор из 10 навыков, полностью сосредоточенных на социальной манипуляции, знаниях законов и ораторском искусстве (`Perform (Oratory)`).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "lawyer".into(),
    description: "Trained in law, uses the legal system to prosecute or defend.".into(),
    wealth_range: vec![WealthLevel::Destitute, WealthLevel::Poor, WealthLevel::Average, WealthLevel::Affluent, WealthLevel::Wealthy],
    skills: vec![
        SkillReq::Specific("Bargain".into()),
        SkillReq::Specific("Fast Talk".into()),
        SkillReq::Specific("Insight".into()),
        SkillReq::Specific("Knowledge (Law)".into()),
        SkillReq::AnyOfCategory("Knowledge".into(), 1),
        SkillReq::Specific("Language (Own)".into()),
        SkillReq::Specific("Perform (Oratory)".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::Specific("Status".into()),
    ],
}
```

---

## Mechanic (Механик)
"Мазутная обезьяна", проводит время за обслуживанием, починкой, а иногда и сборкой машин, транспортных средств или сложных конструкций. Легко решает технические проблемы.

### Механика / Концепция
- **Богатство (Wealth)**: От `Poor` (Бедный) до `Average` (Средний), обычно `Average`.
- **Навыки (Skills)**: Жесткий список из 10 навыков. Абсолютный фокус на ремонтных навыках (все виды `Repair`), работе с инструментами (`Fine Manipulation`) и тяжелой техникой.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "mechanic".into(),
    description: "Spends time maintaining, repairing, and building machines and vehicles.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average],
    skills: vec![
        SkillReq::Specific("Bargain".into()),
        SkillReq::Specific("Craft (Metalwork)".into()),
        SkillReq::Specific("Drive".into()),
        SkillReq::Specific("Fine Manipulation".into()),
        SkillReq::Specific("Heavy Machine".into()),
        SkillReq::Specific("Repair (Electrical)".into()),
        SkillReq::Specific("Repair (Electronics)".into()),
        SkillReq::Specific("Repair (Mechanical)".into()),
        SkillReq::Specific("Repair (Structural)".into()),
        SkillReq::Specific("Spot".into()),
    ],
}
```

---

## Merchant (Торговец / Купец)
Зарабатывает розничной или оптовой торговлей. Может быть владельцем магазина, странствующим лудильщиком или международным торговым представителем.

### Механика / Концепция
- **Богатство (Wealth)**: От `Average` (Средний) до `Wealthy` (Богатый), обычно `Affluent` (Состоятельный). Гейммастер и игрок вместе решают, владеет ли персонаж собственным магазином или торговым судном.
- **Навыки (Skills)**: 8 базовых навыков торговли, оценки и социального статуса. Плюс 2 навыка на выбор, отражающих специализацию торговца (например, вождение для странствующего купца или специфические знания товара).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "merchant".into(),
    description: "Makes a living in retail or wholesale, buying for less and selling for more.".into(),
    wealth_range: vec![WealthLevel::Average, WealthLevel::Affluent, WealthLevel::Wealthy],
    skills: vec![
        SkillReq::Specific("Appraise".into()),
        SkillReq::Specific("Bargain".into()),
        SkillReq::Specific("Fast Talk".into()),
        SkillReq::Specific("Knowledge (Accounting)".into()),
        SkillReq::Specific("Knowledge (Business)".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::Specific("Status".into()),
        SkillReq::ChooseFrom(2, vec!["Any skill appropriate to setting and concept".into()]), // Обрабатывается как открытый выбор
    ],
}
```

---

## Noble (Дворянин / Аристократ)
Рожден в богатстве и правящем классе. Привык к элегантному и экстравагантному образу жизни, обладает широкими связями среди элиты.

### Механика / Концепция
- **Богатство (Wealth)**: От `Affluent` (Состоятельный) до `Wealthy` (Богатый), обычно `Wealthy`.
- **Навыки (Skills)**: 7 обязательных навыков, отражающих статус, образование и этикет. Плюс 3 любых навыка на выбор, представляющих хобби или личные интересы аристократа.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "noble".into(),
    description: "Born into wealth and a ruling class, accustomed to an extravagant lifestyle.".into(),
    wealth_range: vec![WealthLevel::Affluent, WealthLevel::Wealthy],
    skills: vec![
        SkillReq::Specific("Bargain".into()),
        SkillReq::Specific("Drive".into()),
        SkillReq::Specific("Etiquette".into()),
        SkillReq::Specific("Language (Own)".into()),
        SkillReq::Specific("Language (Other)".into()),
        SkillReq::Specific("Literacy".into()),
        SkillReq::Specific("Status".into()),
        SkillReq::ChooseFrom(3, vec!["Any skill as hobbies or fields of interest".into()]), // Открытый выбор
    ],
}
```

---

## Occultist (Оккультист / Эзотерик)
Исследователь тайных знаний, скрытых легенд и магической силы. Верит во влияние сверхъестественных сил.

### Механика / Концепция
- **Богатство (Wealth)**: От `Poor` (Бедный) до `Wealthy` (Богатый), обычно `Affluent` (Состоятельный).
- **Навыки (Skills)**: 8 фиксированных навыков, связанных с историей, антропологией, оккультизмом и исследованиями. Плюс 2 навыка на выбор (искусства, медицина, археология).
- **Особые преимущества (Special)**: Если в сеттинге есть магия, оккультист может иметь магические силы (см. [[Chapter 4: Powers]]).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "occultist".into(),
    description: "Student of obscure secrets, hidden lore, and magical power.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average, WealthLevel::Affluent, WealthLevel::Wealthy],
    skills: vec![
        SkillReq::Specific("Fast Talk".into()),
        SkillReq::Specific("Insight".into()),
        SkillReq::Specific("Knowledge (Anthropology)".into()),
        SkillReq::Specific("Knowledge (History)".into()),
        SkillReq::Specific("Knowledge (Occult)".into()),
        SkillReq::Specific("Language (Other)".into()),
        SkillReq::Specific("Language (Own)".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::ChooseFrom(2, vec![
            "Art (any)".into(), "Craft (any)".into(),
            "Knowledge (Archaeology)".into(), "Medicine".into(),
            "Science (any)".into(), "Status".into()
        ]),
    ],
    has_powers: true, // Имеет доступ к магии или колдовству
}
```

---

## Pilot (Пилот / Капитан)
Управляет транспортными средствами на земле, воде, в воздухе или в космосе. От капитана грузового судна до пилота космического истребителя.

### Механика / Концепция
- **Богатство (Wealth)**: От `Average` (Средний) до `Affluent` (Состоятельный). Если владеет собственным судном — `Affluent`.
- **Навыки (Skills)**: 6 обязательных навыков (вождение, пилотирование, навигация, восприятие). Плюс 4 навыка на выбор, отражающих специфику судна (ремонт, командование экипажем, орудийные системы, наука).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "pilot".into(),
    description: "Trained in guiding and piloting a vessel (land, water, skies, space).".into(),
    wealth_range: vec![WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::Specific("Drive".into()),
        SkillReq::Specific("Heavy Machine".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::Specific("Navigate".into()),
        SkillReq::AnyOfCategory("Pilot".into(), 1),
        SkillReq::Specific("Spot".into()),
        SkillReq::ChooseFrom(4, vec![
            "Bargain".into(), "Climb".into(), "Command".into(),
            "Craft (any)".into(), "Knowledge (Region)".into(),
            "Repair (Electrical)".into(), "Repair (Electronics)".into(),
            "Repair (Mechanical)".into(), "Language (Other)".into(),
            "Persuade".into(), "Science (Physics)".into(),
            "Science (Astronomy)".into(), "Technical (Computer Use)".into()
        ]),
    ],
    has_powers: false,
}
```

---

## Politician (Политик / Чиновник)
Избранный или назначенный представитель власти. От средневекового магистрата до галактического сенатора.

### Механика / Концепция
- **Богатство (Wealth)**: От `Affluent` (Состоятельный) до `Wealthy` (Богатый), обычно `Affluent`.
- **Навыки (Skills)**: 7 обязательных навыков социальной манипуляции, статуса и этикета. Плюс выбор 3 навыков из областей публичных выступлений (`Oratory`), языков и профильных знаний.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "politician".into(),
    description: "Elected or appointed authority making a living directing government activities.".into(),
    wealth_range: vec![WealthLevel::Affluent, WealthLevel::Wealthy],
    skills: vec![
        SkillReq::Specific("Bargain".into()),
        SkillReq::Specific("Etiquette".into()),
        SkillReq::Specific("Fast Talk".into()),
        SkillReq::Specific("Insight".into()),
        SkillReq::Specific("Knowledge (Law)".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::Specific("Status".into()),
        SkillReq::ChooseFrom(3, vec![
            "Knowledge (Accounting)".into(), "Knowledge (Group)".into(),
            "Knowledge (History)".into(), "Knowledge (Region)".into(),
            "Listen".into(), "Language (Other)".into(),
            "Language (Own)".into(), "Perform (Oratory)".into(),
            "Research".into()
        ]),
    ],
    has_powers: false,
}
```

---

## Priest (Священник / Жрец)
Религиозный лидер, проповедник или аскет, посвятивший жизнь служению своему божеству.

### Механика / Концепция
- **Богатство (Wealth)**: От `Poor` (Бедный) до `Affluent` (Состоятельный), обычно `Average` (Средний). Может быть `Wealthy` (Богатый), если возглавляет крупную религиозную организацию.
- **Навыки (Skills)**: 8 базовых навыков, включающих религию, философию, ритуалы и убеждение. Плюс 2 навыка на выбор (оккультизм, грамотность, преподавание и т.д.).
- **Особые преимущества (Special)**: Принадлежность к вере или культу может давать доступ к магическим или сверхъестественным силам (см. [[Chapter 4: Powers]]).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "priest".into(),
    description: "Led by faith to the priesthood, preaching or worshipping their deity.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average, WealthLevel::Affluent, WealthLevel::Wealthy],
    skills: vec![
        SkillReq::Specific("Fast Talk".into()),
        SkillReq::Specific("Insight".into()),
        SkillReq::Specific("Knowledge (History)".into()),
        SkillReq::Specific("Knowledge (Philosophy)".into()),
        SkillReq::Specific("Knowledge (Religion)".into()),
        SkillReq::Specific("Language (Own)".into()),
        SkillReq::Specific("Perform (Ritual)".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::ChooseFrom(2, vec![
            "Knowledge (Occult)".into(), "Language (Other)".into(),
            "Listen".into(), "Literacy".into(), "Perform (Oratory)".into(),
            "Research".into(), "Status".into(), "Teach".into()
        ]),
    ],
    has_powers: true, // Доступ к божественной магии
}
```

---

## Sailor (Моряк)
Бороздит океаны, поддерживая целостность судна: пират, офицер военно-морского флота или древний торговец.

### Механика / Концепция
- **Богатство (Wealth)**: `Poor` (Бедный) или `Average` (Средний), обычно `Average`.
- **Навыки (Skills)**: 7 обязательных физических и корабельных навыков (плавание, лазание по снастям, управление лодкой). Плюс 3 дополнительных навыка на выбор (корабельная артиллерия, ремонт, командование).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "sailor".into(),
    description: "Plys the ocean waves, working to maintain the vessel's integrity.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average],
    skills: vec![
        SkillReq::Specific("Climb".into()),
        SkillReq::AnyOfCategory("Craft".into(), 1),
        SkillReq::Specific("Dodge".into()),
        SkillReq::Specific("Grapple".into()),
        SkillReq::Specific("Navigate".into()),
        SkillReq::Specific("Pilot (Boat)".into()),
        SkillReq::Specific("Swim".into()),
        SkillReq::ChooseFrom(3, vec![
            "Artillery (any)".into(), "Command".into(),
            "Language (Other)".into(), "Listen".into(),
            "Repair (Mechanical)".into(), "Repair (Structural)".into(),
            "Spot".into()
        ]),
    ],
    has_powers: false,
}
```

---

## Scholar (Ученый / Исследователь)
Академический исследователь, посвятивший жизнь изучению одной или нескольких интеллектуальных дисциплин. Он может преподавать или просто накапливать знания. (Фокус на гуманитарных и теоретических науках).

### Механика / Концепция
- **Богатство (Wealth)**: `Average` (Средний) или `Affluent` (Состоятельный), обычно `Average`.
- **Навыки (Skills)**: 5 базовых навыков для работы с информацией и преподавания (исследование, языки, убеждение, обучение). Плюс 5 профильных академических навыков из категорий `Knowledge` или `Science`, отражающих область исследований.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "scholar".into(),
    description: "Specializes in one or more fields of knowledge, seeking out intellectual domains.".into(),
    wealth_range: vec![WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::Specific("Language (Other)".into()),
        SkillReq::Specific("Language (Own)".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::Specific("Teach".into()),
        // Выбор 5 навыков, ограниченный категориями Knowledge или Science
        SkillReq::ChooseFrom(5, vec![
            "Knowledge (any)".into(), "Science (any)".into() 
        ]),
    ],
    has_powers: false,
}
```

---

## Scientist (Ученый-естествоиспытатель)
Работает на корпорацию, государство или независимо. Исследует науки через строгие эксперименты и практические наблюдения (фокус на естественных и точных науках).

### Механика / Концепция
- **Богатство (Wealth)**: `Average` (Средний) или `Affluent` (Состоятельный), обычно `Affluent`.
- **Навыки (Skills)**: 5 обязательных навыков, включающих статус, исследования, убеждение (вероятно, для получения грантов) и работу со сложным или компьютерным оборудованием. Плюс 5 профильных научных/технических навыков, связанных с полем исследований.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "scientist".into(),
    description: "Explores a field of science through rigorous speculation, experimentation, and observation.".into(),
    wealth_range: vec![WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::AnyOfCategory("Craft".into(), 1),
        SkillReq::Specific("Persuade".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::Specific("Status".into()),
        SkillReq::OneOf(vec!["Technical (Computer Use)".into(), "Heavy Machine".into()]),
        SkillReq::ChooseFrom(5, vec![
            "Knowledge (any)".into(), "Science (any)".into()
        ]),
    ],
    has_powers: false,
}
```

---

## Servant (Слуга / Помощник)
Наемный работник, обслуживающий домашнее хозяйство и нужды состоятельного работодателя (камердинер, адъютант генерала, судомойка в замке).

### Механика / Концепция
- **Богатство (Wealth)**: От `Poor` (Бедный) до `Average` (Средний). С разрешения Гейммастера может иметь ограниченный доступ к уровню жизни своего работодателя (`Affluent` или `Wealthy`).
- **Навыки (Skills)**: 6 обязательных навыков, связанных с обслуживанием, этикетом, скрытностью и умением "быть незаметным". Плюс 4 навыка на выбор, зависящих от конкретных обязанностей (вождение, первая помощь, языки, бухгалтерия).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "servant".into(),
    description: "Employed as a helper tending to the household affairs and domestic needs.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average], // Доступ к ресурсам босса обрабатывается нарративно
    skills: vec![
        SkillReq::AnyOfCategory("Craft".into(), 1),
        SkillReq::Specific("Etiquette".into()),
        SkillReq::Specific("Hide".into()),
        SkillReq::Specific("Language (Own)".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::Specific("Stealth".into()),
        SkillReq::ChooseFrom(4, vec![
            "Bargain".into(), "Drive".into(), "First Aid".into(),
            "Insight".into(), "Knowledge (Accounting)".into(),
            "Language (Other)".into(), "Persuade".into()
        ]),
    ],
    has_powers: false,
}
```

---

## Shaman (Шаман)
Племенной маг или духовный лидер, общающийся с миром духов и передающий знания своего племени. 

### Механика / Концепция
- **Богатство (Wealth)**: `Poor` (Бедный) или `Average` (Средний), что соответствует племенному уровню существования.
- **Навыки (Skills)**: 8 базовых навыков, включающих ритуалы, оккультизм, историю и проницательность. Плюс 2 дополнительных навыка на выбор (медицина, антропология, первая помощь).
- **Особые преимущества (Special)**: Шаманы почти всегда имеют доступ к магии или другим сверхъестественным силам (см. [[Chapter 4: Powers]]).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "shaman".into(),
    description: "Tribal magician, skilled in contacting the spirit world and lending advice.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average],
    skills: vec![
        SkillReq::AnyOfCategory("Art".into(), 1),
        SkillReq::Specific("Insight".into()),
        SkillReq::Specific("Knowledge (History)".into()),
        SkillReq::Specific("Knowledge (Occult)".into()),
        SkillReq::Specific("Language (Own)".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::Specific("Perform (Rituals)".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::ChooseFrom(2, vec![
            "Craft (any)".into(), "Fast Talk".into(), "First Aid".into(),
            "Hide".into(), "Knowledge (Anthropology)".into(),
            "Language (Other)".into(), "Medicine".into(),
            "Science (Pharmacy)".into(), "Status".into()
        ]),
    ],
    has_powers: true, // Доступ к магии духов/природы
}
```

---

## Slave (Раб / Пленник)
Захваченный или рожденный в неволе человек, принадлежащий хозяину, организации или религии. Может быть как уважаемым главой домашнего штата, так и бесправным чернорабочим.

### Механика / Концепция
- **Богатство (Wealth)**: От `Destitute` (Нищий) до `Poor` (Бедный). Иногда имеет доступ к уровню жизни хозяина.
- **Навыки (Skills)**: 9 жестко заданных навыков выживания, скрытности и социального взаимодействия (этикет, инсайт, понимание чужого языка). Плюс 1 навык, представляющий основную профессиональную обязанность раба (occupational speciality).
- **Примечание Гейммастера**: Профессия требует чувствительности к другим игрокам. Рекомендуется, чтобы персонаж с этой профессией уже был сбежавшим или освобожденным на момент начала игры (в этом случае профессия отражает его прошлый опыт).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "slave".into(),
    description: "Captured and enslaved or born into captivity. Usually escaped by the start of play.".into(),
    wealth_range: vec![WealthLevel::Destitute, WealthLevel::Poor],
    skills: vec![
        SkillReq::AnyOfCategory("Craft".into(), 1),
        SkillReq::Specific("Dodge".into()),
        SkillReq::Specific("Etiquette".into()),
        SkillReq::Specific("Fast Talk".into()),
        SkillReq::Specific("Hide".into()),
        SkillReq::Specific("Insight".into()),
        SkillReq::Specific("Language (Other)".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::Specific("Stealth".into()),
        SkillReq::ChooseFrom(1, vec!["Any one skill as an occupational speciality".into()]),
    ],
    has_powers: false,
}
```

---

## Soldier (Солдат)
Профессиональный военный, наемник или призывник, обученный сражаться в группе. 

### Механика / Концепция
- **Богатство (Wealth)**: `Poor` (Бедный) или `Average` (Средний). Однако солдаты часто имеют доступ к дорогостоящему или запрещенному оборудованию, выдаваемому государством/организацией.
- **Навыки (Skills)**: 4 базовых боевых и физических навыка (первая помощь, уклонение, рукопашный бой, лазание). Плюс выбор из 6 широких боевых, транспортных и тактических навыков для отражения военной специальности (артиллерист, снайпер, водитель танка).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "soldier".into(),
    description: "Professional soldier, mercenary, or conscript with martial training.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average], // Оружие выдается отдельно
    skills: vec![
        SkillReq::Specific("Brawl".into()),
        SkillReq::Specific("Climb".into()),
        SkillReq::Specific("Dodge".into()),
        SkillReq::Specific("First Aid".into()),
        SkillReq::ChooseFrom(6, vec![
            "Artillery".into(), "Command".into(), "Drive".into(),
            "Firearm (usually Rifle, but any)".into(), "Grapple".into(),
            "Heavy Weapon (any)".into(), "Hide".into(),
            "Language (Other)".into(), "Listen".into(), "Jump".into(),
            "Medicine".into(), "Melee Weapon (any)".into(),
            "Missile Weapon (any)".into(), "Navigate".into(),
            "Repair (Mechanical)".into(), "Ride".into(), "Spot".into(),
            "Stealth".into(), "Throw".into()
        ]),
    ],
    has_powers: false,
}
```

---

## Spy (Шпион / Агент)
Специалист по проникновению, саботажу и сбору информации под прикрытием. Работает на правительство, корпорацию или другую заинтересованную сторону.

### Механика / Концепция
- **Богатство (Wealth)**: От `Average` (Средний) до `Affluent` (Состоятельный).
- **Навыки (Skills)**: 7 обязательных навыков шпионажа (уклонение, скрытность, поиск улик, заговаривание зубов). Плюс 3 навыка на выбор, определяющих профиль агента (боевик, технарь, пилот или мастер маскировки).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "spy".into(),
    description: "Skilled in subterfuge, infiltration, and finding out secrets.".into(),
    wealth_range: vec![WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::Specific("Dodge".into()),
        SkillReq::Specific("Fast Talk".into()),
        SkillReq::Specific("Hide".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::Specific("Spot".into()),
        SkillReq::Specific("Stealth".into()),
        SkillReq::ChooseFrom(3, vec![
            "Art (Photography)".into(), "Brawl".into(), "Disguise".into(),
            "Etiquette".into(), "Firearm (any)".into(), "Grapple".into(),
            "Knowledge (any)".into(), "Language (Other)".into(),
            "Language (Own)".into(), "Martial Arts".into(), "Navigate".into(),
            "Pilot (any)".into(), "Psychology".into(), "Repair (Electronics)".into(),
            "Repair (Mechanical)".into(), "Ride".into(), "Swim".into(),
            "Technical (Computer Use)".into(), "Throw".into(), "Track".into()
        ]),
    ],
    has_powers: false,
}
```

---

## Student (Студент / Ученик)
Ученик, посвящающий время учебе: студент престижного университета, ученик в академии боевых искусств или подмастерье в школе волшебников.

### Механика / Концепция
- **Богатство (Wealth)**: От `Poor` (Бедный) до `Wealthy` (Богатый), обычно `Average` (Средний).
- **Навыки (Skills)**: Обязательное знание родного языка и навыков исследования. Игрок выбирает 8 дополнительных навыков, которые формируют "учебный план" (curriculum) студента.
- **Особые преимущества (Special)**: Ученики магических или эзотерических школ имеют доступ к соответствующим силам (магия, псионика, мутации).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "student".into(),
    description: "Spends time studying as a general student or apprentice.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average, WealthLevel::Affluent, WealthLevel::Wealthy],
    skills: vec![
        SkillReq::Specific("Language (Own)".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::ChooseFrom(8, vec![
            "Art (any)".into(), "Craft (any)".into(), "First Aid".into(),
            "Insight".into(), "Knowledge (any)".into(), "Language (Other)".into(),
            "Listen".into(), "Medicine".into(), "Repair (any)".into(),
            "Perform".into(), "Persuade".into(), "Psychotherapy".into(),
            "Science (any)".into(), "Technical (Computer Use)".into(),
            "Any one Physical skill".into()
        ]),
    ],
    has_powers: true, // В зависимости от школы
}
```

---

## Teacher (Учитель / Инструктор)
Преподаватель в школе, университете или частный репетитор, передающий свои знания другим.

### Механика / Концепция
- **Богатство (Wealth)**: `Average` (Средний) или `Affluent` (Состоятельный), обычно `Average`.
- **Навыки (Skills)**: 5 базовых педагогических навыков, включая навык `Teach`. Плюс 5 профильных навыков на выбор, отражающих предметы, которые учитель преподает.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "teacher".into(),
    description: "Instructor of one or more subjects, teaching groups or individuals.".into(),
    wealth_range: vec![WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::Specific("Language (Own)".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::Specific("Status".into()),
        SkillReq::Specific("Teach".into()),
        SkillReq::ChooseFrom(5, vec![
            "Art (any)".into(), "Craft (any)".into(), "First Aid".into(),
            "Insight".into(), "Knowledge (any)".into(), "Language (Other)".into(),
            "Listen".into(), "Medicine".into(), "Repair (any)".into(),
            "Perform".into(), "Persuade".into(), "Psychotherapy".into(),
            "Science (any)".into(), "Technical (Computer Use)".into()
        ]),
    ],
    has_powers: false,
}
```

---

## Technician (Техник / Системный инженер)
Специализируется на обслуживании, ремонте и использовании сложной техники, компьютеров или электроники. В отличие от `Engineer`, техник не проектирует системы, но знает, как с ними работать лучше, чем их создатели.

### Механика / Концепция
- **Богатство (Wealth)**: `Average` (Средний) или `Affluent` (Состоятельный).
- **Навыки (Skills)**: 8 фиксированных технических навыков (тонкая манипуляция, физика, компьютеры). Плюс 2 навыка на выбор (крафт, вождение или пилотирование).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "technician".into(),
    description: "Specialized in maintaining, repairing, and utilizing complex machinery or electronics.".into(),
    wealth_range: vec![WealthLevel::Average, WealthLevel::Affluent],
    skills: vec![
        SkillReq::Specific("Fine Manipulation".into()),
        SkillReq::Specific("Heavy Machine".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::AnyOfCategory("Repair".into(), 1),
        SkillReq::Specific("Science (Physics)".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::Specific("Spot".into()),
        SkillReq::Specific("Technical (Computer Use)".into()),
        SkillReq::ChooseFrom(2, vec![
            "Craft (any)".into(), "Drive".into(), "Pilot (any)".into()
        ]),
    ],
    has_powers: false,
}
```

---

## Thief (Вор / Грабитель)
От мелкого карманника до легендарного похитителя бриллиантов или разбойника с большой дороги. Игнорирует закон ради наживы.

### Механика / Концепция
- **Богатство (Wealth)**: Любое, на усмотрение Гейммастера (от нищего карманника до богатого мафиози).
- **Навыки (Skills)**: 5 обязательных воровских навыков (оценка, скрытность, уклонение). Плюс 5 навыков на выбор, определяющих профиль: медвежатник, форточник, социальный мошенник или уличный грабитель.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "thief".into(),
    description: "Takes what they want through deception, stealth, or force.".into(),
    wealth_range: vec![WealthLevel::Any],
    skills: vec![
        SkillReq::Specific("Appraise".into()),
        SkillReq::Specific("Dodge".into()),
        SkillReq::Specific("Fast Talk".into()),
        SkillReq::Specific("Hide".into()),
        SkillReq::Specific("Stealth".into()),
        SkillReq::ChooseFrom(5, vec![
            "Bargain".into(), "Brawl".into(), "Climb".into(),
            "Disguise".into(), "Fine Manipulation".into(),
            "Firearm (Pistol)".into(), "Firearm (Revolver)".into(),
            "Firearm (Shotgun)".into(), "Grapple".into(), "Insight".into(),
            "Listen".into(), "Jump".into(), "Knowledge (Law)".into(),
            "Persuade".into(), "Repair (Mechanical)".into(), "Spot".into()
        ]),
    ],
    has_powers: false,
}
```

---

## Tribesperson (Племенной житель / Дикарь)
Член примитивной культуры, незнакомый с благами цивилизации. Выживает за счет охоты и собирательства.

### Механика / Концепция
- **Богатство (Wealth)**: От `Destitute` (Нищий) до `Poor` (Бедный), что соответствует племенному уровню жизни (хотя вождь может иметь более высокий статус внутри племени).
- **Навыки (Skills)**: 8 базовых навыков охотника-собирателя. Плюс 2 навыка на выбор (медицина, оккультизм, плавание, стрельба из лука).

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "tribesperson".into(),
    description: "Accepted member of a tribe, unfamiliar with civilization, survives by hunting/foraging.".into(),
    wealth_range: vec![WealthLevel::Destitute, WealthLevel::Poor],
    skills: vec![
        SkillReq::AnyOfCategory("Craft".into(), 1),
        SkillReq::Specific("Dodge".into()),
        SkillReq::Specific("Grapple".into()),
        SkillReq::Specific("Hide".into()),
        SkillReq::Specific("Knowledge (Natural History)".into()),
        SkillReq::Specific("Spot".into()),
        SkillReq::Specific("Throw".into()),
        SkillReq::Specific("Track".into()),
        SkillReq::ChooseFrom(2, vec![
            "Brawl".into(), "Climb".into(), "First Aid".into(),
            "Listen".into(), "Jump".into(), "Knowledge (Occult)".into(),
            "Melee Weapon (Spear)".into(), "Melee Weapon (Club)".into(),
            "Missile Weapon (Bow)".into(), "Language (Other)".into(),
            "Ride".into(), "Stealth".into(), "Swim".into()
        ]),
    ],
    has_powers: false,
}
```

---

## Warrior (Воин / Мастер боя)
Специалист по индивидуальному бою. В отличие от солдат (`Soldier`), которые сражаются в отрядах, воин опирается только на свои рефлексы и личное мастерство (рыцарь, самурай, наемник, варвар).

### Механика / Концепция
- **Богатство (Wealth)**: От `Destitute` (Нищий) до `Average` (Средний), обычно `Poor` (Бедный).
- **Навыки (Skills)**: 5 базовых боевых навыков (рукопашный бой, оружие ближнего и дальнего боя). Плюс 5 навыков на выбор, отражающих физическую подготовку и тактику.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "warrior".into(),
    description: "Specializes in individual combat, surviving by reflexes and weapon skills.".into(),
    wealth_range: vec![WealthLevel::Destitute, WealthLevel::Poor, WealthLevel::Average],
    skills: vec![
        SkillReq::Specific("Brawl".into()),
        SkillReq::Specific("Dodge".into()),
        SkillReq::Specific("Grapple".into()),
        SkillReq::AnyOfCategory("Melee Weapon".into(), 1),
        SkillReq::AnyOfCategory("Missile Weapon".into(), 1),
        SkillReq::ChooseFrom(5, vec![
            "Climb".into(), "Firearm (any)".into(), "Hide".into(),
            "Listen".into(), "Jump".into(), "Language (Other)".into(),
            "Martial Arts".into(), "Ride".into(), "Spot".into(),
            "Stealth".into(), "Swim".into(), "Throw".into(), "Track".into()
        ]),
    ],
    has_powers: false,
}
```

---

## Wizard (Маг / Волшебник)
Специалист, изучающий и практикующий магию (`Magic`) или колдовство (`Sorcery`). Посвящает себя увеличению запаса магической силы и изучению заклинаний.

### Механика / Концепция
- **Богатство (Wealth)**: От `Poor` (Бедный) до `Wealthy` (Богатый), обычно `Affluent` (Состоятельный).
- **Навыки (Skills)**: 10 академических и мистических навыков (оккультизм, языки, ритуалы).
- **Особые преимущества (Special)**: Волшебник по определению имеет доступ к магии или колдовству. Стартовые заклинания выбираются в сотрудничестве с Гейммастером согласно правилам из [[Chapter 4: Powers]].

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "wizard".into(),
    description: "Understands and uses magic or sorcery, dedicating themselves to spells.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average, WealthLevel::Affluent, WealthLevel::Wealthy],
    skills: vec![
        SkillReq::AnyOfCategory("Craft".into(), 1),
        SkillReq::Specific("Insight".into()),
        SkillReq::Specific("Knowledge (Occult)".into()),
        SkillReq::AnyOfCategory("Knowledge".into(), 2),
        SkillReq::Specific("Language (Other)".into()),
        SkillReq::Specific("Listen".into()),
        SkillReq::Specific("Perform (Rituals)".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::Specific("Research".into()),
    ],
    has_powers: true, // Гарантированный доступ к Magic или Sorcery
}
```

---

## Writer (Писатель / Сценарист)
Создатель прозы, поэзии, сценариев или иных текстов. Владеет силой слова, чтобы развлекать, информировать или убеждать читателей.

### Механика / Концепция
- **Богатство (Wealth)**: От `Poor` (Бедный) до `Wealthy` (Богатый), обычно `Average` (Средний).
- **Навыки (Skills)**: 5 обязательных навыков, сфокусированных на владении родным языком (`Language (Own)`), искусстве письма (`Art (Writing)`) и исследованиях. Плюс 5 навыков на выбор, отражающих темы, о которых он пишет, или его методы сбора информации.

### Архитектура Rust
```rust
ProfessionTemplate {
    id: "writer".into(),
    description: "Writes prose, poetry, or scripts, using words to entertain, inform, or persuade.".into(),
    wealth_range: vec![WealthLevel::Poor, WealthLevel::Average, WealthLevel::Affluent, WealthLevel::Wealthy],
    skills: vec![
        SkillReq::Specific("Art (Writing)".into()),
        SkillReq::Specific("Insight".into()),
        SkillReq::Specific("Language (Own)".into()),
        SkillReq::Specific("Persuade".into()),
        SkillReq::Specific("Research".into()),
        SkillReq::ChooseFrom(5, vec![
            "Fast Talk".into(), "Knowledge (any)".into(),
            "Language (Other)".into(), "Listen".into(),
            "Status".into(), "Technical (Computer Use)".into()
        ]),
    ],
    has_powers: false,
}
```

---

## Создание новых профессий (Creating New Professions)
Гейммастер и игроки не ограничены базовым списком и могут создавать собственные профессии для специфических сеттингов.

### Механика / Концепция
Для создания новой профессии необходимо выполнить следующие шаги:
1. **Название**: Придумать подходящее название.
2. **Описание**: Устно или письменно описать Гейммастеру, чем занимается этот человек, какое у него образование и положение в обществе.
3. **Уровень богатства**: Назначить подходящий уровень (Wealth Level) или диапазон.
4. **Выбор навыков**: Выбрать ровно **10 навыков**, которые наиболее важны для этой профессии. Можно использовать механику "выбор N из списка", но суммарное количество доступных стартовых слотов для вложения профессиональных очков всегда должно равняться 10.

*Альтернатива:* Если задумка похожа на уже существующую профессию (например, Водитель такси похож на `Pilot`), Гейммастер может взять существующий шаблон и просто заменить в нем 2-3 навыка на более подходящие, а также скорректировать уровень богатства.

### Архитектура Rust
Механизм создания пользовательских профессий идеально ложится на сериализацию/десериализацию конфигурационных файлов (например, JSON или TOML), позволяя Гейммастеру добавлять новые шаблоны без изменения исходного кода ядра.

```rust
// Пример валидатора для проверки корректности пользовательской профессии
impl ProfessionTemplate {
    pub fn validate_custom_profession(&self) -> Result<(), &'static str> {
        let mut total_slots = 0;
        
        for req in &self.skills {
            match req {
                SkillReq::Specific(_) => total_slots += 1,
                SkillReq::AnyOfCategory(_, count) => total_slots += *count,
                SkillReq::ChooseFrom(count, _) => total_slots += *count,
                SkillReq::OneOf(_) => total_slots += 1,
            }
        }

        if total_slots != 10 {
            return Err("A profession must provide exactly 10 skill slots.");
        }

        if self.wealth_range.is_empty() {
            return Err("A profession must have at least one valid wealth level.");
        }

        Ok(())
    }
}
```