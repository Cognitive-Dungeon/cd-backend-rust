use cd_core::{ObjectGuid, WorldPos};
use cd_engine::{Engine, InputCmd};
use cd_map::{Chunk, Tile, MaterialId, TileFlags};
use tracing::{info, Level};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Initializing Cognitive Dungeon Server...");

    // 1. Инициализация Движка
    let mut engine = Engine::new();

    // 2. Создаем карту (для теста)
    let mut chunk = Chunk::new();
    // Ставим стену на (10, 11)
    chunk.set(10, 11, Tile {
        material: MaterialId(1),
        flags: TileFlags::SOLID,
        variant: 0,
    });
    engine.map.insert_chunk(0, 0, chunk);

    // 3. Спавним игрока
    let player_id = ObjectGuid::new(1, 1, 1, 1);
    let start_pos = WorldPos::new(10, 10, 0);
    engine.spawn_player(player_id, "Tester".to_string(), start_pos);

    // 4. Эмуляция Игрового Цикла (3 тика)
    info!("--- STARTING LOOP ---");

    // Тик 1: Попытка пройти сквозь стену
    info!("Tick 1: Try move into wall");
    let inputs = vec![InputCmd::Move {
        entity_guid: player_id,
        target: WorldPos::new(10, 11, 0), // Там стена!
    }];
    engine.tick(inputs);

    // Тик 2: Движение в пустоту
    info!("Tick 2: Move to empty space");
    let inputs = vec![InputCmd::Move {
        entity_guid: player_id,
        target: WorldPos::new(10, 12, 0), // Там пусто (Chunk default is void/empty, но в нашей логике world.is_solid проверяет чанк)
        // В world.rs мы написали: если чанка нет - false (пусто).
        // Чанк (0,0) есть, тайл (10,12) пустой (VOID).
        // Tile::VOID flags = NONE, значит is_solid = false.
    }];
    engine.tick(inputs);

    // Тик 3: Просто холостой ход
    info!("Tick 3: Idle");
    engine.tick(vec![]);
}