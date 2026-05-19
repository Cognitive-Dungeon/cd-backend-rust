# Оглавление файла
- [[#Специализации Навыков (Skill Specialities)]]
---
## Специализации Навыков (Skill Specialities)
Некоторые навыки описывают слишком широкие области знаний и требуют сужения до конкретной направленности.

### Механика / Концепция
Такие навыки, как Искусство (`Art`), Ремесло (`Craft`) или Наука (`Science`), являются широкими категориями. Игрок должен определить специализацию — более узкий фокус этого навыка. 

Специализации указываются в скобках после названия навыка. Например, `Melee Weapon (Sword)` и `Melee Weapon (Spear)` считаются **двумя абсолютно разными навыками**, и их рейтинги прокачиваются и используются независимо друг от друга.

**Использование родственных специализаций:**
Если у персонажа нет нужной специализации, но есть родственная (related) в рамках того же базового навыка, Гейммастер может разрешить использовать её со штрафом: рейтинг родственного навыка делится пополам с округлением вверх (`ceil`). Родственность специализаций определяет Гейммастер.

> For example, your character can use half their skill rating in Science (Astronomy) to make skill rolls that would normally require Science (Physics) or Science (Mathematics), as these are related skills. However, this astronomical acumen is useless if the gamemaster calls for a Science (Biology) or Knowledge (History) skill roll.

### Архитектура Rust
Для поддержки специализаций структура навыка должна позволять опциональное уточнение, а механика броска — уметь рассчитывать "половинный" шанс для родственных проверок.

```rust
pub struct CharacterSkill {
    pub definition_id: String, // Например, "science"
    pub speciality: Option<String>, // Например, Some("Astronomy")
    pub rating: SkillRating,
}

impl CharacterSkill {
    /// Получение шанса на успех, если навык используется по прямому назначению
    pub fn get_effective_rating(&self) -> SkillRating {
        self.rating
    }

    /// Получение шанса на успех при использовании как "родственного" (related) навыка
    pub fn get_related_rating(&self) -> SkillRating {
        (self.rating as f32 / 2.0).ceil() as SkillRating
    }
    
    /// Удобный метод форматирования для UI, возвращающий "Science (Astronomy)"
    pub fn display_name(&self, base_name: &str) -> String {
        match &self.speciality {
            Some(spec) => format!("{} ({})", base_name, spec),
            None => base_name.to_string(),
        }
    }
}
```

### Граничные случаи и Критические исходы
Если рейтинг родственного навыка нечетный (например, `45%`), половинное значение вычисляется как `45 / 2 = 22.5` и округляется вверх до `23%`. Навыки из совершенно разных категорий (например, `Science (Astronomy)` вместо `Knowledge (History)`) не могут использоваться для замены друг друга ни при каких условиях.