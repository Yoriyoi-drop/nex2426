use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use crate::kernel::NexKernel;

/// Parallel Crypto Engine
/// Optimized for handling massive files (> 1TB) by utilizing all CPU cores.
/// 
/// Strategy:
/// 1. Main Thread reads File into Chunks.
/// 2. Chunks are sent to Thread Pool.
/// 3. Threads generate Keystream (Computationally Expensive) and XOR.
/// 4. Results are gathered and written sequentially.
///
/// Note: To keep order output correct, we need a reordering buffer or handle strictly.
/// For simplicity and memory safety on 1TB files, we use a "Prefetch Keystream" approach.
/// 
/// approach: "Keystream Producer -> Data Consumer"
/// Workers generate the NexKernel Hash Stream blocks in parallel.
/// Main thread simply XORs data with the streaming key from the channel.

pub struct ParallelStreamCipher {
    receiver: mpsc::Receiver<(u64, Vec<u8>)>,
}

impl ParallelStreamCipher {
    /// Spawns a generator pool that produces keystream bytes starting from `start_counter`.
    /// `nonce`: unique per file.
    pub fn new(seed_key: &str, nonce: &[u8; 32], start_counter: u64) -> Self {
        let (tx, rx) = mpsc::sync_channel(16); // Buffer ahead 16 blocks (keeps memory usage low)
        let key_string = seed_key.to_string();
        let nonce_copy = *nonce;
        let num_threads = 8; // Assumed 8 logical cores, can be dynamic
        
        // Block Manager Shared State
        let counter = Arc::new(Mutex::new(start_counter));
        
        for _ in 0..num_threads {
            let tx_clone = tx.clone();
            let key_clone = key_string.clone();
            let ctr_manager = counter.clone();
            
            thread::spawn(move || {
                let kernel = NexKernel::new(1); // Standard Cost
                loop {
                    // Claim a block index
                    let block_idx = {
                        let mut num = ctr_manager.lock().expect("Counter manager lock poisoned");
                        let this_val = *num;
                        *num += 1;
                        this_val
                    };
                    
                    // Generate Keystream Block (Expensive Part)
                    // Input = Nonce || Block_Index
                    let mut input = Vec::with_capacity(40);
                    input.extend_from_slice(&nonce_copy);
                    input.extend_from_slice(&block_idx.to_be_bytes());
                    
                    // Kernel outputs 512 bits (64 bytes) per round
                    // To be efficient, we might want to generate MORE per thread claim, 
                    // but for now 64 bytes is granular.
                    // WAIT: 64 bytes is too small for thread overhead.
                    // We should generate a "Super Chunk" of keystream, say 1MB.
                    
                    let chunk_size = 1024 * 1024; // 1 MB Keystream Chunk
                    let mut keystream = Vec::with_capacity(chunk_size);
                    
                    // We need `chunk_size / 64` Kernel calls
                    let rounds = chunk_size / 64;
                    
                    // Adjust the mathematical logical counter for the kernel calls
                    let base_inner_ctr = block_idx * (rounds as u64);
                    
                    for r in 0..rounds {
                        let inner_ctr = base_inner_ctr + (r as u64);
                        let mut inner_input = Vec::with_capacity(40);
                        inner_input.extend_from_slice(&nonce_copy);
                        inner_input.extend_from_slice(&inner_ctr.to_be_bytes());
                        
                        let cursor = &mut std::io::Cursor::new(inner_input);
                        let (blocks, _) = kernel.execute_pipeline_raw(cursor, &key_clone);
                        
                        for b in blocks {
                            keystream.extend_from_slice(&b.to_be_bytes());
                        }
                    }
                    
                    // Send result (Index, Data)
                    // Note: This might arrive out of order, receiver must handle reordering!
                    // OR: We use a simple 1-thread generator if 1TB is disk bound anyway.
                    // BUT: NexKernel is WHITEBOX + LATTICE + CHAOS. It is SLOW. CPU Bound.
                    // Reordering is mandatory.
                    
                    // Ideally we use a reordering buffer on Receiver. 
                    // To keep implementation strict without dependencies, we'll assume a simpler approach:
                    // Just return it, and let the receiver buffer it in a BTreeMap.
                    
                    if tx_clone.send((block_idx, keystream)).is_err() {
                        break; // Channel closed
                    }
                }
            });
        }
        
        Self { receiver: rx }
    }
    
    /// Get the specific keystream chunk. 
    /// Note: This simple implementation might block waiting for exact index.
    /// Real world: Reorder queue.
    pub fn get_stream_ordered(&self) -> impl Iterator<Item = Vec<u8>> + '_ {
        // A simple iterator that buffers incoming blocks and yields them in order
        StreamReorderer {
            rx: &self.receiver,
            buffer: std::collections::BTreeMap::new(),
            next_needed: 0,
        }
    }
}

struct StreamReorderer<'a> {
    rx: &'a mpsc::Receiver<(u64, Vec<u8>)>,
    buffer: std::collections::BTreeMap<u64, Vec<u8>>,
    next_needed: u64,
}

impl<'a> Iterator for StreamReorderer<'a> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(data) = self.buffer.remove(&self.next_needed) {
                self.next_needed += 1;
                return Some(data);
            }
            
            match self.rx.recv() {
                Ok((idx, data)) => {
                    self.buffer.insert(idx, data);
                },
                Err(_) => return None, // All senders disconnect
            }
        }
    }
}
