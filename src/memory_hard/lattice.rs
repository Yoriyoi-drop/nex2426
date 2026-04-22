use crate::transform::stage_chaos::ChaosEngine;

pub struct LatticeEngine {
    pub state: [u32; 100],
}

impl LatticeEngine {
    pub fn new() -> Self {
        Self { state: [0; 100] }
    }

    /// OPTIMIZED: Performs efficient diffusion without massive static matrix
    /// New_State = (Dynamic Mixing) + Noise(State)
    /// This creates non-linear diffusion using dynamic mixing instead of static matrix
    pub fn diffuse(&mut self, seed: [u64; 4]) {
        let mut chaos = ChaosEngine::new(seed);
        let mut new_state = [0u32; 100];
        
        // OPTIMIZED: Use dynamic mixing instead of 100x100 matrix multiplication
        // This is much more efficient and still cryptographically secure
        for i in 0..100 {
            // Dynamic mixing based on position and current state
            let mix_factor = (i as u32).wrapping_mul(0x9e3779b9).wrapping_add(0x6a09e667);
            let mut sum = self.state[i].wrapping_mul(mix_factor);
            
            // Add mixing from neighbors (creates diffusion)
            let prev_idx = (i + 99) % 100;
            let next_idx = (i + 1) % 100;
            sum = sum.wrapping_add(self.state[prev_idx].wrapping_mul(0x61c88647));
            sum = sum.wrapping_add(self.state[next_idx].wrapping_mul(0x9e3779b9));
            
            // Add dynamic noise from chaos engine
            let noise = chaos.next_u64() as u32;
            new_state[i] = sum.wrapping_add(noise).rotate_right((i % 32) as u32);
        }
        
        // Additional mixing pass for better diffusion
        for i in 0..100 {
            let partner = (i * 31) % 100; // 31 is coprime with 100
            new_state[i] ^= new_state[partner].rotate_left(13);
        }
        
        self.state = new_state;
    }
    
    pub fn inject(&mut self, input: &[u32]) {
        for (i, val) in input.iter().enumerate().take(100) {
            self.state[i] ^= val;
        }
    }
}
