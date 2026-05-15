# cd-net

Сетевой слой: WebSocket-сервер, сессии, протокол, снапшоты.  
Переводит сетевые сообщения в команды движка и обратно.

## Публичный API

### Сервер

- `run_server` — запускает WebSocket-сервер
- `Router` — таблица маршрутов (path → handler)

### Состояние

- `ApiState` / `SharedApiState` — общий стейт: `CommandSender` + `Engine`
- `ApiEntity` — представление сущности для API
- `ReloadCallback` — колбэк для hot-reload данных

### Снапшоты (для отправки клиенту)

- `EntitySnapshot` — плоское представление сущности (guid, x, y, glyph)
- `ChunkSnapshot` — плоское представление чанка (палитра + индексы)
- `SnapshotBuilder` — строит снапшоты из ECS World

## Не входит в scope

- Игровая логика — это `cd-engine`
- ECS-компоненты — это `cd-ecs`

## Зависит от

- `cd-core`
- `cd-ecs`, `cd-engine`
- `cd-telemetry`
- `bevy_ecs`
