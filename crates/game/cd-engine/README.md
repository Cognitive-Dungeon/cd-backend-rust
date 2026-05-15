# cd-engine

Игровой движок: тик-симуляция, системы, ресурсы мира.  
Точка входа для всей игровой логики. Сеть общается с движком через `CommandSender`.

## Публичный API

### Запуск

- `Engine` — запущенный движок; тикает по расписанию
- `EngineBuilder` — конструктор движка (настройка ресурсов, seed, telemetry)

### Команды (вход)

- `CommandSender` / `CommandBus` — канал для отправки команд в движок из сети
- `InputCmd` — перечисление команд: `SpawnPlayer`, `Move`, ...

### Тик

- `TickId` — монотонный счётчик тиков
- `TickContext` — контекст текущего тика (id, seed)

### Ресурсы мира (`world/`)

- `MapResource` — ECS-ресурс: карта мира (`WorldMap`)
- `GridResource` — ECS-ресурс: пространственный индекс (`SpatialGrid`)
- `RegistryResource` — ECS-ресурс: реестр сущностей (`EntityRegistry`)
- `DefsCache` — ECS-ресурс: загруженные определения из Depot
- `GameDataResource` — ECS-ресурс: `Arc<RwLock<Depot>>` (поддерживает hot-reload)
- `TickResource` — ECS-ресурс: текущий тик и seed
- `TelemetryResource` — ECS-ресурс: sink для событий

### Телеметрия (реэкспорт из cd-telemetry)

- `TelemetrySink`, `EngineEvent`, `BroadcastSink`, `NullSink`

## Не входит в scope

- Сетевой протокол и HTTP/WS — это `cd-net`
- Определения существ/материалов — это `cd-data`
- ECS-компоненты — это `cd-ecs`

## Зависит от

- `cd-core`
- `cd-ecs`, `cd-map`, `cd-data`
- `cd-telemetry`
- `bevy_ecs`
