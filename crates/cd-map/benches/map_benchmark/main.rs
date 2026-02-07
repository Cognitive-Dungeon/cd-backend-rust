use cd_map::{Chunk, Region, Tile, TileFlags, CHUNK_SIZE};
use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;
use cd_map::chunk::ChunkBuilder;

fn benchmark_region_access(c: &mut Criterion) {
    // 1. Подготовка данных (SETUP)
    let mut region = Region::new();
    let mut rng = rand::rng();

    // Заполняем регион случайными данными, чтобы "прогреть" память
    // и заставить процессор реально читать данные.
    for ry in 0..32 {
        for rx in 0..32 {
            // В оптимизированной версии это get_or_create_chunk
            // В наивной версии (если API отличается) адаптируйте этот вызов
            let chunk = region.get_or_create_chunk(rx, ry);

            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    chunk.set_tile(
                        x as usize,
                        y as usize,
                        Tile {
                            material: (rng.random::<u16>() % 100) + 1,
                            flags: TileFlags::SOLID,
                            variant: 0,
                        },
                    );
                }
            }
        }
    }

    // Генерируем список случайных координат заранее, чтобы не мерить скорость рандома
    let mut coords = Vec::with_capacity(10_000);
    for _ in 0..10_000 {
        let rx = rng.random_range(0..32);
        let ry = rng.random_range(0..32);
        let cx = rng.random_range(0..16);
        let cy = rng.random_range(0..16);
        coords.push((rx, ry, cx, cy));
    }

    // 2. Бенчмарк чтения (HOT PATH)
    c.bench_function("region_random_read", |b| {
        b.iter(|| {
            // Итерируемся по случайным координатам
            for &(rx, ry, cx, cy) in &coords {
                // black_box запрещает компилятору выкинуть этот код
                // Мы симулируем чтение тайла движком
                if let Some(chunk) = region.get_chunk(rx, ry) {
                    let tile = chunk.get_tile(cx, cy);
                    std::hint::black_box(tile);
                }
            }
        })
    });
}

fn benchmark_chunk_write(c: &mut Criterion) {
    c.bench_function("chunk_fill_palette", |b| {
        b.iter(|| {
            let mut chunk = Chunk::new();
            // Заполняем чанк уникальными тайлами (худший случай для палитры без HashMap)
            for i in 0..255u16 {
                let x = (i % 16) as usize;
                let y = (i / 16) as usize;
                chunk.set_tile(
                    x,
                    y,
                    Tile {
                        material: i + 1,
                        flags: TileFlags::empty(),
                        variant: 0,
                    },
                );
            }
            std::hint::black_box(chunk);
        })
    });
}

fn benchmark_chunk_builder(c: &mut Criterion) {
    c.bench_function("chunk_builder_fill", |b| {
        b.iter(|| {
            // Используем Builder вместо "сырого" чанка
            let mut builder = ChunkBuilder::new();

            for i in 0..255u16 {
                let x = (i % 16) as usize;
                let y = (i / 16) as usize;
                builder.set_tile(x, y, Tile {
                    material: i + 1,
                    flags: TileFlags::empty(),
                    variant: 0
                });
            }
            let chunk = builder.build();
            std::hint::black_box(chunk);
        })
    });
}

criterion_group!(benches, benchmark_region_access, benchmark_chunk_write, benchmark_chunk_builder);
criterion_main!(benches);
