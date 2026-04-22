use crate::utils::asm_ops;

pub fn reduce_blocks(mut blocks: Vec<u64>) -> Vec<u64> {
    // println!("Starting reduction with {} blocks.", blocks.len());
    
    // Target is 8 blocks (512-bit) for Quantum Resistance
    while blocks.len() > 8 {
        let mut next_layer = Vec::new();
        let current_len = blocks.len();
        
        // Special case adjustment for 9 blocks -> 8 logic
        if current_len == 9 {
             if let Some(last) = blocks.pop() {
                 blocks[0] = asm_ops::asm_scramble(blocks[0] ^ last);
             }
             break;
        }

        let mut iter = blocks.into_iter();
        while let Some(a) = iter.next() {
            if let Some(b) = iter.next() {
                next_layer.push(asm_ops::asm_mix(a, b));
            } else {
                next_layer.push(a);
            }
        }
        
        blocks = next_layer;
    }
    
    // Ensure we have exactly 8 blocks
    while blocks.len() < 8 {
        blocks.push(0xDEADBEEFCAFEBABE);
    }
    
    blocks
}
