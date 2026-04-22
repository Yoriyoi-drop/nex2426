/// Industry Standard PKCS#7 Padding
/// Used to pad blocks to a specific block size (usually 16 bytes for AES, but we support dynamic).

#[derive(Debug)]
pub enum PaddingError {
    InvalidPadding,
    BlockSizeTooLarge,
}

pub fn pad(input: &[u8], block_size: usize) -> crate::error::NexResult<Vec<u8>> {
    if block_size > 255 {
        return Err(crate::error::NexError::InvalidInput("Block size must be <= 255 for PKCS#7".to_string()));
    }
    
    let pad_len = block_size - (input.len() % block_size);
    let mut output = input.to_vec();
    
    for _ in 0..pad_len {
        output.push(pad_len as u8);
    }
    
    Ok(output)
}

pub fn unpad(input: &[u8]) -> crate::error::NexResult<Vec<u8>> {
    if input.is_empty() {
        return Err(crate::error::NexError::InvalidInput("Invalid padding: empty input".to_string()));
    }
    
    let pad_len = input[input.len() - 1] as usize;
    
    if pad_len == 0 || pad_len > input.len() {
        return Err(crate::error::NexError::InvalidInput("Invalid padding: incorrect pad length".to_string()));
    }
    
    // Check all padding bytes
    for i in 0..pad_len {
        if input[input.len() - 1 - i] != pad_len as u8 {
            return Err(crate::error::NexError::InvalidInput("Invalid padding: incorrect padding bytes".to_string()));
        }
    }
    
    Ok(input[0..input.len() - pad_len].to_vec())
}
