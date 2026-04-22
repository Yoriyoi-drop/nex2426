//! Integration Tests for NEX2426
//! 
//! Tests the complete pipeline and fixes that were applied

use nex2426::kernel::NexKernel;
use nex2426::protocol::kx::NexKeyExchange;
use nex2426::standards::hmac::HmacNex;
use nex2426::standards::modes::ctr::CNTMode;
use nex2426::features::config::NexConfig;

#[test]
fn test_key_exchange_real_ring_lwe() {
    println!("Testing Key Exchange with real Ring-LWE implementation...");
    
    // Alice generates keypair
    let mut alice = NexKeyExchange::new();
    let alice_pub = alice.generate_keypair();
    
    // Bob encapsulates secret
    let mut bob = NexKeyExchange::new();
    let (bob_ciphertext, bob_shared) = bob.encapsulate(&alice_pub);
    
    // Alice decapsulates secret
    let alice_shared = alice.decapsulate(&bob_ciphertext);
    
    // Both should have the same shared secret
    assert_eq!(bob_shared, alice_shared, "Shared secrets should match!");
    assert_ne!(bob_shared, [0xAA; 32], "Shared secret should not be placeholder!");
    
    println!("✅ Key Exchange: Real Ring-LWE working correctly");
}

#[test]
fn test_ctr_mode_consistent_key() {
    println!("Testing CTR Mode with consistent key (vulnerability fix)...");
    
    let kernel = NexKernel::new(1);
    let nonce = [0x42u8; 32];
    let ctr_key = "test_ctr_key_123".to_string();
    
    let mut ctr = CNTMode::new(kernel, nonce, ctr_key.clone());
    
    let plaintext = b"Hello, World! This is a test message.";
    let ciphertext = ctr.process(plaintext);
    
    // Reset CTR with same key and nonce
    let kernel2 = NexKernel::new(1);
    let mut ctr2 = CNTMode::new(kernel2, nonce, ctr_key);
    let decrypted = ctr2.process(&ciphertext);
    
    assert_eq!(plaintext.to_vec(), decrypted, "CTR encryption/decryption should match");
    
    println!("✅ CTR Mode: Consistent key working correctly");
}

#[test]
fn test_hmac_block_size_consistency() {
    println!("Testing HMAC with proper block size...");
    
    let key = b"test_hmac_key_123456";
    let message = b"This is a test message for HMAC.";
    
    let hmac = HmacNex::new(key);
    let signature = hmac.sign(message);
    
    // Verification should work
    assert!(hmac.verify(message, &signature), "HMAC verification should succeed");
    
    // Different message should fail
    let wrong_message = b"This is a wrong message.";
    assert!(!hmac.verify(wrong_message, &signature), "HMAC verification should fail for wrong message");
    
    println!("✅ HMAC: Block size consistency working correctly");
}

#[test]
fn test_kernel_verification_fixed() {
    println!("Testing Kernel verification with temporal binding fix...");
    
    let kernel = NexKernel::new(1);
    let data = b"Test data for verification";
    let key = "test_key";
    
    // Generate hash
    let mut cursor = std::io::Cursor::new(data);
    let hash_result = kernel.execute(&mut cursor, key);
    
    // Verification should work for deterministic kernel
    let deterministic_kernel = NexKernel::new(1);
    assert!(deterministic_kernel.verify(data, key, &hash_result.full_formatted_string), 
              "Verification should work for deterministic kernel");
    
    println!("✅ Kernel: Verification working correctly");
}

#[test]
fn test_memory_protection_safe() {
    println!("Testing memory protection safety fixes...");
    
    use nex2426::security::memory::{Protected, Zeroize};
    
    let mut protected_data = Protected::new([0x12345678u64; 8]);
    
    // Access and modify data
    {
        let data = protected_data.access_mut();
        data[0] = 0x87654321;
        assert_eq!(data[0], 0x87654321);
    }
    
    // Data should still be accessible
    assert_eq!(protected_data.access()[0], 0x87654321);
    
    println!("✅ Memory Protection: Safe implementation working correctly");
}

#[test]
fn test_quantum_lattice_optimized() {
    println!("Testing optimized quantum lattice (no massive matrix)...");
    
    use nex2426::quantum::lattice::LatticeEngine;
    
    let mut lattice = LatticeEngine::new();
    let test_input = [1u32, 2, 3, 4, 5];
    
    // Inject test data
    lattice.inject(&test_input);
    
    // Apply diffusion (should be fast without 100x100 matrix)
    let seed = [0x12345678, 0x9ABCDEF0, 0xFEDCBA98, 0x76543210];
    lattice.diffuse(seed);
    
    // State should be different from input
    assert_ne!(lattice.state[..5], test_input, "Lattice diffusion should change state");
    
    println!("✅ Quantum Lattice: Optimized implementation working correctly");
}

#[test]
fn test_whitebox_dynamic_generation() {
    println!("Testing whitebox with dynamic table generation...");
    
    use nex2426::whitebox::network::NetworkEngine;
    
    let seed = [0x12345678u32; 16];
    let mut engine = NetworkEngine::new(seed);
    
    // Execute whitebox rounds
    engine.execute();
    
    // State should be different from initial seed
    assert_ne!(engine.state, seed, "Whitebox execution should change state");
    
    println!("✅ Whitebox: Dynamic generation working correctly");
}

#[test]
fn test_config_validation_specific_errors() {
    println!("Testing config validation with specific error types...");
    
    use nex2426::features::config::{NexConfig, ConfigError};
    
    let mut config = NexConfig::default();
    
    // Valid config should pass
    assert!(config.validate().is_ok(), "Default config should be valid");
    
    // Invalid cost should fail with specific error
    config.default_cost = 0;
    let result = config.validate();
    assert!(result.is_err(), "Zero cost should fail validation");
    
    if let Err(ConfigError::Validation(msg)) = result {
        assert!(msg.contains("between 1 and"), "Error should mention valid range");
    } else {
        panic!("Should return ConfigError::Validation");
    }
    
    println!("✅ Config: Specific error handling working correctly");
}

#[test]
fn test_complete_pipeline_integration() {
    println!("Testing complete pipeline integration...");
    
    // 1. Generate key pair
    let mut alice = NexKeyExchange::new();
    let alice_pub = alice.generate_keypair();
    
    // 2. Bob encapsulates and gets shared secret
    let mut bob = NexKeyExchange::new();
    let (ciphertext, shared_secret) = bob.encapsulate(&alice_pub);
    
    // 3. Alice decapsulates and gets same shared secret
    let alice_shared = alice.decapsulate(&ciphertext);
    assert_eq!(shared_secret, alice_shared);
    
    // 4. Use shared secret for HMAC
    let hmac = HmacNex::new(&shared_secret);
    let message = b"Integration test message";
    let signature = hmac.sign(message);
    
    // 5. Verify HMAC
    assert!(hmac.verify(message, &signature));
    
    // 6. Use shared secret for CTR encryption
    let kernel = NexKernel::new(1);
    let nonce = [0u8; 32];
    let ctr_key = hex::encode(&shared_secret);
    let mut ctr = CNTMode::new(kernel, nonce, ctr_key);
    
    let encrypted = ctr.process(message);
    let mut ctr2 = CNTMode::new(NexKernel::new(1), [0u8; 32], hex::encode(&alice_shared));
    let decrypted = ctr2.process(&encrypted);
    
    assert_eq!(message.to_vec(), decrypted);
    
    println!("✅ Complete Pipeline: All components working together correctly");
}
