//! Consensus mechanisms for NEX2426 Blockchain
//! 
//! Implements proof-of-work and other consensus algorithms using quantum-resistant hashing

use crate::blockchain::{Block, BlockchainError, BlockchainResult};
use crate::kernel::NexKernel;
use std::time::SystemTime;

/// Consensus engine trait
pub trait ConsensusEngine: Send + Sync {
    /// Mine a block (find valid nonce)
    fn mine_block(&self, block: Block) -> BlockchainResult<Block>;
    
    /// Check if block has valid proof
    fn is_valid_proof(&self, block: &Block) -> BlockchainResult<bool>;
    
    /// Get current difficulty
    fn get_difficulty(&self) -> u32;
    
    /// Adjust difficulty based on block time
    fn adjust_difficulty(&mut self, last_block_time: u64, current_time: u64) -> BlockchainResult<u32>;
}

/// Proof-of-Work consensus engine
#[derive(Debug, Clone)]
pub struct ProofOfWork {
    /// Current difficulty
    difficulty: u32,
    /// Target block time in seconds
    block_time_target: u64,
    /// Maximum nonce value
    max_nonce: u64,
}

impl ProofOfWork {
    /// Create new PoW consensus engine
    pub fn new(difficulty: u32, block_time_target: u64) -> Self {
        Self {
            difficulty,
            block_time_target,
            max_nonce: u64::MAX,
        }
    }

    /// Calculate target hash based on difficulty
    fn calculate_target(&self) -> BlockchainResult<Vec<u8>> {
        // For NEX2426, we use a simpler target calculation
        // Higher difficulty = more leading zeros required
        let target_bytes = vec![0u8; (self.difficulty / 8) as usize];
        Ok(target_bytes)
    }

