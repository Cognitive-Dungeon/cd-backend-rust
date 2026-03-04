use cd_map::{Chunk, TileFlags, CHUNK_AREA};
use serde::{Deserialize, Serialize};

/// JSON-представление чанка.
/// Намеренно плоское: палитра u32 + индексы u8.
/// При смене формата — меняем только этот файл.
#[derive(Serialize, Deserialize)]
pub struct ChunkDto {
    pub palette: Vec<u32>,
    pub indices: Vec<u8>,
}

impl ChunkDto {
    pub fn from_chunk(chunk: &Chunk) -> Self {
        let palette = chunk.palette[..chunk.palette_len as usize].to_vec();
        let indices = chunk.indices.to_vec();
        Self { palette, indices }
    }

    pub fn into_chunk(self) -> Result<Chunk, String> {
        if self.indices.len() != CHUNK_AREA {
            return Err(format!(
                "invalid indices length: expected {}, got {}",
                CHUNK_AREA,
                self.indices.len()
            ));
        }
        if self.palette.len() > 256 {
            return Err(format!("palette too large: {}", self.palette.len()));
        }

        let mut chunk = Chunk::new();

        // Восстанавливаем палитру
        chunk.palette_len = self.palette.len() as u8;
        for (i, &packed) in self.palette.iter().enumerate() {
            chunk.palette[i] = packed;
        }

        // Восстанавливаем индексы
        chunk.indices.copy_from_slice(&self.indices);

        // Перестраиваем битовые маски из данных
        chunk.rebuild_masks();

        Ok(chunk)
    }
}