## Использование Навыков (Using Skills)
Основная механика разрешения действий с неочевидным исходом.

### Механика / Концепция
Когда персонаж пытается совершить действие, успех которого не гарантирован, игрок заявляет о своих намерениях, а `Gamemaster` (Гейммастер) запрашивает бросок `D100` против рейтинга соответствующего навыка.
- **Успех**: Выпавшее на `D100` значение меньше или равно модифицированному рейтингу навыка.
- **Провал**: Выпавшее значение больше рейтинга навыка.

**Модификаторы сложности (Difficulty Modifiers):**
Обстоятельства могут изменять рейтинг навыка перед броском:
- `Automatic` (Автоматически): Бросок не требуется, успех гарантирован.
- `Easy` (Легко): Рейтинг навыка удваивается (`x2`).
- `Average` (Средне): Нет модификаторов (`x1`).
- `Difficult` (Сложно): Рейтинг навыка делится пополам (`x0.5`).
- `Impossible` (Невозможно): Бросок не делается, либо шанс равен `01%` (на усмотрение ГМ'а).
*Если сложность не указана явно, предполагается `Average`.*

**Альтернативное применение и Знания:**
Навыки не только определяют способность совершить физическое действие, но и представляют теоретические знания в этой области. Игроки могут предлагать альтернативные варианты использования навыков с одобрения ГМ'а.
> For example, a medieval warrior might use the Melee Weapon (Sword) skill instead of Appraise to judge a sword’s quality. The Martial Arts skill might similarly be used to know about the different dojos in a city and who their senseis are.

### Архитектура Rust
```rust
pub enum TaskDifficulty {
    Automatic,
    Easy,
    Average,
    Difficult,
    Impossible,
}

pub struct SkillRollRequest {
    pub base_rating: u16,
    pub difficulty: TaskDifficulty,
}

impl SkillRollRequest {
    pub fn get_target_chance(&self) -> Option<u16> {
        match self.difficulty {
            TaskDifficulty::Automatic => None, // Бросок не нужен
            TaskDifficulty::Easy => Some(self.base_rating.saturating_mul(2)),
            TaskDifficulty::Average => Some(self.base_rating),
            TaskDifficulty::Difficult => Some((self.base_rating as f32 / 2.0).ceil() as u16),
            TaskDifficulty::Impossible => Some(1), // Или None, в зависимости от настроек GM
        }
    }
}
```
