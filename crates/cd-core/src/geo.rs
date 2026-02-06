use serde::Serialize;
use std::fmt;

/// Упакованная координата (X, Y, Z).
/// Layout: [ Z (12) | Y (26) | X (26) ]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldPos(u64);

impl WorldPos {
    const BITS_Z: u64 = 12;
    const BITS_Y: u64 = 26;
    const BITS_X: u64 = 26;

    const SHIFT_X: u64 = 0;
    const SHIFT_Y: u64 = Self::BITS_X;
    const SHIFT_Z: u64 = Self::BITS_X + Self::BITS_Y;

    const MASK_X: u64 = (1 << Self::BITS_X) - 1;
    const MASK_Y: u64 = (1 << Self::BITS_Y) - 1;
    const MASK_Z: u64 = (1 << Self::BITS_Z) - 1;

    // Смещения для поддержки отрицательных координат
    const OFFSET_X: i32 = 1 << (Self::BITS_X - 1);
    const OFFSET_Y: i32 = 1 << (Self::BITS_Y - 1);
    const OFFSET_Z: i32 = 1 << (Self::BITS_Z - 1);

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        let ux = (x + Self::OFFSET_X) as u64 & Self::MASK_X;
        let uy = (y + Self::OFFSET_Y) as u64 & Self::MASK_Y;
        let uz = (z + Self::OFFSET_Z) as u64 & Self::MASK_Z;

        Self((uz << Self::SHIFT_Z) | (uy << Self::SHIFT_Y) | ux)
    }

    pub fn x(&self) -> i32 {
        let val = (self.0 >> Self::SHIFT_X) & Self::MASK_X;
        (val as i32) - Self::OFFSET_X
    }

    pub fn y(&self) -> i32 {
        let val = (self.0 >> Self::SHIFT_Y) & Self::MASK_Y;
        (val as i32) - Self::OFFSET_Y
    }

    pub fn z(&self) -> i32 {
        let val = (self.0 >> Self::SHIFT_Z) & Self::MASK_Z;
        (val as i32) - Self::OFFSET_Z
    }

    pub fn distance_squared(&self, other: WorldPos) -> i64 {
        let dx = (self.x() - other.x()) as i64;
        let dy = (self.y() - other.y()) as i64;
        dx * dx + dy * dy
    }
}

impl fmt::Debug for WorldPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pos({}, {}, {})", self.x(), self.y(), self.z())
    }
}

impl Serialize for WorldPos {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("WorldPos", 3)?;
        state.serialize_field("x", &self.x())?;
        state.serialize_field("y", &self.y())?;
        state.serialize_field("z", &self.z())?;
        state.end()
    }
}