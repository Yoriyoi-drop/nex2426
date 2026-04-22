//! Blockchain chain management
//! 
//! Manages the blockchain state, validation, and operations

use crate::blockchain::{Block, BlockchainConfig, BlockchainError, BlockchainResult, BlockchainStats};
use crate::blockchain::storage::ChainStorage;
use crate::blockchain::consensus::ConsensusEngine;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Chain state representing current blockchain status
#[derive(Debug, Clone)]
pub struct ChainState {
    /// Current chain height
    pub height: u64,
    /// Total work done in the chain
    pub total_work: u64,
    /// Last block hash
    pub last_hash: String,
    /// Unspent transaction outputs
    pub utxo_set: HashMap<String, Vec<u8>>,
    /// Mempool for pending transactions
    pub mempool: Vec<crate::blockchain::Transaction>,
}

impl ChainState {
    /// Create initial chain state
    pub fn new() -> Self {
        Self {
            height: 0,
            total_work: 0,
            last_hash: "0".repeat(64),
            utxo_set: HashMap::new(),
            mempool: Vec::new(),
        }
    }

    /// Update state with new block
    pub fn apply_block(&mut self, block: &Block) -> BlockchainResult<()> {
        self.height = block.get_height();
        self.last_hash = block.get_hash().unwrap_or(&"0".repeat(64)).clone();
        
        // Update total work (simplified calculation)
        self.total_work += block.header.difficulty as u64;
        
        // Process transactions
        for tx in &block.transactions {
            self.process_transaction(tx)?;
        }

        Ok(())
    }

    /// Process a single transaction
    fn process_transaction(&mut self, tx: &crate::blockchain::Transaction) -> BlockchainResult<()> {
        match tx.tx_type {
            crate::blockchain::TransactionType::Genesis => {
                // Genesis transaction creates initial UTXOs
                self.utxo_set.insert(tx.tx_id.clone(), tx.data.content.clone());
            }
            crate::blockchain::TransactionType::Encryption |
            crate::blockchain::TransactionType::Decryption |
            crate::blockchain::TransactionType::KeyExchange => {
                // Regular transactions update UTXO set
                self.utxo_set.insert(tx.tx_id.clone(), tx.data.content.clone());
            }
            _ => {
                // Handle other transaction types
                self.utxo_set.insert(tx.tx_id.clone(), tx.data.content.clone());
            }
        }
        Ok(())
    }

    /// Add transaction to mempool
    pub fn add_to_mempool(&mut self, tx: crate::blockchain::Transaction) -> BlockchainResult<()> {
        if self.mempool.len() >= 10000 {
            return Err(BlockchainError::ChainValidationFailed {
                reason: "Mempool is full".to_string(),
            });
        }
        
        // Verify transaction before adding
        if !tx.verify()? {
            return Err(BlockchainError::InvalidTransaction {
                reason: "Transaction verification failed".to_string(),
            });
        }
        
        self.mempool.push(tx);
        Ok(())
    }

    /// Get transactions from mempool for block creation
    pub fn get_mempool_transactions(&self, limit: usize) -> Vec<crate::blockchain::Transaction> {
        self.mempool.iter().take(limit).cloned().collect()
    }

    /// Remove processed transactions from mempool
    pub fn remove_from_mempool(&mut self, tx_ids: &[String]) {
        self.mempool.retain(|tx| !tx_ids.contains(&tx.tx_id));
    }
}

/// Main blockchain structure
pub struct Blockchain {
    /// Chain configuration
    config: BlockchainConfig,
    /// Current chain state
    state: Arc<RwLock<ChainState>>,
    /// Storage backend
    storage: Box<dyn ChainStorage>,
    /// Consensus engine
    consensus: Box<dyn ConsensusEngine>,
    /// Chain statistics
    stats: Arc<RwLock<BlockchainStats>>,
}

impl Blockchain {
    /// Create a new blockchain instance
    pub fn new(
        config: BlockchainConfig,
        storage: Box<dyn ChainStorage>,
        consensus: Box<dyn ConsensusEngine>,
    ) -> BlockchainResult<Self> {
        let blockchain = Self {
            config: config.clone(),
            state: Arc::new(RwLock::new(ChainState::new())),
            storage,
            consensus,
            stats: Arc::new(RwLock::new(BlockchainStats::new(config.chain_id.clone()))),
        };

        // Initialize with genesis block if needed
        blockchain.initialize()?;
        Ok(blockchain)
    }

    /// Initialize blockchain with genesis block
    fn initialize(&self) -> BlockchainResult<()> {
        let mut state = self.state.write()
            .map_err(|_| BlockchainError::ChainValidationFailed {
                reason: "Failed to acquire write lock".to_string(),
            })?;

        // Check if blockchain already exists
        if let Some(genesis) = self.storage.get_block(0)? {
            // Load existing blockchain state
            state.height = genesis.get_height();
            state.last_hash = genesis.get_hash().unwrap_or(&"0".repeat(64)).clone();
            
            // Update statistics
            let mut stats = self.stats.write()
                .map_err(|_| BlockchainError::ChainValidationFailed {
                    reason: "Failed to acquire stats lock".to_string(),
                })?;
            stats.total_blocks = state.height + 1;
            stats.last_block_time = genesis.get_timestamp();
        } else {
            // Create genesis block
            let genesis = Block::create_genesis(&self.config.chain_id)?;
            
            // Store genesis block
            self.storage.store_block(&genesis)?;
            
            // Update state
            state.apply_block(&genesis)?;
            
            // Update statistics
            let mut stats = self.stats.write()
                .map_err(|_| BlockchainError::ChainValidationFailed {
                    reason: "Failed to acquire stats lock".to_string(),
                })?;
            stats.total_blocks = 1;
            stats.last_block_time = genesis.get_timestamp();
            stats.difficulty = self.config.difficulty;
        }

        Ok(())
    }

