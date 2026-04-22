//! NEX2426 Blockchain Module
//! 
//! This module implements a quantum-resistant blockchain system using NEX2426 hashing
//! for enhanced security and integrity verification.

pub mod block;
pub mod chain;
pub mod transaction;
pub mod consensus;
pub mod storage;
pub mod crypto;

// Re-export main components
pub use block::{Block, BlockHeader, Transaction, TransactionType, EncryptedData};
pub use chain::{Blockchain, ChainState};
pub use consensus::{ProofOfWork, ConsensusEngine};
pub use storage::{ChainStorage, MemoryStorage};
pub use crypto::{BlockchainCrypto, MerkleTree};

use crate::kernel::NexKernel;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Blockchain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    /// Chain identifier
    pub chain_id: String,
    /// Difficulty for proof-of-work
    pub difficulty: u32,
    /// Block size limit in bytes
    pub max_block_size: usize,
    /// Block time target in seconds
    pub block_time_target: u64,
    /// Maximum number of transactions per block
    pub max_tx_per_block: usize,
    /// Enable quantum-resistant features
    pub quantum_resistant: bool,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self {
            chain_id: "nex2426-main".to_string(),
            difficulty: 4, // Starting difficulty
            max_block_size: 1024 * 1024, // 1MB
            block_time_target: 600, // 10 minutes
            max_tx_per_block: 1000,
            quantum_resistant: true,
        }
    }
}

/// Blockchain error types
#[derive(Debug, thiserror::Error)]
pub enum BlockchainError {
    #[error("Invalid block: {reason}")]
    InvalidBlock { reason: String },
    
    #[error("Invalid transaction: {reason}")]
    InvalidTransaction { reason: String },
    
    #[error("Chain validation failed: {reason}")]
    ChainValidationFailed { reason: String },
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Consensus error: {0}")]
    ConsensusError(String),
    
    #[error("Crypto error: {0}")]
    CryptoError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result type for blockchain operations
pub type BlockchainResult<T> = Result<T, BlockchainError>;

/// Blockchain statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStats {
    pub chain_id: String,
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub difficulty: u32,
    pub hash_rate: f64,
    pub last_block_time: u64,
    pub chain_weight: u64,
}

impl BlockchainStats {
    pub fn new(chain_id: String) -> Self {
        Self {
            chain_id,
            total_blocks: 0,
            total_transactions: 0,
            difficulty: 4,
            hash_rate: 0.0,
            last_block_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            chain_weight: 0,
        }
    }
}
