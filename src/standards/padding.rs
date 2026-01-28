/// Industry Standard PKCS#7 Padding
/// Used to pad blocks to a specific block size (usually 16 bytes for AES, but we support dynamic).

#[derive(Debug)]
pub enum PaddingError {
    InvalidPadding,
    BlockSizeTooLarge,
}

pub fn pad(input: &[u8], block_size: usize) -> Vec<u8> {
    if block_size > 255 {
        panic!("Block size must be <= 255 for PKCS#7");
    }
    
    let pad_len = block_size - (input.len() % block_size);
    let mut output = input.to_vec();
    
    for _ in 0..pad_len {
        output.push(pad_len as u8);
    }
    
    output
}

pub fn unpad(input: &[u8]) -> Result<Vec<u8>, PaddingError> {
    if input.is_empty() {
        return Err(PaddingError::InvalidPadding);
    }
    
    let pad_len = input[input.len() - 1] as usize;
    
    if pad_len == 0 || pad_len > input.len() {
        return Err(PaddingError::InvalidPadding);
    }
    
    // Check all padding bytes
    for i in 0..pad_len {
        if input[input.len() - 1 - i] != pad_len as u8 {
            return Err(PaddingError::InvalidPadding);
        }
    }
    
    Ok(input[0..input.len() - pad_len].to_vec())
}
