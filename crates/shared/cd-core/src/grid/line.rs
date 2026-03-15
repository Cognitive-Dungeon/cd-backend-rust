use crate::TilePos;

/// Брезенхэм: дискретная линия от `from` до `to`.
/// `visit` вызывается для каждого тайла включая start и end.
/// Если `visit` возвращает `false` — обход прекращается.
pub fn line(from: TilePos, to: TilePos, mut visit: impl FnMut(TilePos) -> bool) {
    let (mut x0, mut y0) = (from.x, from.y);
    let (x1, y1) = (to.x, to.y);

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if !visit(TilePos::new(x0, y0)) {
            return;
        }
        if x0 == x1 && y0 == y1 {
            return;
        }
        let e2 = err << 1;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// Брезенхэм без начальной точки `from`.
pub fn line_exclusive(from: TilePos, to: TilePos, mut visit: impl FnMut(TilePos) -> bool) {
    let mut first = true;
    line(from, to, |p| {
        if first {
            first = false;
            return true;
        }
        visit(p)
    });
}

/// Луч из `from` в направлении `dir` на `max_len` тайлов.
pub fn ray(from: TilePos, dir: TilePos, max_len: i32, mut visit: impl FnMut(TilePos) -> bool) {
    let mut cur = from;
    for _ in 0..max_len {
        cur += dir;
        if !visit(cur) {
            return;
        }
    }
}
