//! Block structure for NEX2426 Blockchain
//! 
//! Implements quantum-resistant blocks using NEX2426 hashing algorithm

use crate::blockchain::{BlockchainError, BlockchainResult};
use crate::kernel::NexKernel;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Block header containing metadata and hash references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block version
    pub version: u32,
    /// Previous block hash
    pub prev_hash: String,
    /// Merkle root of transactions
    pub merkle_root: String,
    /// Block timestamp
    pub timestamp: u64,
    /// Difficulty target
    pub difficulty: u32,
    /// Nonce for proof-of-work
    pub nonce: u64,
    /// Block height
    pub height: u64,
    /// Quantum-resistant signature
    pub quantum_signature: Option<String>,
    /// Hash of the block header
    pub hash: Option<String>,
}

impl BlockHeader {
    /// Create a new block header
    pub fn new(
        prev_hash: String,
        merkle_root: String,
        difficulty: u32,
        height: u64,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();

        Self {
            version: 1,
            prev_hash,
            merkle_root,
            timestamp,
            difficulty,
            nonce: 0,
            height,
            quantum_signature: None,
            hash: None,
        }
    }

    /// Calculate hash of the block header using NEX2426
    pub fn calculate_hash(&mut self) -> BlockchainResult<String> {
        let kernel = NexKernel::new(1);
        
        // Create header data for hashing
        let header_data = format!(
            "{}{}{}{}{}{}{}{}",
            self.version,
            self.prev_hash,
            self.merkle_root,
            self.timestamp,
            self.difficulty,
            self.nonce,
            self.height,
            self.quantum_signature.as_deref().unwrap_or("")
        );

        let result = kernel.hash_bytes(header_data.as_bytes(), "block-header");
        let hash_hex = crate::integrity::merkle::hex_util::encode(&result);
        
        self.hash = Some(hash_hex.clone());
        Ok(hash_hex)
    }

    /// Verify block header integrity
    pub fn verify(&self) -> BlockchainResult<bool> {
        if self.hash.is_none() {
            return Err(BlockchainError::InvalidBlock {
                reason: "Block hash not calculated".to_string(),
            });
        }

        // Recreate header without hash for verification
        let mut temp_header = self.clone();
        temp_header.hash = None;
        temp_header.calculate_hash()?;

        Ok(temp_header.hash == self.hash)
    }
}

/// Block containing transactions and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,
    /// List of transactions
    pub transactions: Vec<Transaction>,
    /// Block size in bytes
    pub size: usize,
}

impl Block {
    /// Create a new genesis block
    pub fn create_genesis(chain_id: &str) -> BlockchainResult<Self> {
        let genesis_tx = Transaction::create_genesis(chain_id)?;
        let merkle_root = crate::blockchain::crypto::MerkleTree::new(vec![&genesis_tx])
            .get_root_hash();

        let mut header = BlockHeader::new(
            "0".repeat(64), // Genesis has no previous hash
            merkle_root,
            4, // Starting difficulty
            0, // Genesis block height
        );
        
        header.calculate_hash()?;

        let mut block = Self {
            header,
            transactions: vec![genesis_tx],
            size: 0,
        };

        block.calculate_size()?;
        Ok(block)
    }

    /// Create a new block with transactions
    pub fn new(
        prev_hash: String,
        transactions: Vec<Transaction>,
        difficulty: u32,
        height: u64,
    ) -> BlockchainResult<Self> {
        if transactions.is_empty() {
            return Err(BlockchainError::InvalidBlock {
                reason: "Block must contain at least one transaction".to_string(),
            });
        }

        // Calculate merkle root
        let tx_refs: Vec<&Transaction> = transactions.iter().collect();
        let merkle_root = crate::blockchain::crypto::MerkleTree::new(tx_refs)
            .get_root_hash();

        let mut header = BlockHeader::new(prev_hash, merkle_root, difficulty, height);
        header.calculate_hash()?;

        let mut block = Self {
            header,
            transactions,
            size: 0,
        };

        block.calculate_size()?;
        Ok(block)
    }

    /// Calculate block size
    fn calculate_size(&mut self) -> BlockchainResult<()> {
        let serialized = serde_json::to_string(self)
            .map_err(|e| BlockchainError::SerializationError(e.to_string()))?;
        self.size = serialized.len();
        Ok(())
    }

    /// Add transaction to block
    pub fn add_transaction(&mut self, tx: Transaction) -> BlockchainResult<()> {
        self.transactions.push(tx);
        
        // Recalculate merkle root
        let tx_refs: Vec<&Transaction> = self.transactions.iter().collect();
        self.header.merkle_root = crate::blockchain::crypto::MerkleTree::new(tx_refs)
            .get_root_hash();
        
        // Recalculate header hash
        self.header.calculate_hash()?;
        
        // Update block size
        self.calculate_size()?;
        
        Ok(())
    }

