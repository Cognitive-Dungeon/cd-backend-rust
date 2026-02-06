use cd_core::{ObjectGuid, WorldPos};
use cd_engine::{Engine, InputCmd};
use cd_map::{Chunk, MaterialId, Tile, TileFlags};
use cd_net::{protocol::ServerPacket, protocol::EntityView};
use std::thread;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, Level};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Booting Cognitive Dungeon...");

    // 1. Создаем каналы связи
    // Сеть -> Движок (Команды)
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<InputCmd>(1024);

    // Движок -> Сеть (Снапшоты)
    // Broadcast канал: один писатель (движок), много читателей (вебсокеты)
    let (snapshot_tx, _) = broadcast::channel::<ServerPacket>(16);
    let snapshot_tx_net = snapshot_tx.clone();

    // 2. Запускаем Движок в отдельном OS потоке (CPU Bound)
    thread::spawn(move || {
        let mut engine = Engine::new();

        // Setup Map (Test)
        let mut chunk = Chunk::new();
        chunk.set(5, 5, Tile { material: MaterialId(1), flags: TileFlags::SOLID, variant: 0 });
        engine.map.insert_chunk(0, 0, chunk);

        // Spawn Test Player (чтобы было кем управлять)
        // В реальной жизни это должно происходить по команде Login
        let player_guid = ObjectGuid::new(1, 1, 1, 4); // index 4 (по длине слова "test")
        engine.spawn_player(player_guid, "NetPlayer".to_string(), WorldPos::new(0, 0, 0));

        let tick_rate = Duration::from_millis(50); // 20 TPS
        let mut tick_counter = 0;

        loop {
            let start = std::time::Instant::now();

            // A. Читаем все накопленные команды из сети (Non-blocking)
            let mut inputs = Vec::new();
            while let Ok(cmd) = cmd_rx.try_recv() {
                inputs.push(cmd);
            }

            // B. Тик Симуляции
            engine.tick(inputs);

            // C. Генерация Снапшота (Mock)
            // В реальной системе тут будет engine.create_snapshot()
            let mut entities_view = Vec::new();

            // Запрашиваем данные из ECS для рендера
            // Тут мы нарушаем изоляцию для демо, в проде это будет внутри engine.snapshot()
            for (id, (pos, render)) in engine.world.query::<(&cd_ecs::components::Position, &cd_ecs::components::Render)>().iter() {
                // Тут нужен маппинг Entity -> Guid, но пока фейк
                entities_view.push(EntityView {
                    guid: "test-guid".to_string(), // TODO: use real guid
                    x: pos.0.x(),
                    y: pos.0.y(),
                    glyph: render.glyph,
                    color: format!("#{:06X}", render.color_rgb),
                });
            }

            let packet = ServerPacket::Snapshot {
                tick: tick_counter,
                entities: entities_view,
            };

            // D. Отправка в сеть
            // Игнорируем ошибку, если нет слушателей
            let _ = snapshot_tx.send(packet);

            tick_counter += 1;

            // E. Sleep (Maintain Tick Rate)
            let elapsed = start.elapsed();
            if elapsed < tick_rate {
                thread::sleep(tick_rate - elapsed);
            }
        }
    });

    // 3. Запускаем Сеть (IO Bound) в текущем потоке (Tokio Runtime)
    cd_net::run_server(8080, cmd_tx, snapshot_tx_net).await;
}