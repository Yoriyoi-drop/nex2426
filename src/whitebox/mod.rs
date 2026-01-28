pub mod tables;
pub mod network;

use tables::t1::T1_BOX;

pub struct WhiteBoxEngine {
    state: [u32; 16],
}

impl WhiteBoxEngine {
    pub fn new() -> Self {
        Self {
            state: [0; 16],
        }
    }

    #[inline(always)]
    pub fn apply_round(&mut self) {
        // Real transformation using the massive tables
        // In a real whitebox, every step is a table look up
        for i in 0..16 {
            let val = self.state[i] as usize;
            // Apply T1 box transformation (simulating a mix of S-Box + ShiftRows + MixColumns)
            // Masking index to avoid bounds check in unsafe block, or just standard access
            let idx = val & 0xFF; 
            self.state[i] = T1_BOX[idx];
        }
    }
}