    /// Verify block integrity
    pub fn verify(&self) -> BlockchainResult<bool> {
        // Verify header
        if !self.header.verify()? {
            return Ok(false);
        }

        // Verify merkle root
        let tx_refs: Vec<&Transaction> = self.transactions.iter().collect();
        let calculated_merkle = crate::blockchain::crypto::MerkleTree::new(tx_refs)
            .get_root_hash();
        
        if calculated_merkle != self.header.merkle_root {
            return Ok(false);
        }

        // Verify all transactions
        for tx in &self.transactions {
            if !tx.verify()? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get block hash
    pub fn get_hash(&self) -> Option<&String> {
        self.header.hash.as_ref()
    }

    /// Get block height
    pub fn get_height(&self) -> u64 {
        self.header.height
    }

    /// Get block timestamp
    pub fn get_timestamp(&self) -> u64 {
        self.header.timestamp
    }

    /// Get number of transactions
    pub fn get_tx_count(&self) -> usize {
        self.transactions.len()
    }
}

/// Transaction reference for use in blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction ID (hash)
    pub tx_id: String,
    /// Transaction type
    pub tx_type: TransactionType,
    /// Sender address
    pub sender: String,
    /// Recipient address (if applicable)
    pub recipient: Option<String>,
    /// Transaction data
    pub data: EncryptedData,
    /// Transaction timestamp
    pub timestamp: u64,
    /// Transaction signature
    pub signature: Option<String>,
    /// Quantum-resistant signature
    pub quantum_signature: Option<String>,
}

/// Transaction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    /// Genesis transaction
    Genesis,
    /// Data encryption transaction
    Encryption,
    /// Data decryption transaction
    Decryption,
    /// Key exchange transaction
    KeyExchange,
    /// Smart contract transaction
    SmartContract,
    /// Custom transaction
    Custom(String),
}

/// Encrypted data payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Encrypted content
    pub content: Vec<u8>,
    /// Encryption key reference
    pub key_ref: String,
    /// Encryption algorithm
    pub algorithm: String,
    /// Initialization vector
    pub iv: Option<Vec<u8>>,
}

impl Transaction {
    /// Create a genesis transaction
    pub fn create_genesis(chain_id: &str) -> BlockchainResult<Self> {
        let kernel = NexKernel::new(1);
        let genesis_data = format!("GENESIS:{}", chain_id);
        // Generate secure random genesis transaction key
        use crate::utils::entropy::SecureRng;
        let genesis_key = if let Ok(mut rng) = SecureRng::new() {
            let mut key_bytes = [0u8; 8];
            if rng.fill_bytes(&mut key_bytes).is_ok() {
                format!("genesis-{}", hex::encode(key_bytes))
            } else {
                "genesis-tx-fallback".to_string()
            }
        } else {
            "genesis-tx-fallback".to_string()
        };
        let hash_result = kernel.hash_bytes(genesis_data.as_bytes(), &genesis_key);
        let tx_id = crate::integrity::merkle::hex_util::encode(&hash_result);

        Ok(Self {
            tx_id,
            tx_type: TransactionType::Genesis,
            sender: "0".repeat(64),
            recipient: None,
            data: EncryptedData {
                content: genesis_data.into_bytes(),
                key_ref: {
                    // Generate secure random genesis key
                    use crate::utils::entropy::SecureRng;
                    let mut rng = SecureRng::new().unwrap_or_else(|_| SecureRng::default());
                    let mut key_bytes = [0u8; 16];
                    rng.fill_bytes(&mut key_bytes).unwrap_or_else(|_| {
                        // Fallback to timestamp-based key
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        key_bytes.copy_from_slice(&timestamp.to_be_bytes());
                    });
                    format!("genesis-{}", hex::encode(key_bytes))
                },
                algorithm: "nex2426".to_string(),
                iv: None,
            },
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            signature: None,
            quantum_signature: None,
        })
    }

    /// Create a new transaction
    pub fn new(
        tx_type: TransactionType,
        sender: String,
        recipient: Option<String>,
        data: EncryptedData,
    ) -> BlockchainResult<Self> {
        let kernel = NexKernel::new(1);
        
        // Create transaction data for hashing
        let tx_data = format!(
            "{:?}{}{}{}{}",
            tx_type,
            sender,
            recipient.as_deref().unwrap_or(""),
            String::from_utf8_lossy(&data.content),
            data.key_ref
        );

        // Generate secure random transaction hash key
        use crate::utils::entropy::SecureRng;
        let tx_hash_key = if let Ok(mut rng) = SecureRng::new() {
            let mut key_bytes = [0u8; 8];
            if rng.fill_bytes(&mut key_bytes).is_ok() {
                format!("tx-{}", hex::encode(key_bytes))
            } else {
                "transaction-fallback".to_string()
            }
        } else {
            "transaction-fallback".to_string()
        };
        let hash_result = kernel.hash_bytes(tx_data.as_bytes(), &tx_hash_key);
        let tx_id = crate::integrity::merkle::hex_util::encode(&hash_result);

        Ok(Self {
            tx_id,
            tx_type,
            sender,
            recipient,
            data,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            signature: None,
            quantum_signature: None,
        })
    }

    /// Verify transaction integrity
    pub fn verify(&self) -> BlockchainResult<bool> {
        // Recalculate transaction hash
        let kernel = NexKernel::new(1);
        
        let tx_data = format!(
            "{:?}{}{}{}{}",
            self.tx_type,
            self.sender,
            self.recipient.as_deref().unwrap_or(""),
            String::from_utf8_lossy(&self.data.content),
            self.data.key_ref
        );

        // Generate secure random transaction hash key
        use crate::utils::entropy::SecureRng;
        let tx_hash_key = if let Ok(mut rng) = SecureRng::new() {
            let mut key_bytes = [0u8; 8];
            if rng.fill_bytes(&mut key_bytes).is_ok() {
                format!("tx-{}", hex::encode(key_bytes))
            } else {
                "transaction-fallback".to_string()
            }
        } else {
            "transaction-fallback".to_string()
        };
        let hash_result = kernel.hash_bytes(tx_data.as_bytes(), &tx_hash_key);
        let calculated_id = crate::integrity::merkle::hex_util::encode(&hash_result);

        Ok(calculated_id == self.tx_id)
    }
}
