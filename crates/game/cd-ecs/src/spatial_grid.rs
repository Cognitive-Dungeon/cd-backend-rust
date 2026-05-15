use ahash::HashMap;
use cd_core::{ObjectGuid, WorldPos};

use crate::InstanceId;

// Размер ячейки сетки (Bucket).
// 16 - совпадает с размером чанка. Это удобно для маппинга.
const CELL_SIZE: i32 = cd_map::CHUNK_SIZE;

/// Пространственный индекс.
/// Позволяет быстро отвечать на вопрос "кто находится в точке X,Y?".
#[derive(Debug, Default)]
pub struct SpatialGrid {
    // Ключ: (InstanceId, x / 16, y / 16)
    // Значение - список ID сущностей
    buckets: HashMap<(InstanceId, i32, i32), Vec<ObjectGuid>>,
}

impl SpatialGrid {
    pub fn new() -> Self {
        Self::default()
    }

    /// Конвертирует мировые координаты в ключ ячейки
    fn get_key(instance: InstanceId, pos: WorldPos) -> (InstanceId, i32, i32) {
        (instance, pos.x() / CELL_SIZE, pos.y() / CELL_SIZE)
    }

    pub fn insert(&mut self, instance: InstanceId, entity: ObjectGuid, pos: WorldPos) {
        let key = Self::get_key(instance, pos);
        self.buckets.entry(key).or_default().push(entity);
    }

    pub fn remove(&mut self, instance: InstanceId, entity: ObjectGuid, pos: WorldPos) {
        let key = Self::get_key(instance, pos);
        if let Some(list) = self.buckets.get_mut(&key) {
            list.retain(|&e| e != entity);
        }
    }

    pub fn move_entity(
        &mut self,
        instance: InstanceId,
        entity: ObjectGuid,
        old_pos: WorldPos,
        new_pos: WorldPos,
    ) {
        let old_key = Self::get_key(instance, old_pos);
        let new_key = Self::get_key(instance, new_pos);

        if old_key == new_key {
            return;
        }

        self.remove(instance, entity, old_pos);
        self.insert(instance, entity, new_pos);
    }

    /// Возвращает список сущностей в ячейке, где находится pos
    pub fn query_bucket(&self, instance: InstanceId, pos: WorldPos) -> &[ObjectGuid] {
        let key = Self::get_key(instance, pos);
        self.buckets.get(&key).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Возвращает все сущности в квадрате вокруг точки (bucket-level, не точный радиус).
    /// Достаточно для AoE-эффектов и ИИ-восприятия.
    pub fn query_radius(
        &self,
        instance: InstanceId,
        center: WorldPos,
        radius: i32,
    ) -> Vec<ObjectGuid> {
        let center_key = Self::get_key(instance, center);
        let bucket_radius = (radius / CELL_SIZE) + 1;
        let mut result = Vec::new();

        for dy in -bucket_radius..=bucket_radius {
            for dx in -bucket_radius..=bucket_radius {
                let key = (instance, center_key.1 + dx, center_key.2 + dy);
                if let Some(entities) = self.buckets.get(&key) {
                    result.extend_from_slice(entities);
                }
            }
        }
        result
    }
}
