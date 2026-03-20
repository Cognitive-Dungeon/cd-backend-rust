mod bitmask;
pub mod chunk;
pub mod region;
mod shard;
mod sparse_chunk;
pub mod tile;
pub mod world;

pub use chunk::Chunk;
pub use region::Region;
pub use sparse_chunk::SparseChunk;
pub use tile::{MaterialID, Tile, TileFlags};
pub use world::WorldMap;

// Константы размера чанка
pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_SHIFT: i32 = 4;
pub const CHUNK_MASK: i32 = 15;
pub const CHUNK_AREA: usize = (CHUNK_SIZE * CHUNK_SIZE) as usize;

// 32x32 чанка = 1024 чанка в регионе.
// 32 * 16 = 512 тайлов сторона региона
pub const REGION_SHIFT: i32 = 5;
pub const REGION_SIZE: usize = 1 << REGION_SHIFT; // 32
pub const REGION_MASK: i32 = (REGION_SIZE as i32) - 1;
pub const REGION_AREA: usize = REGION_SIZE * REGION_SIZE;

// Количество шардов для многопоточного доступа
pub const SHARD_COUNT: usize = 64;
pub const SHARD_MASK: usize = SHARD_COUNT - 1;
