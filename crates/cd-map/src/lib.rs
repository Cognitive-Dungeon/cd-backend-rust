pub mod tile;
pub mod chunk;
pub mod grid; // Spatial Index
pub mod world;
pub mod region;
mod bitmask;
mod sparse_chunk;
mod shard;

pub use tile::{Tile, TileFlags};
pub use chunk::Chunk;
pub use sparse_chunk::SparseChunk;
pub use region::Region;
pub use world::WorldMap;
pub use grid::SpatialGrid;

// Константы размера чанка
pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_SHIFT: i32 = 4;
pub const CHUNK_MASK: i32 = 15;
pub const CHUNK_AREA: usize = (CHUNK_SIZE * CHUNK_SIZE) as usize;

// Размер ячейки сетки (Bucket).
// 16 - совпадает с размером чанка. Это удобно для маппинга.
const CELL_SIZE: i32 = CHUNK_SIZE;

// 32x32 чанка = 1024 чанка в регионе.
// 32 * 16 = 512 тайлов сторона региона
pub const REGION_SHIFT: i32 = 5;
pub const REGION_SIZE: usize = 1 << REGION_SHIFT; // 32
pub const REGION_MASK: i32 = (REGION_SIZE as i32) - 1;
pub const REGION_AREA: usize = REGION_SIZE * REGION_SIZE;

// Количество шардов для многопоточного доступа
pub const SHARD_COUNT: usize = 64;
pub const SHARD_MASK: usize = SHARD_COUNT - 1;

