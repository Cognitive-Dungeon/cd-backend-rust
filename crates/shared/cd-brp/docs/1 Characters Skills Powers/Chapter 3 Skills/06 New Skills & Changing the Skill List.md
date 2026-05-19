# Оглавление файла
- [[#Создание и Изменение Навыков (New Skills & Changing the Skill List)]]
---
## Создание и Изменение Навыков (New Skills & Changing the Skill List)
Гибкость системы позволяет адаптировать список навыков под любой жанр.

### Механика / Концепция
Гейммастер и игроки могут:
- **Создавать новые навыки**: В листе персонажа всегда есть пустые строки для новых навыков. Например, навык `Projection`, вводимый для стрельбы суперспособностями.
- **Переименовывать навыки**: Адаптировать названия под сеттинг (например, переименовать `Fine Manipulation` в `Pick Lock` или `Devise` для средневекового фэнтези).
- **Удалять навыки**: Полностью исключать навыки, которые не вписываются в стиль игры.

**Важное требование:** Любые изменения в списке навыков (добавление, удаление, переименование) должны быть утверждены **до начала создания персонажей**, чтобы игроки не потратили очки на навыки, которые будут удалены или изменены.

### Архитектура Rust
Список навыков должен быть словарем (Map) или списком (List), подгружаемым из внешней конфигурации сессии, а не захардкоженным `enum`.

```rust
use std::collections::HashMap;

pub struct SkillRegistry {
    pub skills: HashMap<String, SkillDefinition>,
}

impl SkillRegistry {
    pub fn load_from_config(config: &GameSessionConfig) -> Self {
        let mut registry = HashMap::new();
        // Загрузка базовых навыков...
        
        // Переименование или удаление на основе настроек сеттинга
        if config.setting == SettingType::Fantasy {
            if let Some(mut fine_manipulation) = registry.remove("Fine Manipulation") {
                fine_manipulation.name = "Pick Lock".to_string();
                registry.insert("Pick Lock".to_string(), fine_manipulation);
            }
        }
        
        Self { skills: registry }
    }
}
```