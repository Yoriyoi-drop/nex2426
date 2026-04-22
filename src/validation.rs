//! Input validation utilities for NEX2426 Secure Storage Engine
//! 
//! Provides comprehensive input validation for all public APIs to ensure
//! security and prevent invalid states. Focus on secure storage applications.

use crate::error::NexResult;
use crate::ensure;

/// Validate byte array input
pub fn validate_bytes(input: &[u8], name: &str, min_len: usize, max_len: usize) -> NexResult<()> {
    ensure!(!input.is_empty(), invalid_input, "{} cannot be empty", name);
    ensure!(input.len() >= min_len, invalid_input, "{} must be at least {} bytes", name, min_len);
    ensure!(input.len() <= max_len, invalid_input, "{} cannot exceed {} bytes", name, max_len);
    Ok(())
}

/// Validate string input
pub fn validate_string(input: &str, name: &str, min_len: usize, max_len: usize) -> NexResult<()> {
    ensure!(!input.is_empty(), invalid_input, "{} cannot be empty", name);
    ensure!(input.len() >= min_len, invalid_input, "{} must be at least {} characters", name, min_len);
    ensure!(input.len() <= max_len, invalid_input, "{} cannot exceed {} characters", name, max_len);
    ensure!(input.chars().all(|c| c.is_ascii()), invalid_input, "{} must contain only ASCII characters", name);
    Ok(())
}

/// Validate array input
pub fn validate_array<T>(input: &[T], name: &str, min_len: usize, max_len: usize) -> NexResult<()> {
    ensure!(!input.is_empty(), invalid_input, "{} cannot be empty", name);
    ensure!(input.len() >= min_len, invalid_input, "{} must have at least {} elements", name, min_len);
    ensure!(input.len() <= max_len, invalid_input, "{} cannot exceed {} elements", name, max_len);
    Ok(())
}

/// Validate numeric input
pub fn validate_numeric<T: PartialOrd + std::fmt::Display>(
    input: T, name: &str, min_val: T, max_val: T
) -> NexResult<()> {
    ensure!(input >= min_val, invalid_input, "{} must be at least {}", name, min_val);
    ensure!(input <= max_val, invalid_input, "{} must be at most {}", name, max_val);
    Ok(())
}

/// Validate file path
pub fn validate_file_path(path: &str) -> NexResult<()> {
    ensure!(!path.is_empty(), invalid_input, "File path cannot be empty");
    ensure!(path.len() <= 4096, invalid_input, "File path too long (max 4096 characters)");
    
    // Check for invalid characters
    let invalid_chars = ['\0', '<', '>', ':', '"', '|', '?', '*'];
    for c in path.chars() {
        ensure!(!invalid_chars.contains(&c), invalid_input, "File path contains invalid character: {}", c);
    }
    
    Ok(())
}

/// Validate key material
pub fn validate_key_material(key: &[u8], name: &str) -> NexResult<()> {
    validate_array(key, name, 16, 1024)?; // 16 bytes to 1KB
    
    // Check for weak keys (all zeros, all ones, repeated patterns)
    ensure!(!key.iter().all(|&b| b == 0), invalid_input, "{} cannot be all zeros", name);
    ensure!(!key.iter().all(|&b| b == 0xFF), invalid_input, "{} cannot be all 0xFF", name);
    
    // Check for repeating patterns
    if key.len() >= 8 {
        let first_8 = &key[..8];
        let mut has_repetition = false;
        for chunk in key.chunks_exact(8) {
            if chunk == first_8 {
                has_repetition = true;
                break;
            }
        }
        ensure!(!has_repetition, invalid_input, "{} contains repeating patterns", name);
    }
    
    Ok(())
}

/// Validate nonce/IV
pub fn validate_nonce(nonce: &[u8], name: &str) -> NexResult<()> {
    validate_array(nonce, name, 8, 64)?; // 8 to 64 bytes
    
    // Nonce should have sufficient entropy
    let unique_bytes = nonce.iter().collect::<std::collections::HashSet<_>>().len();
    let min_entropy = (nonce.len() as f64 * 0.5) as usize; // At least 50% unique
    ensure!(unique_bytes >= min_entropy, invalid_input, "{} has insufficient entropy", name);
    
    Ok(())
}


