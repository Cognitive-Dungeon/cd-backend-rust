# Оглавление файла
- [[#Опциональное правило: Типы Личности (Personality Types)]]

---

## Опциональное правило: Типы Личности (Personality Types)
Это опциональное правило предлагает быстрый способ детализировать персонажа, выдав ему пакет стартовых навыков, соответствующих его архетипу характера и подходу к решению проблем.

### Механика / Концепция
Игрок может выбрать один из четырех предложенных типов личности, либо определить его случайно броском `roll(1, d4)`. 

Каждый тип личности содержит список из 13 навыков (`Skills`). Применение пакета добавляет фиксированные **20 очков навыков (20%)** к [[Terms Used in Basic Roleplaying#Base Chance|базовому шансу]] (Base Chance) каждого из этих 13 навыков.

> For example, adding 20 skill points to Fast Talk (15%) yields a rating of 15+20=35%.

**Таблица Personality Types:**

| Результат (1D4) | Тип Личности | Описание и Навыки |
| :--- | :--- | :--- |
| 1 | **Brutal** (Жестокий) | Решает проблемы грубой силой. +20% к: `Brawl`, `Climb`, `Dodge`, `Grapple`, `Insight`, `Jump`, `Ride`, `Sense`, `Stealth`, `Swim`, `Throw` и любым двум навыкам категории *Combat*. |
| 2 | **Skilled** (Умелый) | Верит в технику, ремесло и экспертизу. +20% к: `Appraise`, любому одному `Craft`, `Disguise`, `Dodge`, `Fine Manipulation`, `First Aid`, любому одному навыку *Knowledge*, `Navigate`, `Pilot`, `Ride`, `Sleight of Hand`, `Stealth` и любому одному навыку категории *Combat*. |
| 3 | **Cunning** (Хитрый) | Пытается перехитрить оппонента для получения преимущества. +20% к: `Appraise`, `Bargain`, `Disguise`, `Insight`, любым двум навыкам *Knowledge*, `Listen`, `Research`, `Sense`, `Spot`, `Stealth`, любому одному навыку *Technical* (соответствующему сеттингу) и любому одному навыку категории *Combat*. |
| 4 | **Charming** (Обаятельный) | Убеждает других делать работу за него. +20% к: `Appraise`, `Bargain`, `Command`, `Etiquette`, `Fast Talk`, `Insight`, `Perform`, `Persuade`, любому одному `Language (Other)`, `Language (Own)`, `Sense`, `Status` и любому одному навыку категории *Combat*. |

**Кастомные типы:** Гейммастер может создавать собственные типы личностей по этой же формуле: выбрать ровно 13 навыков и добавить к каждому из них +20%.

### Архитектура Rust
С точки зрения структур данных, пакеты навыков можно представить как шаблон конфигурации, который накладывается на персонажа на этапе работы `CharacterBuilder`. Так как некоторые навыки требуют выбора игрока (например, "любой один навык категории Combat"), структура должна поддерживать отложенный выбор (Unresolved Choices).

```rust
pub enum PersonalityType {
    Brutal,
    Skilled,
    Cunning,
    Charming,
    Custom(String, Vec<SkillChoice>),
}

pub enum SkillChoice {
    Specific(String),            // Точно заданный навык, напр. "Dodge"
    Category(SkillCategory, u8), // Выбор из категории, напр. (Combat, 2)
}

pub struct PersonalityPackage {
    pub personality_type: PersonalityType,
    pub bonus_value: u16, // Обычно 20
}

impl PersonalityPackage {
    pub fn generate_choices(roll_result: u8) -> Self {
        let choices = match roll_result {
            1 => vec![
                SkillChoice::Specific("Brawl".to_string()),
                SkillChoice::Specific("Climb".to_string()),
                // Остальные конкретные навыки...
                SkillChoice::Category(SkillCategory::Combat, 2),
            ],
            2 => vec![
                SkillChoice::Specific("Appraise".to_string()),
                SkillChoice::Specific("Craft (Any)".to_string()), // Требует уточнения
                // ...
                SkillChoice::Category(SkillCategory::Combat, 1),
            ],
            3 => vec![
                // ...
                SkillChoice::Category(SkillCategory::Mental, 2), // Knowledge skills
            ],
            4 => vec![
                // ...
            ],
            _ => vec![], // Резерв для обработки ошибок
        };

        Self {
            personality_type: match roll_result {
                1 => PersonalityType::Brutal,
                2 => PersonalityType::Skilled,
                3 => PersonalityType::Cunning,
                4 => PersonalityType::Charming,
                _ => PersonalityType::Custom("Unknown".to_string(), vec![]),
            },
            bonus_value: 20,
        }
    }
}
```

### Граничные случаи и Критические исходы
Этот бонус применяется **в дополнение** к базовому шансу навыка и любым бонусным модификаторам от характеристик, но **до** распределения профессиональных или личных очков навыков, которое происходит в [[Step Seven: Profession and Skills]]. Это важно для проверки лимитов (максимального допустимого значения навыка на старте игры), которые будут накладываться на следующих шагах генерации.