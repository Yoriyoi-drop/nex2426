//! Reproducible Hashing Demo
//! 
//! This demo shows how to use the fixed Nex2426 with:
//! 1. Reproducible hashing (same input + same seed = same output)
//! 2. Simplified modes for different use cases
//! 3. Debug information for troubleshooting
//! 4. Standard compatibility

use nex2426::kernel::NexKernel;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 NEX2426 REPRODUCIBLE HASHING DEMO");
    println!("=====================================\n");

    // Test data
    let test_data = b"Hello, World!";
    let test_key = "test_key_123";
    let fixed_seed = 12345;

    println!("📋 Test Configuration:");
    println!("  Data: {:?}", std::str::from_utf8(test_data).unwrap_or("[binary]"));
    println!("  Key: {}", test_key);
    println!("  Fixed Seed: {}\n", fixed_seed);

    // 1. REPRODUCIBLE HASHING
    println!("🔄 1. REPRODUCIBLE HASHING");
    println!("---------------------------");
    
    let kernel = NexKernel::new(1);
    
    println!("Testing reproducibility with same seed...");
    let result1 = kernel.hash_reproducible(test_data, test_key, fixed_seed);
    let result2 = kernel.hash_reproducible(test_data, test_key, fixed_seed);
    
    println!("Hash 1: {}", result1.full_formatted_string);
    println!("Hash 2: {}", result2.full_formatted_string);
    println!("Reproducible: {}\n", result1.full_formatted_string == result2.full_formatted_string);

    // 2. SIMPLIFIED HASHING
    println!("⚡ 2. SIMPLIFIED HASHING");
    println!("-------------------------");
    
    let simple_result = kernel.hash_simple(test_data, test_key);
    println!("Simple Hash: {}", simple_result.full_formatted_string);
    println!("Debug Info: {:?}", simple_result.debug_info);
    println!();

    // 3. STANDARD HASHING (SHA-256)
    println!("🔐 3. STANDARD HASHING (SHA-256)");
    println!("--------------------------------");
    
    let standard_result = kernel.hash_standard(test_data, test_key);
    println!("Standard Hash: {}", standard_result.full_formatted_string);
    println!("Debug Info: {:?}", standard_result.debug_info);
    println!();

    // 4. DEBUG MODE
    println!("🐛 4. DEBUG MODE");
    println!("---------------");
    
    let debug_kernel = NexKernel::debug(1);
    let mut cursor = std::io::Cursor::new(test_data);
    let debug_result = debug_kernel.execute(&mut cursor, test_key);
    
    println!("Debug Hash: {}", debug_result.full_formatted_string);
    println!("Debug Information:");
    for info in &debug_result.debug_info {
        println!("  - {}", info);
    }
    println!();

    // 5. DIFFERENT SEEDS = DIFFERENT RESULTS
    println!("🎲 5. DIFFERENT SEEDS");
    println!("---------------------");
    
    let seed1_result = kernel.hash_reproducible(test_data, test_key, 11111);
    let seed2_result = kernel.hash_reproducible(test_data, test_key, 22222);
    let seed3_result = kernel.hash_reproducible(test_data, test_key, 33333);
    
    println!("Seed 11111: {}", seed1_result.full_formatted_string);
    println!("Seed 22222: {}", seed2_result.full_formatted_string);
    println!("Seed 33333: {}", seed3_result.full_formatted_string);
    println!("All different: {}", 
        seed1_result.full_formatted_string != seed2_result.full_formatted_string &&
        seed2_result.full_formatted_string != seed3_result.full_formatted_string &&
        seed1_result.full_formatted_string != seed3_result.full_formatted_string
    );
    println!();

    // 6. STATE EXPORT
    println!("📤 6. STATE EXPORT");
    println!("------------------");
    
    let state = kernel.export_state(test_data, test_key);
    println!("Internal State:");
    println!("{}", serde_json::to_string_pretty(&state)?);
    println!();

    // 7. PERFORMANCE COMPARISON
    println!("⚡ 7. PERFORMANCE COMPARISON");
    println!("----------------------------");
    
    let iterations = 1000;
    
    // Full pipeline
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = kernel.hash_reproducible(test_data, test_key, fixed_seed);
    }
    let full_time = start.elapsed();
    
    // Simplified pipeline
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = kernel.hash_simple(test_data, test_key);
    }
    let simple_time = start.elapsed();
    
    // Standard pipeline
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = kernel.hash_standard(test_data, test_key);
    }
    let standard_time = start.elapsed();
    
    println!("Performance ({} iterations):", iterations);
    println!("  Full Pipeline:    {:?} ({:.2} ops/sec)", 
        full_time, iterations as f64 / full_time.as_secs_f64());
    println!("  Simplified:       {:?} ({:.2} ops/sec)", 
        simple_time, iterations as f64 / simple_time.as_secs_f64());
    println!("  Standard (SHA256): {:?} ({:.2} ops/sec)", 
        standard_time, iterations as f64 / standard_time.as_secs_f64());
    println!();

    // 8. USE CASE EXAMPLES
    println!("💡 8. USE CASE EXAMPLES");
    println!("-----------------------");
    
    // Password hashing
    println!("Password Hashing:");
    let password = b"my_secure_password_123";
    let salt = "user_salt_456";
    let password_hash = kernel.hash_simple(password, salt);
    println!("  Hash: {}", password_hash.full_formatted_string);
    
    // File integrity
    println!("\nFile Integrity:");
    let file_content = b"important_file_content_that_should_not_be_modified";
    let integrity_hash = kernel.hash_standard(file_content, "integrity_check");
    println!("  Hash: {}", integrity_hash.full_formatted_string);
    
    // Data verification
    println!("\nData Verification:");
    let verification_hash = kernel.hash_reproducible(test_data, test_key, fixed_seed);
    let is_valid = kernel.verify_reproducibility(test_data, test_key, fixed_seed);
    println!("  Hash: {}", verification_hash.full_formatted_string);
    println!("  Verified: {}", is_valid);
    
    println!("\n✅ Demo completed successfully!");
    println!("\n🎯 Key Improvements:");
    println!("  ✓ Reproducible: Same input + same seed = same output");
    println!("  ✓ Simplified: No over-engineering for simple cases");
    println!("  ✓ Debuggable: Clear internal state information");
    println!("  ✓ Standard: SHA-256 compatibility for interoperability");
    println!("  ✓ Flexible: Multiple modes for different use cases");
    println!("  ✓ Fast: Optimized for performance when needed");

    Ok(())
}
