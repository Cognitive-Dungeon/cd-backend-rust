//! Целочисленная математика BRP.
//! Правила BRP почти всегда требуют округления вверх при делении (ceil).
//! Формула ceil для целых чисел: (a + b - 1) / b

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
        self.div_ceil(2)
    }

    #[inline(always)]
    fn fifth_ceil(self) -> Self {
        self.div_ceil(5)
    }

    #[inline(always)]
    fn twentieth_ceil(self) -> Self {
        self.div_ceil(20)
    }
}

// Также реализуем для i32 (понадобится для комплексных расчетов штрафов)
impl BrpFractions for i32 {
    #[inline(always)]
    fn half_ceil(self) -> Self {
        if self >= 0 { (self + 1) / 2 } else { self / 2 }
    }

    #[inline(always)]
    fn fifth_ceil(self) -> Self {
        if self >= 0 { (self + 4) / 5 } else { self / 5 }
    }

    #[inline(always)]
    fn twentieth_ceil(self) -> Self {
        if self >= 0 {
            (self + 19) / 20
        } else {
            self / 20
        }
    }
}
