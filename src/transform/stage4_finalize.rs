use crate::utils::asm_ops;

/// Stage 4: Cross-Mixes the final 4 blocks to ensure interdependence.
/// This prevents an attacker from isolating one partial block.
pub fn finalize_blocks(mut blocks: Vec<u64>) -> Vec<u64> {
    if blocks.len() != 8 {
        return blocks;
    }

    // println!("Starting Stage 4: Cross-Block Mixing (10 Rounds)...");

    // We use a Feistel-like mixing network for 10 rounds
    for round in 0..10 {
        // Round constant
        let rc = 0x9375CA02113 ^ (round as u64);
        
        // OPTIMIZED: Use individual values to avoid borrowing conflicts
        let b0 = blocks[0];
        let b1 = blocks[1];
        let b2 = blocks[2];
        let b3 = blocks[3];
        let b4 = blocks[4];
        let b5 = blocks[5];
        let b6 = blocks[6];
        let b7 = blocks[7];

        blocks[0] = asm_ops::asm_scramble(b0 ^ b1 ^ rc);
        blocks[1] = asm_ops::asm_scramble(b1 ^ b2);
        blocks[2] = asm_ops::asm_scramble(b2 ^ b3 ^ rc);
        blocks[3] = asm_ops::asm_scramble(b3 ^ b4);
        blocks[4] = asm_ops::asm_scramble(b4 ^ b5 ^ rc);
        blocks[5] = asm_ops::asm_scramble(b5 ^ b6);
        blocks[6] = asm_ops::asm_scramble(b6 ^ b7 ^ rc);
        blocks[7] = asm_ops::asm_scramble(b7 ^ b0); // Wrap
    }

    blocks
}
