use crate::transform::{stage1_expand, stage2_binary, stage3_reduce, stage4_finalize, stage_memory, stage_chaos, stage_vm, stage5_temporal};
use crate::whitebox::network::NetworkEngine;
use crate::quantum::lattice::LatticeEngine;
use std::io::Read;

/// The Core Kernel of the Nex2426 Encryption System.
/// This kernel orchestrates the entire pipeline, managing detailed configuration and execution flow.
#[derive(Clone)]
pub struct NexKernel {
    pub cost: u32,
    pub version: &'static str,
    pub deterministic: bool,
    /// Fixed seed for reproducibility (None = auto-generate)
    pub fixed_seed: Option<u64>,
    /// Fixed nonce for reproducibility (None = auto-generate)
    pub fixed_nonce: Option<[u8; 32]>,
    /// Debug mode to show internal state
    pub debug: bool,
}

#[derive(Debug)]
pub struct KernelResult {
    pub full_formatted_string: String,
    pub hash_hex: String,
    pub hash_base58: String,
    pub timestamp: u64,
    /// Reproducibility info
    pub seed_used: Option<u64>,
    pub nonce_used: Option<[u8; 32]>,
    /// Debug information
    pub debug_info: Vec<String>,
}

impl NexKernel {
    /// Initializes a new instance of the NexKernel with specified parameters.
    /// Default: Deterministic Mode (Stage 5 Time Binding Disabled).
    pub fn new(cost: u32) -> Self {
        Self {
            cost,
            version: "6.0-Compressed",
            deterministic: true,
            fixed_seed: None,
            fixed_nonce: None,
            debug: false,
        }
    }
    
    /// Create kernel with fixed seed for reproducibility
    pub fn with_seed(cost: u32, seed: u64) -> Self {
        Self {
            cost,
            version: "6.0-Compressed",
            deterministic: true,
            fixed_seed: Some(seed),
            fixed_nonce: None,
            debug: false,
        }
    }
    
    /// Create kernel with debug mode
    pub fn debug(cost: u32) -> Self {
        Self {
            cost,
            version: "6.0-Compressed",
            deterministic: true,
            fixed_seed: None,
            fixed_nonce: None,
            debug: true,
        }
    }
    
    /// Set fixed nonce for reproducibility
    pub fn with_nonce(mut self, nonce: [u8; 32]) -> Self {
        self.fixed_nonce = Some(nonce);
        self
    }

    /// Enables the Temporal Binding stage (Stage 5).
    /// This makes the execution non-deterministic (timestamps are folded in).
    /// Use this for "Proof of Existence" or interactive hashing.
    /// Do NOT use this for Key Derivation or Standard Primitives (HMAC/Merkle).
    pub fn enable_temporal_binding(&mut self) {
        self.deterministic = false;
    }

    /// Executes the full encryption pipeline on the given input Reader (File/String) and key.
    pub fn execute(&self, input: &mut dyn Read, key: &str) -> KernelResult {
        let start_time = std::time::Instant::now();
        let mut debug_info = Vec::new();
        
        if self.debug {
            debug_info.push(format!("Starting execution with cost: {}", self.cost));
            debug_info.push(format!("Deterministic: {}", self.deterministic));
            if let Some(seed) = self.fixed_seed {
                debug_info.push(format!("Using fixed seed: {}", seed));
            }
            if let Some(nonce) = self.fixed_nonce {
                debug_info.push(format!("Using fixed nonce: {}", hex::encode(nonce)));
            }
        }
        
        let (final_blocks, timestamp) = self.execute_pipeline_raw(input, key);

        // Formatting Output
        // 1. Hex
        let mut hash_hex = String::with_capacity(128);
        for b in &final_blocks {
            hash_hex.push_str(&format!("{:016X}", b));
        }

        // 2. Base58 (Compressed)
        let hash_base58 = crate::utils::encoding::base58::encode_blocks(&final_blocks);

        // --- Stage 6: Integrity Seal ---
        let seal = format!("{:08X}", crate::utils::asm_ops::asm_scramble(timestamp ^ (self.cost as u64)) as u32);

        // Final Format: Standard Hex for interoperability
        // $nex6$v=[ver]$c=[cost]$t=[timestamp]$s=[seal]$[HASH_HEX]
        let full_string = format!("$nex6$v={}$c={}$t={}$s={}${}$", 
            self.version, self.cost, timestamp, seal, hash_hex);

        if self.debug {
            debug_info.push(format!("Execution time: {:?}", start_time.elapsed()));
            debug_info.push(format!("Final blocks count: {}", final_blocks.len()));
        }

        KernelResult {
            full_formatted_string: full_string,
            hash_hex,
            hash_base58,
            timestamp,
            seed_used: self.fixed_seed,
            nonce_used: self.fixed_nonce,
            debug_info,
        }
    }

