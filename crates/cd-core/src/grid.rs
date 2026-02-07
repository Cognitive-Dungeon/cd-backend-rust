use crate::geo::WorldPos;
use serde::{Deserialize, Serialize};

/// Направления движения.
/// В Go: enums/tiles.go
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Direction {
    None = 0,
    North,
    South,
    West,
    East,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
    Up,
    Down,
}

impl Direction {
    /// Возвращает дельту координат (dx, dy, dz).
    /// В Go: func (d Direction) Offset() ...
    pub fn offset(&self) -> (i32, i32, i32) {
        match self {
            Self::None => (0, 0, 0),
            Self::North => (0, -1, 0),
            Self::South => (0, 1, 0),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
            Self::NorthWest => (-1, -1, 0),
            Self::NorthEast => (1, -1, 0),
            Self::SouthWest => (-1, 1, 0),
            Self::SouthEast => (1, 1, 0),
            Self::Up => (0, 0, 1),
            Self::Down => (0, 0, -1),
        }
    }

    /// Список ортогональных соседей (4-way).
    pub const ORTHOGONAL: [Direction; 4] = [
        Self::North, Self::South, Self::West, Self::East
    ];

    /// Список всех 2D соседей (8-way).
    pub const ALL_2D: [Direction; 8] = [
        Self::North, Self::South, Self::West, Self::East,
        Self::NorthWest, Self::NorthEast, Self::SouthWest, Self::SouthEast
    ];
}

/// Трейт для геометрических операций.
/// Мы не пишем методы прямо в WorldPos, чтобы разделить хранение данных и логику.
pub trait GridLogic {
    fn shift(&self, dir: Direction) -> Self;
    fn distance_squared(&self, other: Self) -> i64;
    fn manhattan_distance(&self, other: Self) -> i32;
    fn is_in_radius(&self, center: Self, radius: i32) -> bool;
}

impl GridLogic for WorldPos {
    fn shift(&self, dir: Direction) -> Self {
        let (dx, dy, dz) = dir.offset();
        WorldPos::new(self.x() + dx, self.y() + dy, self.z() + dz)
    }

    fn distance_squared(&self, other: Self) -> i64 {
        let dx = (self.x() - other.x()) as i64;
        let dy = (self.y() - other.y()) as i64;
        dx * dx + dy * dy
    }

    fn manhattan_distance(&self, other: Self) -> i32 {
        (self.x() - other.x()).abs() + (self.y() - other.y()).abs()
    }

    fn is_in_radius(&self, center: Self, radius: i32) -> bool {
        self.distance_squared(center) <= (radius as i64 * radius as i64)
    }
}