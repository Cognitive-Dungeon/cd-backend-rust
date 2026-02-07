#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct BitMask256([u64; 4]);

impl BitMask256 {
    pub(crate) fn set(&mut self, idx: usize, value: bool) {
        let block = idx >> 6;
        let bit = 1u64 << (idx & 63);
        if value {
            self.0[block] |= bit;
        } else {
            self.0[block] &= !bit;
        }
    }

    pub(crate) fn get(&self, idx: usize) -> bool {
        let block = idx >> 6;
        let bit = 1u64 << (idx & 63);
        (self.0[block] & bit) != 0
    }

    // Быстрый merge (OR)
    pub(crate) fn merge(&mut self, other: &BitMask256) {
        for i in 0..4 {
            self.0[i] |= other.0[i];
        }
    }
}