# cd-map

Хранение и доступ к тайловой карте мира.  
Отвечает только за геометрию и содержимое тайлов — не за сущности на них.

## Публичный API

### Иерархия карты

```
WorldMap → Shard → Region → Chunk (32×32) → Tile
```

- `WorldMap` — точка входа; хранит шарды, отвечает на `get_tile(WorldPos)` и `is_solid_fast(WorldPos)`
- `Chunk` — основная единица симуляции; 16×16 тайлов
- `Tile` — один тайл: `material: MaterialID`, `flags: TileFlags`, `variant: u8`
- `TileFlags` — битовые флаги: `SOLID`, `OPAQUE`, ...
- `MaterialID` — числовой идентификатор материала

## Не входит в scope

- Сущности, игроки, существа, предметы (это `cd-ecs`)
- Пространственный индекс сущностей (это `cd-ecs::SpatialGrid`)
- Генерация карты (это `cd-engine`)

## Зависит от

- `cd-core`