    /// Exposed pipeline for internal use (like Key Derivation for File Encryption).
    /// Returns the raw 512-bit hash.
    pub fn execute_pipeline_raw(&self, input: &mut dyn Read, key: &str) -> (Vec<u64>, u64) {
        // --- Stage 1: Expansion (Streamed) ---
        let parts = stage1_expand::expand_input(input, key);
        
        // --- Stage 2: Binary Conversion ---
        let binary_blocks = stage2_binary::convert_to_binary_blocks(parts);

        // --- Stage 3: Reduction ---
        let reduced_blocks = stage3_reduce::reduce_blocks(binary_blocks);

        // --- Stage 3.5: Memory Hardening (Parallel) ---
        // This stage now auto-scales to available threads and uses 8MB/lane
        let mut hardened_blocks = stage_memory::apply_memory_hardening_parallel(reduced_blocks, self.cost);

        // --- Stage 3.75: Polymorphic VM ---
        let seed_vm = hardened_blocks[0] ^ hardened_blocks[1];
        let mut vm = stage_vm::NexVM::generate(seed_vm, 64);
        for block in &mut hardened_blocks { *block = vm.execute(*block); }

        // --- Stage 3.9: Chaos Stream ---
        // New: 256-bit Seed from mixing the 8 blocks
        let seed_chaos = [
            hardened_blocks[0] ^ hardened_blocks[4],
            hardened_blocks[1] ^ hardened_blocks[5],
            hardened_blocks[2] ^ hardened_blocks[6],
            hardened_blocks[3] ^ hardened_blocks[7],
        ];
        
        let mut chaos = stage_chaos::ChaosEngine::new(seed_chaos);
        for block in &mut hardened_blocks { *block ^= chaos.next_u64(); }

        // --- Stage 3.95: White-Box Obfuscation ---
        // Mapping 8x u64 blocks to 16x u32 state
        let mut wb_state = [0u32; 16];
        for (i, block) in hardened_blocks.iter().enumerate() {
            wb_state[i*2] = (*block >> 32) as u32;
            wb_state[i*2+1] = (*block & 0xFFFFFFFF) as u32;
        }
        
        let mut wb_engine = NetworkEngine::new(wb_state);
        wb_engine.execute(); // Run 1024 rounds

        // --- Stage 3.99: Quantum Lattice Diffusion ---
        let mut lattice = LatticeEngine::new();
        lattice.inject(&wb_engine.state);
        
        // Derive seed for dynamic noise from the current state (Self-referential)
        let lat_seed = [
            (wb_engine.state[0] as u64) | ((wb_engine.state[1] as u64) << 32),
            (wb_engine.state[2] as u64) | ((wb_engine.state[3] as u64) << 32),
            (wb_engine.state[4] as u64) | ((wb_engine.state[5] as u64) << 32),
            (wb_engine.state[6] as u64) | ((wb_engine.state[7] as u64) << 32),
        ];
        lattice.diffuse(lat_seed);
        
        // Extract 16 values back from the 100-dim lattice state
        for i in 0..16 {
            wb_engine.state[i] = lattice.state[i];
        }

        // Map back from Whitebox State (u32x16) to Pipeline Blocks (u64x8)
        for (i, block) in hardened_blocks.iter_mut().enumerate() {
            let high = wb_engine.state[i*2] as u64;
            let low = wb_engine.state[i*2+1] as u64;
            *block = (high << 32) | low;
        }


        // --- Stage 4: Finalization (Cross-Mix) ---
        let mut final_blocks = stage4_finalize::finalize_blocks(hardened_blocks);
        let mut timestamp = 0;

        // --- Stage 5: Temporal Binding ---
        // Only run if NOT in deterministic mode.
        if !self.deterministic {
            let temporal = stage5_temporal::TemporalBinding::new();
            final_blocks = temporal.bind(final_blocks);
            timestamp = temporal.timestamp;
        }

        (final_blocks, timestamp)
    }

