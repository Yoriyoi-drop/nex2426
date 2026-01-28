use crate::utils::asm_ops;

pub fn convert_to_binary_blocks(parts: Vec<String>) -> Vec<u64> {
    let mut binary_blocks = Vec::new();

    for part in parts {
        let bytes = part.as_bytes();
        let len = bytes.len();
        let mid = len / 2;
        
        // Block 1 from first half
        let mut val1: u64 = 0x123456789ABCDEF0;
        for i in 0..mid {
            val1 = asm_ops::asm_mix(val1, bytes[i] as u64);
        }
        
        // Block 2 from second half
        let mut val2: u64 = 0x0FEDCBA987654321;
        for i in mid..len {
            val2 = asm_ops::asm_mix(val2, bytes[i] as u64);
        }
        
        binary_blocks.push(val1);
        binary_blocks.push(val2);
    }
    
    // Should be 20 blocks
    binary_blocks
}
