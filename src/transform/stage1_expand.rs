use crate::utils::asm_ops;
use std::io::Read;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";

/// Expands input from any Reader (File, String, Stdin) into initial entropy parts.
/// Uses buffered reading to support large files without memory issues.
pub fn expand_input<R: Read>(mut reader: R, key: &str) -> Vec<String> {
    let mut parts = Vec::new();
    
    // 1. Generate a seed from the input + key (Salt)
    let mut seed: u64 = 0;
    
    // Mix Key first (Salt)
    for (i, byte) in key.bytes().enumerate() {
        seed = asm_ops::asm_scramble(seed ^ (byte as u64));
        seed = seed.wrapping_add((i * 1999) as u64); 
    }

    // Mix Input (Streaming)
    let mut global_idx: u64 = 0;
    let mut buffer = [0u8; 8192]; // 8KB Buffer

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(n) => {
                for i in 0..n {
                    let byte = buffer[i];
                    seed = asm_ops::asm_scramble(seed ^ (byte as u64));
                    seed = seed.wrapping_add(global_idx);
                    global_idx += 1;
                }
            }
            Err(_) => break, // Error handling typically defaults to stop mixing
        }
    }
    
    // 2. Generate 10 parts
    for _ in 0..10 {
        let mut part = String::new();
        // Generate a 12-char random string for each part
        for _ in 0..12 {
            seed = asm_ops::asm_pseudo_rand(seed);
            let idx = (seed as usize) % CHARSET.len();
            part.push(CHARSET[idx] as char);
        }
        parts.push(part);
    }
    
    parts
}