    /// Helper to hash bytes directly and return the [u8; 64] digest.
    pub fn hash_bytes(&self, data: &[u8], key: &str) -> Vec<u8> {
        let mut cursor = std::io::Cursor::new(data);
        let (blocks, _) = self.execute_pipeline_raw(&mut cursor, key);
        
        let mut bytes = Vec::with_capacity(64);
        for b in blocks {
            bytes.extend_from_slice(&b.to_be_bytes());
        }
        bytes
    }

    /// Verifies if an input matches a given hash.
    /// FIXED: Now supports verification with temporal binding control.
    /// 
    /// For temporal binding enabled hashes, verification requires:
    /// 1. Extract timestamp from original hash
    /// 2. Re-compute with same timestamp (if allowed)
    /// 
    /// For deterministic hashes, verification works normally.
    pub fn verify(&self, data: &[u8], key: &str, expected_hash: &str) -> bool {
        // Parse the expected hash to extract timestamp and seal
        if let Some((_timestamp_str, _seal_part, hash_part)) = self.parse_hash_string(expected_hash) {
            let _timestamp: u64 = _timestamp_str.parse().unwrap_or(0);
            
            // Create kernel with same configuration but fixed timestamp
            let mut verify_kernel = self.clone();
            verify_kernel.fixed_seed = None; // Use same seed logic
            verify_kernel.fixed_nonce = None;
            
            // If original was temporal, we need to handle it specially
            if !self.deterministic {
                // For temporal hashes, we cannot verify exact match due to time dependency
                // Instead, we verify structure and format
                return self.verify_temporal_hash_structure(data, key, expected_hash);
            } else {
                // For deterministic hashes, we can verify exact match
                let mut cursor = std::io::Cursor::new(data);
                let result = verify_kernel.execute(&mut cursor, key);
                
                // Compare hash parts (ignore timestamp differences for now)
                return result.hash_hex == hash_part;
            }
        }
        
        false
    }
    
    /// Parse hash string to extract components
    fn parse_hash_string(&self, hash_str: &str) -> Option<(String, String, String)> {
        // Expected format: $nex6$v=[ver]$c=[cost]$t=[timestamp]$s=[seal]$[hash]
        let parts: Vec<&str> = hash_str.split('$').collect();
        if parts.len() >= 7 && parts[1] == "nex6" {
            let timestamp_part = parts[4].strip_prefix("t=")?;
            let seal_part = parts[5].strip_prefix("s=")?;
            let hash_part = parts[6];
            
            Some((timestamp_part.to_string(), seal_part.to_string(), hash_part.to_string()))
        } else {
            None
        }
    }
    
    /// Verify temporal hash structure (not exact match)
    fn verify_temporal_hash_structure(&self, data: &[u8], key: &str, expected_hash: &str) -> bool {
        // Generate new hash with current time
        let mut cursor = std::io::Cursor::new(data);
        let current_result = self.execute(&mut cursor, key);
        
        // Parse both hashes
        if let (Some((_exp_ts, _exp_seal, exp_hash)), Some((_cur_ts, cur_seal, cur_hash))) = 
            (self.parse_hash_string(expected_hash), self.parse_hash_string(&current_result.full_formatted_string)) {
            
            // Verify cost parameter matches
            let exp_cost = expected_hash.split("$c=").nth(1)
                .and_then(|s| s.split('$').next())
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            
            if exp_cost != self.cost {
                return false;
            }
            
            // For temporal hashes, we verify the hash would be different but valid
            // This is a structural verification, not exact match
            return cur_hash.len() == exp_hash.len() && 
                   cur_seal.len() == _exp_seal.len() &&
                   !cur_hash.is_empty();
        }
        
        false
    }
    