    /// Add a new block to the blockchain
    pub fn add_block(&self, mut block: Block) -> BlockchainResult<()> {
        // Validate block
        if !self.validate_block(&block)? {
            return Err(BlockchainError::InvalidBlock {
                reason: "Block validation failed".to_string(),
            });
        }

        // Mine block if not already mined
        if !self.consensus.is_valid_proof(&block)? {
            block = self.consensus.mine_block(block)?;
        }

        // Store block
        self.storage.store_block(&block)?;

        // Update chain state
        {
            let mut state = self.state.write()
                .map_err(|_| BlockchainError::ChainValidationFailed {
                    reason: "Failed to acquire write lock".to_string(),
                })?;
            
            state.apply_block(&block)?;
            
            // Remove processed transactions from mempool
            let tx_ids: Vec<String> = block.transactions.iter()
                .map(|tx| tx.tx_id.clone())
                .collect();
            state.remove_from_mempool(&tx_ids);
        }

        // Update statistics
        {
            let mut stats = self.stats.write()
                .map_err(|_| BlockchainError::ChainValidationFailed {
                    reason: "Failed to acquire stats lock".to_string(),
                })?;
            
            stats.total_blocks = block.get_height() + 1;
            stats.total_transactions += block.get_tx_count() as u64;
            stats.last_block_time = block.get_timestamp();
            stats.difficulty = block.header.difficulty;
            stats.chain_weight += block.header.difficulty as u64;
        }

        Ok(())
    }

    /// Create and add a new block from mempool transactions
    pub fn create_block(&self) -> BlockchainResult<Block> {
        let state = self.state.read()
            .map_err(|_| BlockchainError::ChainValidationFailed {
                reason: "Failed to acquire read lock".to_string(),
            })?;

        // Get transactions from mempool
        let transactions = state.get_mempool_transactions(self.config.max_tx_per_block);
        
        if transactions.is_empty() {
            return Err(BlockchainError::InvalidBlock {
                reason: "No transactions in mempool".to_string(),
            });
        }

        // Create new block
        let block = Block::new(
            state.last_hash.clone(),
            transactions,
            self.config.difficulty,
            state.height + 1,
        )?;

        Ok(block)
    }

    /// Validate a block
    pub fn validate_block(&self, block: &Block) -> BlockchainResult<bool> {
        let state = self.state.read()
            .map_err(|_| BlockchainError::ChainValidationFailed {
                reason: "Failed to acquire read lock".to_string(),
            })?;

        // Check block height
        if block.get_height() != state.height + 1 {
            return Ok(false);
        }

        // Check previous hash
        if block.header.prev_hash != state.last_hash {
            return Ok(false);
        }

        // Verify block integrity
        if !block.verify()? {
            return Ok(false);
        }

        // Verify consensus
        if !self.consensus.is_valid_proof(block)? {
            return Ok(false);
        }

        // Verify all transactions
        for tx in &block.transactions {
            if !tx.verify()? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Add transaction to mempool
    pub fn add_transaction(&self, tx: crate::blockchain::Transaction) -> BlockchainResult<()> {
        let mut state = self.state.write()
            .map_err(|_| BlockchainError::ChainValidationFailed {
                reason: "Failed to acquire write lock".to_string(),
            })?;
        
        state.add_to_mempool(tx)
    }

    /// Get block by height
    pub fn get_block(&self, height: u64) -> BlockchainResult<Option<Block>> {
        self.storage.get_block(height)
    }

    /// Get block by hash
    pub fn get_block_by_hash(&self, hash: &str) -> BlockchainResult<Option<Block>> {
        self.storage.get_block_by_hash(hash)
    }

    /// Get current chain height
    pub fn get_height(&self) -> BlockchainResult<u64> {
        let state = self.state.read()
            .map_err(|_| BlockchainError::ChainValidationFailed {
                reason: "Failed to acquire read lock".to_string(),
            })?;
        Ok(state.height)
    }

    /// Get last block hash
    pub fn get_last_hash(&self) -> BlockchainResult<String> {
        let state = self.state.read()
            .map_err(|_| BlockchainError::ChainValidationFailed {
                reason: "Failed to acquire read lock".to_string(),
            })?;
        Ok(state.last_hash.clone())
    }

    /// Get blockchain statistics
    pub fn get_stats(&self) -> BlockchainResult<BlockchainStats> {
        let stats = self.stats.read()
            .map_err(|_| BlockchainError::ChainValidationFailed {
                reason: "Failed to acquire stats lock".to_string(),
            })?;
        Ok(stats.clone())
    }

    /// Get mempool size
    pub fn get_mempool_size(&self) -> BlockchainResult<usize> {
        let state = self.state.read()
            .map_err(|_| BlockchainError::ChainValidationFailed {
                reason: "Failed to acquire read lock".to_string(),
            })?;
        Ok(state.mempool.len())
    }

    /// Validate entire chain
    pub fn validate_chain(&self) -> BlockchainResult<bool> {
        let current_height = self.get_height()?;
        
        for height in 0..=current_height {
            if let Some(block) = self.get_block(height)? {
                if !self.validate_block(&block)? {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
