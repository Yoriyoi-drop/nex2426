use crate::standards::hmac::HmacNex;

/// Industry Standard Key Derivation Function (PBKDF2-style)
/// Uses HMAC-NEX as the pseudo-random function.

pub fn derivate_key(password: &[u8], salt: &[u8], iterations: u32, length: usize) -> Vec<u8> {
    let mut derived_key = Vec::with_capacity(length);
    let hmac = HmacNex::new(password).unwrap_or_else(|_| panic!("Failed to create HMAC"));
    let chunks = (length as f64 / 64.0).ceil() as u32;

    for i in 1..=chunks {
        // F(Password, Salt, c, i) = U1 ^ U2 ^ ... ^ Uc
        // U1 = PRF(Password, Salt || INT_32_BE(i))
        
        let mut salt_idx = salt.to_vec();
        salt_idx.extend_from_slice(&i.to_be_bytes());
        
        let mut u_block = hmac.sign(&salt_idx);
        let mut t_block = u_block.clone();
        
        for _ in 1..iterations {
            u_block = hmac.sign(&u_block);
            for k in 0..64 {
                t_block[k] ^= u_block[k];
            }
        }
        
        derived_key.extend_from_slice(&t_block);
    }
    
    derived_key.truncate(length);
    derived_key
}
