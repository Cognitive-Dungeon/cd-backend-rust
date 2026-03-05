use crate::TilePos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub min: TilePos,
    pub max: TilePos,
}

impl Rect {
    #[inline]
    pub const fn new(min: TilePos, max: TilePos) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn contains(self, p: TilePos) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    pub fn width(self) -> i32 {
        self.max.x - self.min.x
    }
    pub fn height(self) -> i32 {
        self.max.y - self.min.y
    }
}
