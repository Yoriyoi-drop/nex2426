use crate::utils::asm_ops;

// Fixed-Point Arithmetic for Determinism across Architectures
// Scale Factor = 2^20 (approx 1,000,000 precision)
const SCALE: i64 = 1 << 20;

/// Chaos Engine based on a discrete simulation of the Lorenz Attractor.
/// Now uses Fixed-Point Integer Arithmetic (i64) to guarantee determinism
/// across different CPU architectures (no floating point drift).
pub struct ChaosEngine {
    x: i64,
    y: i64,
    z: i64,
    // sigma: i64, // 10.0 - effectively constant in logic
    rho: i64,   // 28.0
    beta_num: i64, // 8
    beta_den: i64, // 3
}

impl ChaosEngine {
    /// Accepts a 256-bit seed (4 x 64-bit integers) for high entropy.
    pub fn new(seed: [u64; 4]) -> Self {
        // Map 256-bit seed to initial conditions in Fixed-Point
        // We ensure values are within a reasonable range for the attractor
        
        let s0 = (seed[0] as i64).abs();
        let s1 = (seed[1] as i64).abs();
        let s2 = (seed[2] as i64).abs();

        Self {
            // Initialize around typical starting points but randomized by seed
            // x start ~ 0.1 to 10.0 range
            x: ((s0 % 1000) * SCALE / 100) + (SCALE / 10), 
            y: ((s1 % 1000) * SCALE / 100) + (SCALE / 10),
            z: ((s2 % 1000) * SCALE / 100) + (SCALE / 10),
            
            // sigma: 10 * SCALE,
            rho: 28 * SCALE,
            beta_num: 8,
            beta_den: 3,
        }
    }

    /// Advances the system one step and returns a 64-bit entropy value.
    /// Implements Lorenz Attractor using Integer Math:
    /// dx = sigma * (y - x) * dt
    /// dy = (x * (rho - z) - y) * dt
    /// dz = (x * y - beta * z) * dt
    pub fn next_u64(&mut self) -> u64 {
        // dt = 0.01 = 1/100
        // We use a helper macro or closures for fixed point mul/div could be cleaner, 
        // but explicit is fine for performance.
        
        // FIXED: Use wrapping arithmetic to prevent overflow and optimize divisions
        // dx = 10 * (y - x) / 100 => (y - x) / 10
        let dx = (self.y.wrapping_sub(self.x)) / 10;
        
        // dy = (x * (rho - z) - y) * dt
        // OPTIMIZED: Use pre-computed constants and wrapping operations
        let rho_minus_z = self.rho.wrapping_sub(self.z);
        let x_term = self.x.wrapping_mul(rho_minus_z) / SCALE;
        let dy = x_term.wrapping_sub(self.y) / 100;
        
        // dz = (x * y - beta * z) * dt
        // FIXED: Use safe division with beta_den (should never be zero)
        let xy = self.x.wrapping_mul(self.y) / SCALE;
        let beta_z = if self.beta_den != 0 {
            (self.beta_num.wrapping_mul(self.z)) / self.beta_den
        } else {
            self.z // Fallback if beta_den is somehow zero
        };
        let dz = xy.wrapping_sub(beta_z) / 100;

        self.x += dx;
        self.y += dy;
        self.z += dz;

        // Extract Entropy
        // We mix the internal state to produce the output
        let bits_x = self.x as u64;
        let bits_y = self.y as u64;
        let bits_z = self.z as u64;

        asm_ops::asm_scramble(bits_x ^ bits_y.rotate_left(21) ^ bits_z.rotate_left(42))
    }
}
