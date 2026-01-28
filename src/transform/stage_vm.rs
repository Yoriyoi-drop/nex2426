use crate::utils::asm_ops;

/// NexVM: A stateful polymorphic virtual machine.
/// Unlike standard VMs, this one preserves state across blocks (Infinite State).
/// This means encryption of byte N depends on all bytes [0..N-1].
pub struct NexVM {
    program: Vec<u8>,
    registers: [u64; 4], // Internal state: [R0, R1, R2, R3]
}

impl NexVM {
    /// Generates a random logic program and initializes internal state.
    pub fn generate(seed: u64, length: usize) -> Self {
        let mut program = Vec::with_capacity(length);
        let mut current_seed = seed;
        
        // 1. Generate Bytecode
        for _ in 0..length {
            current_seed = asm_ops::asm_scramble(current_seed);
            program.push((current_seed % 256) as u8);
        }

        // 2. Initialize Registers (Silicon Shadows)
        let r0 = asm_ops::asm_scramble(seed ^ 0x5555555555555555);
        let r1 = asm_ops::asm_scramble(r0 ^ 0xAAAAAAAAAAAAAAAA);
        let r2 = asm_ops::asm_scramble(r1 ^ 0x3333333333333333);
        let r3 = asm_ops::asm_scramble(r2 ^ 0xCCCCCCCCCCCCCCCC);
        
        Self { 
            program,
            registers: [r0, r1, r2, r3]
        }
    }

    /// Executes the VM on a specific block, mutating the internal state.
    pub fn execute(&mut self, block: u64) -> u64 {
        let mut pc = 0;
        let mut local_accumulator = block;

        while pc < self.program.len() {
            let opcode = self.program[pc] % 10; // Expanded opcodes
            let operand_idx = (self.program[pc] as usize >> 4) % 4;
            let val = self.program[pc + 1] as u64;
            
            match opcode {
                0 => { // Mix with Register
                    self.registers[operand_idx] ^= local_accumulator.rotate_left(val as u32 % 64);
                   local_accumulator = asm_ops::asm_mix(local_accumulator, self.registers[operand_idx]);
                },
                1 => local_accumulator = local_accumulator.wrapping_add(val ^ self.registers[0]),
                2 => local_accumulator ^= val.rotate_left(self.registers[1] as u32 % 64),
                3 => local_accumulator = local_accumulator.wrapping_mul(val | 1), 
                4 => local_accumulator = local_accumulator.rotate_left(val as u32 % 64),
                5 => local_accumulator = !local_accumulator ^ self.registers[2],
                6 => local_accumulator = local_accumulator.swap_bytes(),
                7 => { // Register Cross-Pollination
                    self.registers[0] = asm_ops::asm_scramble(self.registers[0] ^ self.registers[operand_idx]);
                },
                8 => { // Nonlinear feedback
                    let shift = (local_accumulator % 64) as u32;
                    self.registers[3] = self.registers[3].wrapping_add(local_accumulator).rotate_right(shift);
                },
                9 => local_accumulator ^= self.registers[3],
                _ => {}
            }
            
            pc += 2;
        }

        // The final block output is a mix of accumulator and all registers
        let final_state = self.registers[0] ^ self.registers[1] ^ self.registers[2] ^ self.registers[3];
        local_accumulator ^ final_state
    }
}
