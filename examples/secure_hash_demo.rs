//! Demo of secure password hashing with Nex2426
//! 
//! This example demonstrates the new secure password hashing functionality
//! with proper salt, cost factor, and memory-hard preprocessing.

use nex2426::kernel::NexKernel;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Nex2426 Secure Password Hashing Demo ===\n");
    
    let kernel = NexKernel::new(1);
    let password = b"my_secure_password_123!";
    let cost = 1000; // Reasonable cost for demo
    
    println!("1. Creating secure hash...");
    let hash_result = kernel.hash_secure(password, cost)?;
    
    println!("   Password: {}", String::from_utf8_lossy(password));
    println!("   Cost factor: {}", cost);
    println!("   Hash format: {}", hash_result.full_formatted_string);
    println!("   Hash length: {} chars", hash_result.full_formatted_string.len());
    println!("   Debug info: {:?}", hash_result.debug_info);
    
    println!("\n2. Verifying correct password...");
    let is_valid = kernel.verify_secure(password, &hash_result.full_formatted_string)?;
    println!("   Verification result: {}", is_valid);
    
    println!("\n3. Verifying wrong password...");
    let wrong_password = b"wrong_password";
    let is_invalid = kernel.verify_secure(wrong_password, &hash_result.full_formatted_string)?;
    println!("   Verification result: {}", is_invalid);
    
    println!("\n4. Testing hash uniqueness...");
    let hash2 = kernel.hash_secure(password, cost)?;
    println!("   First hash:  {}", &hash_result.full_formatted_string[..80]);
    println!("   Second hash: {}", &hash2.full_formatted_string[..80]);
    println!("   Hashes are different: {}", hash_result.full_formatted_string != hash2.full_formatted_string);
    println!("   Both verify correctly: {} && {}", 
        kernel.verify_secure(password, &hash_result.full_formatted_string)?,
        kernel.verify_secure(password, &hash2.full_formatted_string)?);
    
    println!("\n5. Comparing with old insecure method...");
    let insecure_hash = kernel.hash_standard(password, "some_key");
    println!("   Insecure format: {}", insecure_hash.full_formatted_string);
    println!("   Insecure debug: {:?}", insecure_hash.debug_info);
    
    println!("\n=== Security Analysis ===");
    println!("   Old method: No salt, SHA-256, no cost factor");
    println!("   New method: 32-byte salt, SHA-512, cost factor {}, memory-hard", cost);
    println!("   Security improvement: ~1000x stronger against brute force");
    
    Ok(())
}
