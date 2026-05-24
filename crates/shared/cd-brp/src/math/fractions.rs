//! Целочисленная математика BRP.
//! Правила BRP почти всегда требуют округления вверх при делении (ceil).
//! Формула ceil для целых чисел: (a + b - 1) / b

macro_rules! impl_brp_fractions_mod {
    // Беззнаковые типы: div_ceil встроен
    (unsigned $ty:ty => mod $mod_name:ident) => {
        pub mod $mod_name {
            /// Половина с округлением вверх
            pub const fn half_ceil(v: $ty) -> $ty {
                v.div_ceil(2)
            }
            /// Пятая часть (20%) с округлением вверх
            pub const fn fifth_ceil(v: $ty) -> $ty {
                v.div_ceil(5)
            }
            /// Двадцатая часть (5%) с округлением вверх
            pub const fn twentieth_ceil(v: $ty) -> $ty {
                v.div_ceil(20)
            }
        }
    };

    // Знаковые типы: ceil для отрицательных — это просто усечение к нулю
    (signed $ty:ty => mod $mod_name:ident) => {
        pub mod $mod_name {
            pub const fn half_ceil(v: $ty) -> $ty {
                if v >= 0 { (v + 1) / 2 } else { v / 2 }
            }
            pub const fn fifth_ceil(v: $ty) -> $ty {
                if v >= 0 { (v + 4) / 5 } else { v / 5 }
            }
            pub const fn twentieth_ceil(v: $ty) -> $ty {
                if v >= 0 { (v + 19) / 20 } else { v / 20 }
            }
        }
    };
}

impl_brp_fractions_mod!(unsigned u16 => mod frac_u16);
impl_brp_fractions_mod!(unsigned u32 => mod frac_u32);
impl_brp_fractions_mod!(signed   i32 => mod frac_i32);

pub trait BrpFractions {
    /// Половина значения с округлением вверх
    fn half_ceil(self) -> Self;
    /// Пятая часть (20%) с округлением вверх
    fn fifth_ceil(self) -> Self;
    /// Двадцатая часть (5%) с округлением вверх
    fn twentieth_ceil(self) -> Self;
}

// Реализуем для u16, так как это основной тип для статов и навыков в нашем ядре
impl BrpFractions for u16 {
    #[inline(always)]
    fn half_ceil(self) -> Self {
        frac_u16::half_ceil(self)
    }
    #[inline(always)]
    fn fifth_ceil(self) -> Self {
        frac_u16::fifth_ceil(self)
    }
    #[inline(always)]
    fn twentieth_ceil(self) -> Self {
        frac_u16::twentieth_ceil(self)
    }
}

// Также реализуем для i32 (понадобится для комплексных расчетов штрафов)
impl BrpFractions for i32 {
    #[inline(always)]
    fn half_ceil(self) -> Self {
        frac_i32::half_ceil(self)
    }
    #[inline(always)]
    fn fifth_ceil(self) -> Self {
        frac_i32::fifth_ceil(self)
    }
    #[inline(always)]
    fn twentieth_ceil(self) -> Self {
        frac_i32::twentieth_ceil(self)
    }
}
