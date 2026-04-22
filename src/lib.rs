//! NEX2426 - Quantum-Resistant Chaos Encryption Engine
//! 
//! A sophisticated encryption system providing multiple layers of
//! security against both classical and quantum computer attacks.

pub mod kernel;
pub mod transform;
pub mod utils;
pub mod whitebox;
pub mod quantum;
pub mod integrity;
pub mod compression;
pub mod kms;
pub mod audit;
pub mod standards;
pub mod protocol;
pub mod security;
pub mod nex_io;
pub mod features;
pub mod c_api;
pub mod blockchain;
pub mod error;
pub mod validation;
pub mod memory_opt;
pub mod logging;

#[cfg(test)]
mod tests {
    use super::kernel::NexKernel;
    use std::io::Cursor;

    #[test]
    fn test_basic_hashing() {
        let kernel = NexKernel::new(1);
        let mut cursor = Cursor::new("test input");
        let result = kernel.execute(&mut cursor, "test_key");
        
        assert!(!result.full_formatted_string.is_empty());
        assert!(result.full_formatted_string.starts_with("$nex6$"));
        // In deterministic mode, timestamp is 0
        assert!(result.timestamp >= 0);
    }

    #[test]
    fn test_cost_parameter() {
        let kernel1 = NexKernel::new(1);
        let kernel3 = NexKernel::new(3);
        
        let mut cursor = Cursor::new("test");
        let result1 = kernel1.execute(&mut cursor, "key");
        
        let mut cursor = Cursor::new("test");
        let result2 = kernel3.execute(&mut cursor, "key");
        
        // Different cost should produce different results
        assert_ne!(result1.full_formatted_string, result2.full_formatted_string);
    }

    #[test]
    fn test_temporal_binding() {
        let mut kernel = NexKernel::new(1);
        kernel.enable_temporal_binding();
        
        let mut cursor = Cursor::new("test");
        let result1 = kernel.execute(&mut cursor, "key");
        
        // Wait a bit to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let mut cursor = Cursor::new("test");
        let result2 = kernel.execute(&mut cursor, "key");
        
        // With temporal binding, results should be different
        assert_ne!(result1.full_formatted_string, result2.full_formatted_string);
    }

    #[test]
    fn test_deterministic_mode() {
        let kernel = NexKernel::new(1);
        
        let mut cursor = Cursor::new("test");
        let result1 = kernel.execute(&mut cursor, "key");
        
        let mut cursor = Cursor::new("test");
        let result2 = kernel.execute(&mut cursor, "key");
        
        // Deterministic mode should produce same result
        assert_eq!(result1.full_formatted_string, result2.full_formatted_string);
    }

    #[test]
    fn test_hash_bytes() {
        let kernel = NexKernel::new(1);
        let data = b"test data";
        let result = kernel.hash_bytes(data, "key");
        
        assert_eq!(result.len(), 64); // 512 bits = 64 bytes
    }

    #[test]
    fn test_empty_input() {
        let kernel = NexKernel::new(1);
        let mut cursor = Cursor::new("");
        let result = kernel.execute(&mut cursor, "key");
        
        assert!(!result.full_formatted_string.is_empty());
    }

    #[test]
    fn test_long_input() {
        let kernel = NexKernel::new(1);
        let long_data = "x".repeat(10000);
        let mut cursor = Cursor::new(long_data);
        let result = kernel.execute(&mut cursor, "key");
        
        assert!(!result.full_formatted_string.is_empty());
        assert!(result.full_formatted_string.len() > 100);
    }

    #[test]
    fn test_unicode_input() {
        let kernel = NexKernel::new(1);
        let unicode_data = "Hello 世界 🌍 ñoël";
        let mut cursor = Cursor::new(unicode_data);
        let result = kernel.execute(&mut cursor, "key");
        
        assert!(!result.full_formatted_string.is_empty());
    }

    #[test]
    fn test_special_characters_key() {
        let kernel = NexKernel::new(1);
        let special_key = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
        let mut cursor = Cursor::new("test");
        let result = kernel.execute(&mut cursor, special_key);
        
        assert!(!result.full_formatted_string.is_empty());
    }
}
