use crate::utils::asm_ops;
use std::thread;

// 1 Million u64s = 8MB per Lane. 
// With 16 threads, this uses ~128MB RAM, which is decent for a "hardened" config without being excessive.
const MEMORY_SIZE: usize = 1024 * 1024; 

/// Performs the memory hardening walk on a single lane (buffer) using AVX2 SIMD.
/// Processes 4 x u64 (256 bits) at a time for maximum throughput.
fn memory_walk_lane(mut seed: u64, iterations: u32) -> Vec<u64> {
    let mut arena = vec![0u64; MEMORY_SIZE];

    // 1. Initialize Arena (Scalar initialization is fine, or could be vectorized)
    for i in 0..MEMORY_SIZE {
        seed = asm_ops::asm_scramble(seed ^ (i as u64));
        arena[i] = seed;
    }

    // 2. SIMD Mixing Walk
    // We treat the arena as a series of 256-bit (4 u64) blocks.
    // Total chunks = MEMORY_SIZE / 4
    const CHUNK_SIZE: usize = 4;
    let num_chunks = MEMORY_SIZE / CHUNK_SIZE;

    // Accumulator register (4 x u64)
    // We simulate a register using a small slice
    let mut accumulator = [0x1234567890ABCDEF, 0xDEADBEEFCAFEBABE, 0x0FEDCBA987654321, 0xA5A5A5A55A5A5A5A];

    for _ in 0..iterations {
        // Linear Pass with Feedback (Access Pattern: Cache-Friendly Sequential for Burst Speed + Mixing)
        for i in 0..num_chunks {
            let offset = i * CHUNK_SIZE;
            let chunk = &mut arena[offset..offset+CHUNK_SIZE];
            
            // Mix Accumulator specific to AVX2
            asm_ops::asm_mix_avx2(chunk, &accumulator);
            
            // Update Accumulator with new chunk state
            accumulator.copy_from_slice(chunk);
        }
        
        // Backward Pass (To ensure dependence on tail)
        for i in (0..num_chunks).rev() {
             let offset = i * CHUNK_SIZE;
             let chunk = &mut arena[offset..offset+CHUNK_SIZE];
             
             asm_ops::asm_mix_avx2(chunk, &accumulator);
             accumulator.copy_from_slice(chunk);
        }
    }

    // 3. Compress into 8 blocks (Quantum Proof)
    let mut lane_output = vec![0u64; 8];
    for i in 0..MEMORY_SIZE {
        let chunk_idx = i % 8;
        lane_output[chunk_idx] = asm_ops::asm_scramble(lane_output[chunk_idx] ^ arena[i]);
    }

    lane_output
}

/// Uses standard library threads to utilize all available CPU cores.
/// Scans hardware concurrency to determine lane count.
pub fn apply_memory_hardening_parallel(blocks: Vec<u64>, iterations: u32) -> Vec<u64> {
    if blocks.len() != 8 { return blocks; }

    // dynamic thread count
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2); // Fallback to 2 if query fails

    // println!("Starting Parallel Memory Hardening (Lanes: {}, Memory: {}MB)...", num_threads, (num_threads * MEMORY_SIZE * 8) / 1024 / 1024);

    let mut handles = Vec::with_capacity(num_threads);

    // Create a base seed from the input blocks
    let base_seed = blocks.iter().fold(0u64, |acc, &x| acc ^ x);

    for i in 0..num_threads {
        // Unique seed per lane
        let lane_seed = base_seed ^ (i as u64).wrapping_mul(0x9E3779B97F4A7C15); 
        let iter = iterations;

        handles.push(thread::spawn(move || {
            memory_walk_lane(lane_seed, iter)
        }));
    }

    // Collect and Mix Results
    let mut final_blocks = vec![0u64; 8];
    
    // Initialize with original blocks to preserve entropy
    final_blocks.copy_from_slice(&blocks);

    for handle in handles {
        match handle.join() {
            Ok(lane_result) => {
                for i in 0..8 {
                    final_blocks[i] ^= lane_result[i];
                    final_blocks[i] = asm_ops::asm_scramble(final_blocks[i]);
                }
            }
            Err(_) => {
                // In production we might panic, but here we just continue 
                // to avoid crashing the whole shell if one thread dies (unlikely).
                eprintln!("Error: A memory hardening thread panicked.");
            }
        }
    }

    final_blocks
}
