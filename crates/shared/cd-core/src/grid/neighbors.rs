use crate::TilePos;

pub const NEIGHBORS_4: [TilePos; 4] = [
    TilePos::new(1, 0),
    TilePos::new(-1, 0),
    TilePos::new(0, 1),
    TilePos::new(0, -1),
];

pub const NEIGHBORS_8: [TilePos; 8] = [
    TilePos::new(1, 0),
    TilePos::new(-1, 0),
    TilePos::new(0, 1),
    TilePos::new(0, -1),
    TilePos::new(1, 1),
    TilePos::new(1, -1),
    TilePos::new(-1, 1),
    TilePos::new(-1, -1),
];

pub const NEIGHBORS_DIAGONAL: [TilePos; 4] = [
    TilePos::new(1, 1),
    TilePos::new(1, -1),
    TilePos::new(-1, 1),
    TilePos::new(-1, -1),
];

pub const COST_ORTHOGONAL: i32 = 10;
pub const COST_DIAGONAL: i32 = 14;

#[inline]
pub fn for_each_neighbor_4(p: TilePos, mut f: impl FnMut(TilePos)) {
    for &d in &NEIGHBORS_4 {
        f(p + d);
    }
}

#[inline]
pub fn for_each_neighbor_8(p: TilePos, mut f: impl FnMut(TilePos)) {
    for &d in &NEIGHBORS_8 {
        f(p + d);
    }
}

#[inline]
pub fn for_each_neighbor_4_with_cost(p: TilePos, mut f: impl FnMut(TilePos, i32)) {
    for &d in &NEIGHBORS_4 {
        f(p + d, COST_ORTHOGONAL);
    }
}

#[inline]
pub fn for_each_neighbor_8_with_cost(p: TilePos, mut f: impl FnMut(TilePos, i32)) {
    for &d in &NEIGHBORS_4 {
        f(p + d, COST_ORTHOGONAL);
    }
    for &d in &NEIGHBORS_DIAGONAL {
        f(p + d, COST_DIAGONAL);
    }
}
