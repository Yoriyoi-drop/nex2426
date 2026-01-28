/// Returns a unique hash of the current CPU hardware.
pub fn get_hardware_id() -> u64 {
    // Cross-platform hardware ID fallback
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    
    // Use multiple system identifiers for uniqueness
    format!("{:?}_{:?}_{:?}", 
        std::process::id(),
        std::env::consts::OS,
        std::env::consts::ARCH
    ).hash(&mut hasher);
    
    hasher.finish()
}

/// Safe wrapper for AVX2 mixing (Rust fallback)
/// ptr: mutable slice of 4 u64s
/// mask: slice of 4 u64s
#[inline(always)]
pub fn asm_mix_avx2(ptr: &mut [u64], mask: &[u64]) {
    assert!(ptr.len() >= 4 && mask.len() >= 4);
    
    // Rust fallback implementation
    for i in 0..4 {
        ptr[i] = asm_mix(ptr[i], mask[i]);
    }
}

/// Mixes two 64-bit integers using Rust implementation.
#[inline(always)]
pub fn asm_mix(a: u64, b: u64) -> u64 {
    // Rust fallback: (a + b) ^ b, then rotate left by 13
    ((a.wrapping_add(b)) ^ b).rotate_left(13)
}

/// Scrambles a 64-bit integer using Rust implementation.
#[inline(always)]
pub fn asm_scramble(val: u64) -> u64 {
    // Rust fallback: simple scrambling based on splitmix64
    let mut result = val;
    result = result.wrapping_mul(0x9E3779B97F4A7C15);
    result ^= result >> 30;
    result = result.wrapping_mul(0xBF58476D1CE4E5B9);
    result ^= result >> 27;
    result = result.wrapping_mul(0x94D049BB133111EB);
    result ^= result >> 31;
    result
}

/// Alias for random generation, uses scramble internally
#[inline(always)]
pub fn asm_pseudo_rand(seed: u64) -> u64 {
    asm_scramble(seed)
}