/// Validate thread count
pub fn validate_thread_count(threads: usize) -> NexResult<()> {
    validate_numeric(threads, "thread count", 1, 1024)?;
    
    let available = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    
    ensure!(threads <= available * 2, invalid_input, 
             "Thread count ({}) exceeds reasonable limit ({})", threads, available * 2);
    
    Ok(())
}

/// Validate network packet
pub fn validate_network_packet(packet: &[u32]) -> NexResult<()> {
    validate_array(packet, "network packet", 16, 1024)?;
    
    // Check for weak packets
    ensure!(!packet.iter().all(|&x| x == 0), invalid_input, "Network packet cannot be all zeros");
    
    Ok(())
}

/// Validate cost parameter for memory-hard encryption
pub fn validate_cost(cost: u32) -> NexResult<()> {
    ensure!(cost >= 1, invalid_input, "Cost must be at least 1");
    ensure!(cost <= 100, invalid_input, "Cost cannot exceed 100 (memory limit)");
    Ok(())
}

/// Validate memory requirements for secure storage
pub fn validate_memory_requirements(cost: u32, available_mb: u64) -> NexResult<()> {
    let required_mb = (8 * cost) as u64; // 8MB per cost unit
    ensure!(required_mb <= available_mb, 
memory, 
            "Insufficient memory: required {}MB, available {}MB", 
            required_mb, available_mb);
    Ok(())
}

/// Validate file size for encryption (prevent DoS)
pub fn validate_file_size(size: u64, max_size_mb: u64) -> NexResult<()> {
    let max_bytes = max_size_mb * 1024 * 1024;
    ensure!(size <= max_bytes, 
            invalid_input, 
            "File too large: {}MB exceeds limit of {}MB", 
            size / 1024 / 1024, max_size_mb);
    Ok(())
}

/// Macro for easy validation in function signatures
#[macro_export]
macro_rules! validate_input {
    ($input:expr, $name:expr, bytes, $min:expr, $max:expr) => {
        $crate::validation::validate_bytes($input, $name, $min, $max)?
    };
    ($input:expr, $name:expr, string, $min:expr, $max:expr) => {
        $crate::validation::validate_string($input, $name, $min, $max)?
    };
    ($input:expr, $name:expr, array, $min:expr, $max:expr) => {
        $crate::validation::validate_array($input, $name, $min, $max)?
    };
    ($input:expr, $name:expr, numeric, $min:expr, $max:expr) => {
        $crate::validation::validate_numeric($input, $name, $min, $max)?
    };
    ($input:expr, $name:expr, key) => {
        $crate::validation::validate_key_material($input, $name)?
    };
    ($input:expr, $name:expr, nonce) => {
        $crate::validation::validate_nonce($input, $name)?
    };
    ($input:expr, $name:expr, cost) => {
        $crate::validation::validate_cost($input, $name)?
    };
    ($input:expr, threads) => {
        $crate::validation::validate_thread_count($input)?
    };
    ($input:expr, $name:expr, packet) => {
        $crate::validation::validate_network_packet($input)?
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_bytes() {
        let valid = b"hello world";
        assert!(validate_bytes(valid, "test", 1, 100).is_ok());
        
        let invalid = b"";
        assert!(validate_bytes(invalid, "test", 1, 100).is_err());
    }

    #[test]
    fn test_validate_key_material() {
        let valid = [0x12, 0x34, 0x56, 0x78];
        assert!(validate_key_material(&valid, "test").is_ok());
        
        let weak = [0x00; 16];
        assert!(validate_key_material(&weak, "test").is_err());
    }

    #[test]
    fn test_validate_macro() {
        let key = b"test key";
        assert!(validate_input!(key, "key", key).is_ok());
        
        let cost = 1000u32;
        assert!(validate_input!(cost, "cost", cost).is_ok());
    }
}
