use cd_core::{ObjectGuid, WorldPos};
use tracing::{info, Level};

#[tokio::main]
async fn main() {
    // 1. Инициализация логирования
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    info!("🚀 Cognitive Dungeon Server (Rust Rewrite) starting...");

    // 2. Тест Core примитивов
    // Эмуляция создания игрока
    let player_guid = ObjectGuid::new(1, 1, 1, 500); // Shard 1, Type 1 (Player), Gen 1, Index 500
    let spawn_pos = WorldPos::new(100, 200, 0);

    info!("Spawned Entity: {:?} at {:?}", player_guid, spawn_pos);

    // Проверка математики координат
    let target_pos = WorldPos::new(110, 200, 0);
    let dist_sq = spawn_pos.distance_squared(target_pos);

    info!("Distance check: {} (Expected 100)", dist_sq);

    // Здесь позже будет запуск ECS лупа и сетевого слоя
    // run_server().await;
}