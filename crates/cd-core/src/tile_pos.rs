use crate::WorldPos;

/// Локальная 2D позиция внутри чанка/региона.
///
/// Используется для локальной математики: LOS, pathfinding, AoE, соседи.
/// Не путать с `WorldPos` — глобальной адресацией мира.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TilePos {
    pub x: i32,
    pub y: i32,
}

impl TilePos {
    #[inline]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    // ── Конвертация ───────────────────────────────────────────────────────

    /// Локальная позиция + origin чанка → глобальный WorldPos.
    #[inline]
    pub fn to_world_pos(self, chunk_origin: WorldPos) -> WorldPos {
        WorldPos::new(
            chunk_origin.x() + self.x,
            chunk_origin.y() + self.y,
            chunk_origin.z(),
        )
    }

    // ── Distance ──────────────────────────────────────────────────────────

    /// Квадрат евклидова расстояния — без sqrt.
    /// Для горячих циклов: сравнивай с `r * r`.
    #[inline]
    pub fn distance_squared(self, other: Self) -> i64 {
        let d = self - other;
        d.x as i64 * d.x as i64 + d.y as i64 * d.y as i64
    }

    /// Евклидово расстояние. Только для UI — не для симуляции.
    #[inline]
    pub fn euclidean_distance(self, other: Self) -> f32 {
        (self.distance_squared(other) as f64).sqrt() as f32
    }

    /// Манхэттенское расстояние: |dx| + |dy|.
    #[inline]
    pub fn manhattan_distance(self, other: Self) -> i32 {
        let d = self - other;
        d.x.abs() + d.y.abs()
    }

    /// Расстояние Чебышёва: max(|dx|, |dy|).
    #[inline]
    pub fn chebyshev_distance(self, other: Self) -> i32 {
        let d = self - other;
        d.x.abs().max(d.y.abs())
    }

    /// Октильная эвристика для A* (8-направленное движение).
    /// Ортогональный шаг = 10, диагональный = 14.
    #[inline]
    pub fn octile_distance(self, other: Self) -> i32 {
        let d = self - other;
        let (dx, dy) = (d.x.abs(), d.y.abs());
        let (mn, mx) = if dx < dy { (dx, dy) } else { (dy, dx) };
        14 * mn + 10 * (mx - mn)
    }

    /// L1 — алиас для manhattan_distance.
    #[inline]
    pub fn l1(self, o: Self) -> i32 {
        self.manhattan_distance(o)
    }
    /// L2² — алиас для distance_squared.
    #[inline]
    pub fn l2_squared(self, o: Self) -> i64 {
        self.distance_squared(o)
    }
    /// L∞ — алиас для chebyshev_distance.
    #[inline]
    pub fn l_inf(self, o: Self) -> i32 {
        self.chebyshev_distance(o)
    }

    #[inline]
    pub fn in_manhattan_range(self, center: Self, r: i32) -> bool {
        self.manhattan_distance(center) <= r
    }

    #[inline]
    pub fn in_chebyshev_range(self, center: Self, r: i32) -> bool {
        self.chebyshev_distance(center) <= r
    }

    // ── Shapes ────────────────────────────────────────────────────────────

    #[inline]
    pub fn in_radius(self, center: Self, r: i32) -> bool {
        self.distance_squared(center) <= (r as i64) * (r as i64)
    }

    #[inline]
    pub fn in_square(self, center: Self, r: i32) -> bool {
        let d = self - center;
        d.x.abs() <= r && d.y.abs() <= r
    }

    #[inline]
    pub fn in_diamond(self, center: Self, r: i32) -> bool {
        let d = self - center;
        d.x.abs() + d.y.abs() <= r
    }

