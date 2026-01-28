/// Base58 Encoding implementation (Bitcoin style).
/// Used to compress the final output string to be more human-readable and shorter.
pub mod base58 {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    pub fn encode_blocks(blocks: &[u64]) -> String {
        let mut final_str = String::new();
        for &val in blocks {
            final_str.push_str(&encode_u64(val));
        }
        final_str
    }

    fn encode_u64(mut val: u64) -> String {
        let mut chars = Vec::new();
        if val == 0 { return "1".to_string(); }
        
        while val > 0 {
            let rem = (val % 58) as usize;
            val /= 58;
            chars.push(ALPHABET[rem] as char);
        }
        
        // Pad to fixed width roughly to ensure alignment? 
        // Base58 of u64 max is ~11 chars.
        // Padding isn't strictly necessary for visual hash, but good for parsing.
        // Let's keep it variable for max compression.
        chars.reverse();
        chars.into_iter().collect()
    }
}
