# Оглавление файла
- [[#Скрытые или Тайные Броски (Secret or Concealed Skill Rolls)]]
---
## Скрытые или Тайные Броски (Secret or Concealed Skill Rolls)
Инструмент Гейммастера для предотвращения метагейминга (использования игроком знаний, которых нет у персонажа).

### Механика / Концепция
Гейммастер может делать броски навыков за персонажа в тайне от игрока. Это применяется, когда игрок не должен знать исход проверки, или даже сам факт того, что проверка совершается.
Чаще всего это касается навыков Восприятия (`Perception`) или Ментальных (`Mental`) навыков (например: `Spot`, `Listen`, `Insight`, `Hide`, `Stealth`, `Appraise`). 

Если игрок сам кидает кубики на поиск ловушки (`Spot`) и проваливает бросок, он знает, что мог что-то упустить. Если бросок делает ГМ в тайне и сообщает "Вы ничего не нашли", игрок не знает, действительно ли там ничего нет, или персонаж просто не заметил угрозу.
- **Успех**: ГМ сообщает игроку результат и позволяет отметить галочку опыта (`Experience Check`).
- **Провал**: ГМ может сказать, что ничего не произошло, дать ложную информацию (misinform) или вообще промолчать.

ГМ может использовать гибридный подход: сказать игроку бросить кубики, но так, чтобы результат видел только ГМ (например, за ширмой).

### Архитектура Rust
Для реализации VTT (Virtual Tabletop) или движка, тайные броски должны иметь флаг видимости результата, который маршрутизирует ответ только на клиент Гейммастера.

```rust
pub enum RollVisibility {
    Public,       // Видят все
    PlayerAndGm,  // Видит игрок, совершающий бросок, и ГМ
    GmOnly,       // Видит только ГМ (Secret Roll)
}

pub struct RollEvent {
    pub character_id: u32,
    pub skill_name: String,
    pub result: RollResult,
    pub visibility: RollVisibility,
}

pub fn execute_secret_roll(
    skill_rating: u16, 
    difficulty: TaskDifficulty
) -> RollEvent {
    let target = SkillRollRequest { base_rating: skill_rating, difficulty }.get_target_chance();
    
    // Внутренняя логика броска...
    let rolled_value = roll(1, d100);
    
    RollEvent {
        character_id: 1, // ID персонажа
        skill_name: "Spot".to_string(),
        result: evaluate_roll(rolled_value, target.unwrap_or(100)),
        visibility: RollVisibility::GmOnly, // Результат не отправляется клиенту игрока
    }
}
```
