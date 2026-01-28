use crate::transform::stage_chaos::ChaosEngine;
use super::constants::LATTICE_MATRIX;

pub struct LatticeEngine {
    pub state: [u32; 100],
}

impl LatticeEngine {
    pub fn new() -> Self {
        Self { state: [0; 100] }
    }

    /// Performs a Matrix-Vector multiplication with Dynamic Noise (State-Dependent).
    /// New_State = (Matrix * Old_State) + Noise(State)
    /// This effectively creates a non-linear diffusion layer.
    pub fn diffuse(&mut self, seed: [u64; 4]) {
        let mut chaos = ChaosEngine::new(seed);
        let mut new_state = [0u32; 100];
        
        for row in 0..100 {
            let mut sum: u32 = 0;
            for col in 0..100 {
                // Using Static Matrix for mixing (Structure)
                let matrix_val = LATTICE_MATRIX[row * 100 + col];
                let vec_val = self.state[col];
                
                sum = sum.wrapping_add(matrix_val.wrapping_mul(vec_val));
            }
            // Add Dynamic Noise derived from the State/Seed
            // This prevents "Elimination" attacks because the error term is not constant.
            let noise = chaos.next_u64() as u32;
            new_state[row] = sum.wrapping_add(noise);
        }
        
        self.state = new_state;
    }
    
    pub fn inject(&mut self, input: &[u32]) {
        for (i, val) in input.iter().enumerate().take(100) {
            self.state[i] ^= val;
        }
    }
}