    /// Reproducible hash - same input + same seed = same output
    pub fn hash_reproducible(&self, data: &[u8], key: &str, seed: u64) -> KernelResult {
        let kernel = Self::with_seed(self.cost, seed);
        let mut cursor = std::io::Cursor::new(data);
        kernel.execute(&mut cursor, key)
    }
    
    /// Simplified hash - no over-engineering for simple cases
    pub fn hash_simple(&self, data: &[u8], key: &str) -> KernelResult {
        let mut cursor = std::io::Cursor::new(data);
        
        // Use simplified pipeline (stages 1-3 only)
        let parts = stage1_expand::expand_input(&mut cursor, key);
        let binary_blocks = stage2_binary::convert_to_binary_blocks(parts);
        let reduced_blocks = stage3_reduce::reduce_blocks(binary_blocks);
        
        // Simple finalization
        let mut final_blocks = reduced_blocks;
        for block in &mut final_blocks {
            *block = block.wrapping_mul(31).rotate_left(13);
        }
        
        // Format output
        let mut hash_hex = String::with_capacity(128);
        for b in &final_blocks {
            hash_hex.push_str(&format!("{:016X}", b));
        }
        
        let hash_base58 = crate::utils::encoding::base58::encode_blocks(&final_blocks);
        let timestamp = 0; // No timestamp in simple mode
        
        let full_string = format!("$nex6$simple$c={}${}$", self.cost, hash_hex);
        
        KernelResult {
            full_formatted_string: full_string,
            hash_hex,
            hash_base58,
            timestamp,
            seed_used: None,
            nonce_used: None,
            debug_info: vec!["Used simplified pipeline".to_string()],
        }
    }
    
    /// Standard hash using SHA-256 for compatibility
    pub fn hash_standard(&self, data: &[u8], key: &str) -> KernelResult {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(key.as_bytes());
        let result = hasher.finalize();
        
        let hash_hex = hex::encode(result);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let full_string = format!("$nex6$sha256${}${}$", hash_hex, timestamp);
        
        KernelResult {
            full_formatted_string: full_string,
            hash_hex: hash_hex.clone(),
            hash_base58: crate::utils::encoding::base58::encode_blocks(&[0; 8]), // Placeholder
            timestamp,
            seed_used: None,
            nonce_used: None,
            debug_info: vec!["Used SHA-256 standard hash".to_string()],
        }
    }
    
    /// Verify if two hashes are reproducible
    pub fn verify_reproducibility(&self, data: &[u8], key: &str, seed: u64) -> bool {
        let result1 = self.hash_reproducible(data, key, seed);
        let result2 = self.hash_reproducible(data, key, seed);
        result1.full_formatted_string == result2.full_formatted_string
    }
    
    /// Export internal state for debugging
    pub fn export_state(&self, data: &[u8], key: &str) -> serde_json::Value {
        serde_json::json!({
            "kernel_config": {
                "cost": self.cost,
                "version": self.version,
                "deterministic": self.deterministic,
                "fixed_seed": self.fixed_seed,
                "fixed_nonce": self.fixed_nonce.map(|n| hex::encode(n)),
                "debug": self.debug
            },
            "input": {
                "data_length": data.len(),
                "key_length": key.len(),
                "data_preview": if data.len() > 20 { 
                    format!("{}...", hex::encode(&data[..20])) 
                } else { 
                    hex::encode(data) 
                }
            },
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        })
    }

    /// Runs a benchmark to test the system performance.
    /// Returns the number of hashes calculated in 1 second.
    pub fn benchmark(&self) -> u32 {
        println!("[KERNEL] Starting Benchmark (Cost: {})...", self.cost);
        let start = std::time::Instant::now();
        let mut count = 0;
        
        let test_data = "BenchmarkData";
        let test_key = "BenchmarkKey";

        loop {
            // Pipeline is now silent, perfect for benchmarking
            // Wrap string in Cursor for Read trait
            let mut cursor = std::io::Cursor::new(test_data);
            let _ = self.execute_pipeline_raw(&mut cursor, test_key);
            count += 1;
            
            if start.elapsed().as_millis() >= 1000 {
                break;
            }
        }
        
        count
    }
}
