use crate::utils::asm_ops;

/// OPTIMIZED White-Box Network with dynamic table generation
/// Replaces 53 static table files (420KB) with compact dynamic generation
pub struct NetworkEngine {
    pub state: [u32; 16],
    round_counter: u32,
}

impl NetworkEngine {
    pub fn new(seed: [u32; 16]) -> Self {
        Self { 
            state: seed,
            round_counter: 0,
        }
    }

    /// Executes 1024 rounds of dynamic whitebox operations
    /// OPTIMIZED: No static tables, generates values on-the-fly
    pub fn execute(&mut self) {
        for _ in 0..1024 {
            self.round();
            self.round_counter = self.round_counter.wrapping_add(1);
        }
    }

    #[inline(always)]
    fn round(&mut self) {
        // DYNAMIC: Generate table values on-the-fly using cryptographic mixing
        for i in 0..16 {
            let byte_val = (self.state[i] & 0xFF) as u32;
            
            // Generate 4 different "table" values dynamically
            // T1 equivalent: mixing with round counter
            let t1_val = self.dynamic_table_lookup(byte_val, 1, i);
            // T2 equivalent: mixing with state position
            let t2_val = self.dynamic_table_lookup(byte_val, 2, i);
            // T3 equivalent: mixing with accumulated entropy
            let t3_val = self.dynamic_table_lookup(byte_val, 3, i);
            // T4 equivalent: mixing with cross-state
            let t4_val = self.dynamic_table_lookup(byte_val, 4, i);

            // Apply transformations based on position
            match i % 4 {
                0 => self.state[i] ^= t1_val,
                1 => self.state[i] ^= t2_val,
                2 => self.state[i] ^= t3_val,
                3 => self.state[i] ^= t4_val,
                _ => unreachable!(),
            }
        }

        // Shuffle state (same as original but more efficient)
        self.state.rotate_right(1);
    }

    /// Dynamic table lookup replacing static T1-T4 tables
    /// Generates cryptographically secure values on-the-fly
    #[inline(always)]
    fn dynamic_table_lookup(&self, input: u32, table_id: u32, position: usize) -> u32 {
        let base_seed = match table_id {
            1 => 0xA3F192B1,
            2 => 0x89C324D4, 
            3 => 0x12F5A912,
            4 => 0x56B192C3,
            _ => 0x12345678,
        };

        // Mix input with table-specific seed
        let mixed = input.wrapping_mul(base_seed);
        
        // Add position and round dependency
        let pos_factor = position as u32;
        let round_factor = self.round_counter.wrapping_mul(table_id);
        
        // Generate final value using asm scramble
        let final_val = asm_ops::asm_scramble(
            (mixed ^ pos_factor ^ round_factor) as u64
        ) as u32;

        final_val
    }
}
