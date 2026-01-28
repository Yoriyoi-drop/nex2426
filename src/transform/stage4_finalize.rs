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
        
        let b = blocks.clone(); // Snapshot for mixing

        blocks[0] = asm_ops::asm_scramble(b[0] ^ b[1] ^ rc);
        blocks[1] = asm_ops::asm_scramble(b[1] ^ b[2]);
        blocks[2] = asm_ops::asm_scramble(b[2] ^ b[3] ^ rc);
        blocks[3] = asm_ops::asm_scramble(b[3] ^ b[4]);
        blocks[4] = asm_ops::asm_scramble(b[4] ^ b[5] ^ rc);
        blocks[5] = asm_ops::asm_scramble(b[5] ^ b[6]);
        blocks[6] = asm_ops::asm_scramble(b[6] ^ b[7] ^ rc);
        blocks[7] = asm_ops::asm_scramble(b[7] ^ b[0]); // Wrap
    }

    blocks
}
