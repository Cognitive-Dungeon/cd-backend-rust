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
    pub const ORTHOGONAL: [Direction; 4] = [Self::North, Self::South, Self::West, Self::East];

    /// Список всех 2D соседей (8-way).
    pub const ALL_2D: [Direction; 8] = [
        Self::North,
        Self::South,
        Self::West,
        Self::East,
        Self::NorthWest,
        Self::NorthEast,
        Self::SouthWest,
        Self::SouthEast,
    ];

    /// Конвертировать направление в TilePos-смещение для локальной математики.
    ///
    /// Позволяет использовать Direction в pathfinding и LOS без ручного маппинга:
    /// ```rust
    /// use cd_core::{Direction, TilePos};
    /// let pos = TilePos::new(0, 0);
    /// let neighbor = pos + Direction::North.to_tile_pos();
    /// assert_eq!(neighbor, TilePos::new(0, -1));
    /// ```
    pub fn to_tile_pos(self) -> crate::TilePos {
        let (dx, dy, _) = self.offset();
        crate::TilePos::new(dx, dy)
    }
}

pub mod line;
pub mod neighbors;
pub mod rect;
pub mod shapes;
