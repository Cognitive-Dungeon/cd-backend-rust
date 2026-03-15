// Выравниваем маску по 32 байта для AVX инструкций (хотя используем u64)
#[repr(C, align(32))]
#[derive(Clone, Copy, Debug)]
pub struct BitMask256 {
    pub data: [u64; 4],
}

impl Default for BitMask256 {
    fn default() -> Self {
        Self { data: [0; 4] }
    }
}

#[allow(unsafe_code)]
impl BitMask256 {
    #[inline(always)]
    pub fn set(&mut self, idx: usize, value: bool) {
        // Убираем bounds check, т.к. idx гарантированно приходит из indices[0..255]
        let block = idx >> 6; // idx / 64
        let bit = 1u64 << (idx & 63);

        unsafe {
            let item = self.data.get_unchecked_mut(block);
            if value {
                *item |= bit;
            } else {
                *item &= !bit;
            }
        }
    }

    #[inline(always)]
    pub fn get(&self, idx: usize) -> bool {
        let block = idx >> 6;
        let bit = 1u64 << (idx & 63);
        unsafe { (*self.data.get_unchecked(block) & bit) != 0 }
    }
}