    /// Ближайшая позиция из слайса.
    /// Возвращает `(index, distance_squared)` или `None` если слайс пуст.
    /// Оптимизировано под LLVM auto-vectorization через `chunks_exact(4)`.
    pub fn find_nearest(self, targets: &[Self]) -> Option<(usize, i64)> {
        if targets.is_empty() {
            return None;
        }

        let (sx, sy) = (self.x as i64, self.y as i64);
        let mut min_dist = i64::MAX;
        let mut min_idx = 0usize;
        let mut i = 0usize;

        let chunks = targets.chunks_exact(4);
        let remainder = chunks.remainder();

        for chunk in chunks {
            let (p0, p1, p2, p3) = (chunk[0], chunk[1], chunk[2], chunk[3]);
            let d0 = {
                let (dx, dy) = (sx - p0.x as i64, sy - p0.y as i64);
                dx * dx + dy * dy
            };
            let d1 = {
                let (dx, dy) = (sx - p1.x as i64, sy - p1.y as i64);
                dx * dx + dy * dy
            };
            let d2 = {
                let (dx, dy) = (sx - p2.x as i64, sy - p2.y as i64);
                dx * dx + dy * dy
            };
            let d3 = {
                let (dx, dy) = (sx - p3.x as i64, sy - p3.y as i64);
                dx * dx + dy * dy
            };

            if d0 < min_dist {
                min_dist = d0;
                min_idx = i;
            }
            if d1 < min_dist {
                min_dist = d1;
                min_idx = i + 1;
            }
            if d2 < min_dist {
                min_dist = d2;
                min_idx = i + 2;
            }
            if d3 < min_dist {
                min_dist = d3;
                min_idx = i + 3;
            }
            i += 4;
        }

        for (j, &p) in remainder.iter().enumerate() {
            let d = self.distance_squared(p);
            if d < min_dist {
                min_dist = d;
                min_idx = i + j;
            }
        }

        Some((min_idx, min_dist))
    }
}

pub const TILE_ZERO: TilePos = TilePos::new(0, 0);

impl std::fmt::Display for TilePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl std::ops::Add for TilePos {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for TilePos {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Neg for TilePos {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl std::ops::AddAssign for TilePos {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl std::ops::SubAssign for TilePos {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl std::ops::Mul<i32> for TilePos {
    type Output = Self;
    #[inline]
    fn mul(self, scalar: i32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl std::ops::Mul<TilePos> for i32 {
    type Output = TilePos;
    #[inline]
    fn mul(self, pos: TilePos) -> TilePos {
        TilePos {
            x: pos.x * self,
            y: pos.y * self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ops() {
        let a = TilePos::new(3, 4);
        let b = TilePos::new(1, 2);
        assert_eq!(a + b, TilePos::new(4, 6));
        assert_eq!(a - b, TilePos::new(2, 2));
        assert_eq!(-a, TilePos::new(-3, -4));
        assert_eq!(a * 3, TilePos::new(9, 12));

        let mut c = a;
        c += b;
        assert_eq!(c, TilePos::new(4, 6));
    }

    #[test]
    fn test_distances() {
        let a = TilePos::new(0, 0);
        let b = TilePos::new(3, 4);
        assert_eq!(a.distance_squared(b), 25);
        assert_eq!(a.manhattan_distance(b), 7);
        assert_eq!(a.chebyshev_distance(b), 4);
        assert_eq!(a.octile_distance(b), 52); // 14*3 + 10*1
        // симметрия
        assert_eq!(a.distance_squared(b), b.distance_squared(a));
    }

    #[test]
    fn test_shapes() {
        let c = TilePos::new(0, 0);
        assert!(TilePos::new(3, 4).in_radius(c, 5));
        assert!(!TilePos::new(3, 4).in_radius(c, 4));
        assert!(TilePos::new(1, 1).in_diamond(c, 2));
        assert!(!TilePos::new(2, 2).in_diamond(c, 2));
        assert!(TilePos::new(2, 2).in_square(c, 2));
    }

    #[test]
    fn test_find_nearest() {
        let origin = TilePos::new(0, 0);
        let targets = vec![
            TilePos::new(10, 10),
            TilePos::new(3, 4), // dist²=25 ← ближайшая
            TilePos::new(7, 7),
        ];
        let (idx, dist_sq) = origin.find_nearest(&targets).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(dist_sq, 25);
        assert!(origin.find_nearest(&[]).is_none());
    }

    #[test]
    fn test_world_pos_roundtrip() {
        use crate::WorldPos;
        let origin = WorldPos::new(32, 64, 1);
        let tile = TilePos::new(5, 3);
        let world = tile.to_world_pos(origin);
        assert_eq!(world.x(), 37);
        assert_eq!(world.y(), 67);
        assert_eq!(world.to_tile_pos(origin), tile);
    }
}
