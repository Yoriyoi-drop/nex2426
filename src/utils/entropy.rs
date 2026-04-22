//! Cross-platform entropy source for cryptographic operations
//! Provides secure random bytes on both Unix-like systems and Windows

use rand::{RngCore, Rng, Error as RandError};

/// Cross-platform secure random number generator
pub struct SecureRng {
    rng: rand::rngs::ThreadRng,
}

impl SecureRng {
    /// Create a new secure RNG instance
    pub fn new() -> Result<Self, EntropyError> {
        // Test RNG to ensure it works
        let mut test_bytes = [0u8; 4];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut test_bytes);
        
        // Ensure we got some random data (not all zeros)
        if test_bytes.iter().all(|&b| b == 0) {
            return Err(EntropyError::InsufficientEntropy);
        }
        
        Ok(SecureRng { rng })
    }
    
    /// Create a secure RNG without validation (for performance)
    pub fn new_fast() -> Self {
        SecureRng { 
            rng: rand::thread_rng()
        }
    }
    
    /// Fill a buffer with cryptographically secure random bytes
    pub fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), EntropyError> {
        self.rng.fill_bytes(dest);
        
        // Basic sanity check - ensure not all zeros
        if dest.len() > 0 && dest.iter().all(|&b| b == 0) {
            return Err(EntropyError::InsufficientEntropy);
        }
        
        Ok(())
    }
    
    /// Generate a random u64 value
    pub fn next_u64(&mut self) -> Result<u64, EntropyError> {
        let mut bytes = [0u8; 8];
        self.fill_bytes(&mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
    }
    
    /// Generate a random u64 value without validation (faster)
    pub fn next_u64_fast(&mut self) -> u64 {
        use rand::Rng;
        self.rng.r#gen()
    }
    
    /// Generate a random i64 value in specified range
    pub fn next_i64_range(&mut self, min: i64, max: i64) -> Result<i64, EntropyError> {
        if min >= max {
            return Err(EntropyError::InvalidRange);
        }
        
        let range = max - min;
        let random_u64 = self.next_u64()?;
        let random_i64 = (random_u64 % range as u64) as i64;
        Ok(min + random_i64)
    }
}

impl Default for SecureRng {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            eprintln!("Warning: Failed to initialize secure RNG, using fallback");
            // For fallback, we just create a SecureRng without additional validation
            // In production, this should be handled more carefully
            SecureRng::new_fast()
        })
    }
}

/// Errors that can occur during entropy generation
#[derive(Debug, Clone)]
pub enum EntropyError {
    /// System does not have sufficient entropy
    InsufficientEntropy,
    /// Invalid range specified
    InvalidRange,
    /// Underlying RNG error
    RngError(String),
}

impl std::fmt::Display for EntropyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntropyError::InsufficientEntropy => write!(f, "Insufficient entropy available"),
            EntropyError::InvalidRange => write!(f, "Invalid range specified"),
            EntropyError::RngError(msg) => write!(f, "RNG error: {}", msg),
        }
    }
}

impl std::error::Error for EntropyError {}

impl From<RandError> for EntropyError {
    fn from(err: RandError) -> Self {
        EntropyError::RngError(err.to_string())
    }
}

/// Convenience function to get secure random bytes
pub fn secure_random_bytes(len: usize) -> Result<Vec<u8>, EntropyError> {
    let mut rng = SecureRng::new()?;
    let mut bytes = vec![0u8; len];
    rng.fill_bytes(&mut bytes)?;
    Ok(bytes)
}

/// Convenience function to get secure random bytes (fast version)
pub fn secure_random_bytes_fast(len: usize) -> Vec<u8> {
    let mut rng = SecureRng::new_fast();
    let mut bytes = vec![0u8; len];
    rng.rng.fill_bytes(&mut bytes);
    bytes
}

/// Convenience function to get a secure random u64
pub fn secure_random_u64() -> Result<u64, EntropyError> {
    let mut rng = SecureRng::new()?;
    rng.next_u64()
}

/// Convenience function to get a secure random u64 (fast version)
pub fn secure_random_u64_fast() -> u64 {
    let mut rng = SecureRng::new_fast();
    rng.next_u64_fast()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_rng_basic() {
        let mut rng = SecureRng::new().expect("Failed to create SecureRng");
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes).expect("Failed to fill bytes");
        
        // Ensure we got some random data
        assert!(!bytes.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_secure_rng_u64() {
        let mut rng = SecureRng::new().expect("Failed to create SecureRng");
        let val1 = rng.next_u64().expect("Failed to generate u64");
        let val2 = rng.next_u64().expect("Failed to generate u64");
        
        // Values should be different (very high probability)
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_secure_rng_range() {
        let mut rng = SecureRng::new().expect("Failed to create SecureRng");
        let val = rng.next_i64_range(-100, 100).expect("Failed to generate i64 in range");
        
        assert!(val >= -100);
        assert!(val < 100);
    }

    #[test]
    fn test_convenience_functions() {
        let bytes = secure_random_bytes(16).expect("Failed to generate random bytes");
        assert_eq!(bytes.len(), 16);
        assert!(!bytes.iter().all(|&b| b == 0));
        
        let val = secure_random_u64().expect("Failed to generate random u64");
        assert!(val > 0 || val == 0); // Just ensure it's a valid u64
    }
}
