// math/fractions.rs
pub trait BrpFractions {
    fn half_ceil(self) -> Self;
    fn fifth_ceil(self) -> Self;
    fn twentieth_ceil(self) -> Self;
}
