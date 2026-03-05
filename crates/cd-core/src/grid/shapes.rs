use crate::TilePos;

pub fn for_each_in_radius(center: TilePos, r: i32, mut visit: impl FnMut(TilePos) -> bool) {
    let rr = (r as i64) * (r as i64);
    for dy in -r..=r {
        for dx in -r..=r {
            if (dx as i64) * (dx as i64) + (dy as i64) * (dy as i64) <= rr
                && !visit(center + TilePos::new(dx, dy))
            {
                return;
            }
        }
    }
}

pub fn for_each_in_square(center: TilePos, r: i32, mut visit: impl FnMut(TilePos) -> bool) {
    for dy in -r..=r {
        for dx in -r..=r {
            if !visit(center + (TilePos::new(dx, dy))) {
                return;
            }
        }
    }
}

pub fn for_each_in_diamond(center: TilePos, r: i32, mut visit: impl FnMut(TilePos) -> bool) {
    for dy in -r..=r {
        let limit = r - dy.abs();
        for dx in -limit..=limit {
            if !visit(center + (TilePos::new(dx, dy))) {
                return;
            }
        }
    }
}
