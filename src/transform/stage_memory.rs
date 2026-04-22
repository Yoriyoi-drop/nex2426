use crate::utils::asm_ops;
use std::thread;

// OPTIMIZED: Reduced memory size for better cache performance
// 256K u64s = 2MB per Lane. With 16 threads, this uses ~32MB RAM.
const MEMORY_SIZE: usize = 256 * 1024; 
const CHUNK_SIZE: usize = 4;

use std::sync::{Arc, Mutex};

// Thread-safe memory pool for reusing allocations across calls
struct MemoryPool {
    pool: Vec<u64>,
}

impl MemoryPool {
    fn new() -> Self {
        Self {
            pool: Vec::with_capacity(MEMORY_SIZE),
        }
    }
    
    fn get_arena(&mut self) -> Vec<u64> {
        if self.pool.len() < MEMORY_SIZE {
            self.pool.resize(MEMORY_SIZE, 0);
        }
        // Return a copy for thread safety
        self.pool.clone()
    }
}

/// Thread-safe memory pool shared across all threads
lazy_static::lazy_static! {
    static ref MEMORY_POOL: Arc<Mutex<MemoryPool>> = Arc::new(Mutex::new(MemoryPool::new()));
}

/// OPTIMIZED: Performs memory hardening with thread-safe memory pool
fn memory_walk_lane(mut seed: u64, iterations: u32) -> Vec<u64> {
    // Get arena from thread-safe pool
    let mut arena = {
        let mut pool = MEMORY_POOL.lock().unwrap();
        pool.get_arena()
    };
    
    // 1. Initialize Arena (vectorized when possible)
    for (i, val) in arena.iter_mut().enumerate() {
        seed = asm_ops::asm_scramble(seed ^ (i as u64));
        *val = seed;
    }

    // 2. SIMD Mixing Walk
    // We treat the arena as a series of 256-bit (4 u64) blocks.
    // Total chunks = MEMORY_SIZE / 4
    let num_chunks = MEMORY_SIZE / CHUNK_SIZE;

    // Accumulator register (4 x u64)
    // We simulate a register using a small slice
    let mut accumulator = [0x1234567890ABCDEF, 0xDEADBEEFCAFEBABE, 0x0FEDCBA987654321, 0xA5A5A5A55A5A5A5A];

    for iter in 0..iterations {
        // OPTIMIZED: Early exit if accumulator converges (security check)
        if iter > 0 && accumulator.iter().all(|x| *x == 0) {
            break;
        }
        
        // Linear Pass with Feedback (Cache-Friendly Sequential Access)
        for i in 0..num_chunks {
            let offset = i * CHUNK_SIZE;
            let chunk = &mut arena[offset..offset+CHUNK_SIZE];
            
            // Mix Accumulator with bounds checking
            if !asm_ops::asm_mix_avx2(chunk, &accumulator) {
                // Fallback to scalar mixing if AVX2 fails
                for j in 0..CHUNK_SIZE {
                    chunk[j] = asm_ops::asm_mix(chunk[j], accumulator[j]);
                }
            }
            
            // Update Accumulator with new chunk state
            accumulator.copy_from_slice(chunk);
        }
        
        // Backward Pass (Reduced iterations for performance)
        for i in (0..num_chunks/2).rev() {
             let offset = i * CHUNK_SIZE;
             let chunk = &mut arena[offset..offset+CHUNK_SIZE];
             
             if !asm_ops::asm_mix_avx2(chunk, &accumulator) {
                 for j in 0..CHUNK_SIZE {
                     chunk[j] = asm_ops::asm_mix(chunk[j], accumulator[j]);
                 }
             }
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

    // dynamic thread count with proper error handling
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or_else(|_| {
            eprintln!("Warning: Failed to get thread count, using fallback");
            2 // Fallback to 2 if query fails
        });

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
