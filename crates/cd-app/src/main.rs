use cd_core::{ObjectGuid, WorldPos};
use cd_ecs::components::{Name, Position, Render, Stats};
use cd_map::{Chunk, Tile, MaterialId, TileFlags};
use hecs::World;
use tracing::{info, Level};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    info!("🚀 Initializing Engine Context...");

    // 1. Создаем Мир ECS
    let mut ecs_world = World::new();

    // 2. Спавним игрока (Archetype)
    let player_entity = ecs_world.spawn((
        Name("Leeroy Jenkins".to_string()),
        Position(WorldPos::new(10, 10, 0)),
        Render { glyph: '@', color_rgb: 0x00FF00 },
        Stats { hp: 100, max_hp: 100, mana: 50, max_mana: 50, is_dead: false }
    ));

    info!("ECS: Spawned Player Entity: {:?}", player_entity);

    // 3. Тест доступа к компонентам
    // Query - это основной способ работы в ECS
    for (id, (name, pos)) in ecs_world.query::<(&Name, &Position)>().iter() {
        info!("Entity {:?}: '{}' at {:?}", id, name.0, pos.0);
    }

    // 4. Тест Карты
    let mut chunk = Chunk::new();
    let wall = Tile {
        material: MaterialId(10), // Stone Wall
        flags: TileFlags::SOLID | TileFlags::OPAQUE,
        variant: 0,
    };

    // Ставим стену в (5, 5) внутри чанка
    chunk.set(5, 5, wall);

    // Проверяем
    if let Some(tile) = chunk.get(5, 5) {
        info!("Map: Tile at (5,5) is Solid? {}", tile.is_solid()); // true
        info!("Map: Tile at (0,0) is Solid? {}", chunk.get(0, 0).unwrap().is_solid()); // false
    }
}