    /// Check if hash meets difficulty target
    fn hash_meets_target(&self, hash: &str) -> bool {
        // Convert hex hash to bytes for comparison
        if let Ok(hash_bytes) = hex::decode(hash) {
            let target = self.calculate_target().unwrap_or_default();
            
            // Check if hash is less than target (has enough leading zeros)
            if hash_bytes.len() >= target.len() {
                for i in 0..target.len() {
                    if hash_bytes[i] > target[i] {
                        return false;
                    }
                }
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Mine block with quantum-resistant hashing
    fn mine_block_internal(&self, mut block: Block) -> BlockchainResult<Block> {
        let _kernel = NexKernel::new(1);
        let start_time = SystemTime::now();
        
        println!("Starting mining for block {} with difficulty {}", 
                block.get_height(), self.difficulty);

        loop {
            // Update block header with current nonce
            block.header.nonce = block.header.nonce.wrapping_add(1);
            
            // Recalculate block hash
            block.header.calculate_hash()?;
            
            // Check if hash meets target
            if let Some(ref hash) = block.header.hash {
                if self.hash_meets_target(hash) {
                    let elapsed = start_time.elapsed()
                        .unwrap_or_else(|_| std::time::Duration::from_secs(0));
                    
                    println!("Block mined! Hash: {}, Nonce: {}, Time: {:.2}s", 
                            hash, block.header.nonce, elapsed.as_secs_f64());
                    break;
                }
            }

            // Prevent infinite loop
            if block.header.nonce >= self.max_nonce {
                return Err(BlockchainError::ConsensusError(
                    "Failed to find valid nonce within range".to_string()
                ));
            }
        }

        Ok(block)
    }
}

impl ConsensusEngine for ProofOfWork {
    fn mine_block(&self, block: Block) -> BlockchainResult<Block> {
        self.mine_block_internal(block)
    }

    fn is_valid_proof(&self, block: &Block) -> BlockchainResult<bool> {
        if let Some(ref hash) = block.header.hash {
            Ok(self.hash_meets_target(hash))
        } else {
            Ok(false)
        }
    }

    fn get_difficulty(&self) -> u32 {
        self.difficulty
    }

    fn adjust_difficulty(&mut self, last_block_time: u64, current_time: u64) -> BlockchainResult<u32> {
        let block_time = current_time.saturating_sub(last_block_time);
        
        if block_time > self.block_time_target * 2 {
            // Blocks are too slow, decrease difficulty
            self.difficulty = self.difficulty.saturating_sub(1);
        } else if block_time < self.block_time_target / 2 {
            // Blocks are too fast, increase difficulty
            self.difficulty = self.difficulty.saturating_add(1);
        }
        
        // Keep difficulty in reasonable range
        self.difficulty = self.difficulty.clamp(1, 32);
        
        println!("Difficulty adjusted to {} (block time: {}s)", 
                self.difficulty, block_time);
        
        Ok(self.difficulty)
    }
}

/// Quantum-enhanced proof-of-work using NEX2426 chaos encryption
#[derive(Debug, Clone)]
pub struct QuantumProofOfWork {
    base_pow: ProofOfWork,
    quantum_factor: u32,
}

impl QuantumProofOfWork {
    /// Create new quantum PoW consensus engine
    pub fn new(difficulty: u32, block_time_target: u64, quantum_factor: u32) -> Self {
        Self {
            base_pow: ProofOfWork::new(difficulty, block_time_target),
            quantum_factor,
        }
    }

    /// Apply quantum enhancement to hash
    fn apply_quantum_enhancement(&self, hash: &str) -> BlockchainResult<String> {
        let kernel = NexKernel::new(1);
        
        // Create quantum-enhanced hash input
        let quantum_input = format!("{}{}", hash, self.quantum_factor);
        let enhanced_result = kernel.hash_bytes(quantum_input.as_bytes(), "quantum-pow");
        
        Ok(crate::integrity::merkle::hex_util::encode(&enhanced_result))
    }
}

impl ConsensusEngine for QuantumProofOfWork {
    fn mine_block(&self, block: Block) -> BlockchainResult<Block> {
        let mut enhanced_block = block;
        
        loop {
            // Mine with base PoW
            enhanced_block = self.base_pow.mine_block_internal(enhanced_block)?;
            
            // Apply quantum enhancement
            if let Some(ref base_hash) = enhanced_block.header.hash {
                let quantum_hash = self.apply_quantum_enhancement(base_hash)?;
                
                // Check if quantum-enhanced hash meets target
                if self.base_pow.hash_meets_target(&quantum_hash) {
                    // Update block with quantum signature
                    enhanced_block.header.quantum_signature = Some(quantum_hash.clone());
                    enhanced_block.header.hash = Some(quantum_hash);
                    break;
                }
            }
            
            // Continue mining with next nonce
            enhanced_block.header.nonce = enhanced_block.header.nonce.wrapping_add(1);
        }

        Ok(enhanced_block)
    }

    fn is_valid_proof(&self, block: &Block) -> BlockchainResult<bool> {
        // Check base PoW validity
        if !self.base_pow.is_valid_proof(block)? {
            return Ok(false);
        }

        // Check quantum signature if present
        if let Some(ref quantum_sig) = block.header.quantum_signature {
            // Recreate quantum enhancement and verify
            if let Some(ref base_hash) = block.header.hash {
                let expected_quantum = self.apply_quantum_enhancement(base_hash)?;
                Ok(expected_quantum == *quantum_sig)
            } else {
                Ok(false)
            }
        } else {
            Ok(false) // Quantum signature required for quantum PoW
        }
    }

    fn get_difficulty(&self) -> u32 {
        self.base_pow.get_difficulty()
    }

    fn adjust_difficulty(&mut self, last_block_time: u64, current_time: u64) -> BlockchainResult<u32> {
        self.base_pow.adjust_difficulty(last_block_time, current_time)
    }
}

/// Proof-of-Stake consensus (placeholder for future implementation)
#[derive(Debug, Clone)]
pub struct ProofOfStake {
    /// Minimum stake required
    pub min_stake: u64,
    /// Validator set
    validators: Vec<String>,
}

impl ProofOfStake {
    /// Create new PoS consensus engine
    pub fn new(min_stake: u64) -> Self {
        Self {
            min_stake,
            validators: Vec::new(),
        }
    }

    /// Add validator
    pub fn add_validator(&mut self, validator: String) {
        self.validators.push(validator);
    }
}

impl ConsensusEngine for ProofOfStake {
    fn mine_block(&self, block: Block) -> BlockchainResult<Block> {
        // PoS doesn't mine in traditional sense
        // Select validator and create block
        // Note: In a real implementation, you would select a validator based on stake
        if self.validators.is_empty() {
            return Err(BlockchainError::ConsensusError("No validators available".to_string()));
        }
        
        let mut mined_block = block;
        mined_block.header.nonce = 1; // Fixed nonce for PoS
        mined_block.header.calculate_hash()?;
        Ok(mined_block)
    }

    fn is_valid_proof(&self, block: &Block) -> BlockchainResult<bool> {
        // For PoS, we check if block was created by valid validator
        // This is a simplified implementation that checks minimum stake concept
        Ok(block.header.hash.is_some() && self.min_stake > 0)
    }

    fn get_difficulty(&self) -> u32 {
        1 // PoS doesn't use traditional difficulty
    }

    fn adjust_difficulty(&mut self, _last_block_time: u64, _current_time: u64) -> BlockchainResult<u32> {
        Ok(1) // PoS doesn't adjust difficulty
    }
}

/// Hybrid consensus combining PoW and PoS
#[derive(Debug, Clone)]
pub struct HybridConsensus {
    pow: ProofOfWork,
    pos: ProofOfStake,
    pub pow_weight: f64, // Weight for PoW (0.0 to 1.0)
}

impl HybridConsensus {
    /// Create new hybrid consensus engine
    pub fn new(pow: ProofOfWork, pos: ProofOfStake, pow_weight: f64) -> Self {
        Self {
            pow,
            pos,
            pow_weight: pow_weight.clamp(0.0, 1.0),
        }
    }
}

impl ConsensusEngine for HybridConsensus {
    fn mine_block(&self, block: Block) -> BlockchainResult<Block> {
        // Use consensus mechanism based on weight
        if self.pow_weight > 0.5 {
            // Use PoW for now (simplified hybrid)
            self.pow.mine_block(block)
        } else {
            // Use PoS
            self.pos.mine_block(block)
        }
    }

    fn is_valid_proof(&self, block: &Block) -> BlockchainResult<bool> {
        // Check both PoW and PoS validity
        let pow_valid = self.pow.is_valid_proof(block)?;
        let pos_valid = self.pos.is_valid_proof(block)?;
        Ok(pow_valid && pos_valid)
    }

    fn get_difficulty(&self) -> u32 {
        // Adjust difficulty based on weight
        let base_difficulty = self.pow.get_difficulty();
        (base_difficulty as f64 * self.pow_weight) as u32
    }

    fn adjust_difficulty(&mut self, last_block_time: u64, current_time: u64) -> BlockchainResult<u32> {
        self.pow.adjust_difficulty(last_block_time, current_time)
    }
}
