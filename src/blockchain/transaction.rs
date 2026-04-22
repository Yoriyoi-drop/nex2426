//! Transaction system for NEX2426 Blockchain
//! 
//! Implements various transaction types for encryption operations and data management

use crate::blockchain::{BlockchainError, BlockchainResult};
use crate::blockchain::block::{EncryptedData, TransactionType};
use crate::blockchain::crypto::BlockchainCrypto;
use crate::kernel::NexKernel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Transaction structure (re-export from block module)
pub use crate::blockchain::block::Transaction;

/// Transaction builder for creating various types of transactions
#[derive(Debug, Clone)]
pub struct TransactionBuilder {
    tx_type: Option<TransactionType>,
    sender: Option<String>,
    recipient: Option<String>,
    data: Option<EncryptedData>,
    signature: Option<String>,
    quantum_signature: Option<String>,
}

impl TransactionBuilder {
    /// Create new transaction builder
    pub fn new() -> Self {
        Self {
            tx_type: None,
            sender: None,
            recipient: None,
            data: None,
            signature: None,
            quantum_signature: None,
        }
    }
    
    /// Set transaction type
    pub fn with_type(mut self, tx_type: TransactionType) -> Self {
        self.tx_type = Some(tx_type);
        self
    }
    
    /// Set sender address
    pub fn with_sender(mut self, sender: String) -> Self {
        self.sender = Some(sender);
        self
    }
    
    /// Set recipient address
    pub fn with_recipient(mut self, recipient: String) -> Self {
        self.recipient = Some(recipient);
        self
    }
    
    /// Set optional recipient address
    pub fn with_optional_recipient(mut self, recipient: Option<String>) -> Self {
        self.recipient = recipient;
        self
    }
    
    /// Set encrypted data
    pub fn with_data(mut self, data: EncryptedData) -> Self {
        self.data = Some(data);
        self
    }
    
    /// Set signature
    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
    }
    
    /// Set quantum signature
    pub fn with_quantum_signature(mut self, quantum_signature: String) -> Self {
        self.quantum_signature = Some(quantum_signature);
        self
    }
    
    /// Build transaction
    pub fn build(self) -> BlockchainResult<Transaction> {
        let tx_type = self.tx_type.ok_or_else(|| BlockchainError::InvalidTransaction {
            reason: "Transaction type is required".to_string(),
        })?;
        
        let sender = self.sender.ok_or_else(|| BlockchainError::InvalidTransaction {
            reason: "Sender is required".to_string(),
        })?;
        
        let data = self.data.ok_or_else(|| BlockchainError::InvalidTransaction {
            reason: "Transaction data is required".to_string(),
        })?;
        
        let mut tx = Transaction::new(tx_type, sender, self.recipient, data)?;
        tx.signature = self.signature;
        tx.quantum_signature = self.quantum_signature;
        
        Ok(tx)
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Encryption transaction for storing encrypted data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionTransaction {
    /// Original data hash
    pub data_hash: String,
    /// Encryption algorithm used
    pub algorithm: String,
    /// Key reference
    pub key_ref: String,
    /// Encrypted content
    pub encrypted_content: Vec<u8>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl EncryptionTransaction {
    /// Create new encryption transaction
    pub fn new(
        data: &[u8],
        key: &[u8],
        algorithm: String,
        metadata: HashMap<String, String>,
    ) -> BlockchainResult<Self> {
        let crypto = BlockchainCrypto::new();
        
        // Calculate data hash
        let kernel = NexKernel::new(1);
        let data_hash_result = kernel.hash_bytes(data, "data-hash");
        let data_hash = crate::integrity::merkle::hex_util::encode(&data_hash_result);
        
        // Encrypt data
        let encrypted_content = crypto.encrypt_data(data, key)?;
        
        // Generate key reference
        let key_ref = format!("key_{}", 
            crate::integrity::merkle::hex_util::encode(&kernel.hash_bytes(key, "key-ref"))[..16].to_string());
        
        Ok(Self {
            data_hash,
            algorithm,
            key_ref,
            encrypted_content,
            metadata,
        })
    }
    
    /// Convert to blockchain transaction
    pub fn to_transaction(&self, sender: String) -> BlockchainResult<Transaction> {
        let data = EncryptedData {
            content: self.encrypted_content.clone(),
            key_ref: self.key_ref.clone(),
            algorithm: self.algorithm.clone(),
            iv: None,
        };
        
        let mut metadata = self.metadata.clone();
        metadata.insert("data_hash".to_string(), self.data_hash.clone());
        
        TransactionBuilder::new()
            .with_type(TransactionType::Encryption)
            .with_sender(sender)
            .with_data(data)
            .build()
    }
    
    /// Decrypt transaction data
    pub fn decrypt(&self, key: &[u8]) -> BlockchainResult<Vec<u8>> {
        let crypto = BlockchainCrypto::new();
        crypto.decrypt_data(&self.encrypted_content, key)
    }
    
    /// Verify data integrity
    pub fn verify_integrity(&self, decrypted_data: &[u8]) -> BlockchainResult<bool> {
        let kernel = NexKernel::new(1);
        let current_hash_result = kernel.hash_bytes(decrypted_data, "data-hash");
        let current_hash = crate::integrity::merkle::hex_util::encode(&current_hash_result);
        Ok(current_hash == self.data_hash)
    }
}

/// Decryption transaction for authorized data access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptionTransaction {
    /// Reference to encryption transaction
    pub encryption_tx_id: String,
    /// Requester address
    pub requester: String,
    /// Access proof
    pub access_proof: Vec<u8>,
    /// Decryption purpose
    pub purpose: String,
}

impl DecryptionTransaction {
    /// Create new decryption transaction
    pub fn new(
        encryption_tx_id: String,
        requester: String,
        access_proof: Vec<u8>,
        purpose: String,
    ) -> Self {
        Self {
            encryption_tx_id,
            requester,
            access_proof,
            purpose,
        }
    }
    
    /// Convert to blockchain transaction
    pub fn to_transaction(&self, sender: String) -> BlockchainResult<Transaction> {
        let data = EncryptedData {
            content: self.access_proof.clone(),
            key_ref: self.encryption_tx_id.clone(),
            algorithm: "decryption-request".to_string(),
            iv: None,
        };
        
        let mut metadata = HashMap::new();
        metadata.insert("encryption_tx_id".to_string(), self.encryption_tx_id.clone());
        metadata.insert("requester".to_string(), self.requester.clone());
        metadata.insert("purpose".to_string(), self.purpose.clone());
        
        TransactionBuilder::new()
            .with_type(TransactionType::Decryption)
            .with_sender(sender)
            .with_data(data)
            .build()
    }
}

/// Key exchange transaction for secure communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExchangeTransaction {
    /// Public key
    pub public_key: Vec<u8>,
    /// Key exchange algorithm
    pub algorithm: String,
    /// Session identifier
    pub session_id: String,
    /// Expiration time
    pub expires_at: u64,
}

impl KeyExchangeTransaction {
    /// Create new key exchange transaction
    pub fn new(
        public_key: Vec<u8>,
        algorithm: String,
        session_id: String,
        expires_at: u64,
    ) -> Self {
        Self {
            public_key,
            algorithm,
            session_id,
            expires_at,
        }
    }
    
    /// Convert to blockchain transaction
    pub fn to_transaction(&self, sender: String) -> BlockchainResult<Transaction> {
        let data = EncryptedData {
            content: self.public_key.clone(),
            key_ref: self.session_id.clone(),
            algorithm: self.algorithm.clone(),
            iv: None,
        };
        
        let mut metadata = HashMap::new();
        metadata.insert("session_id".to_string(), self.session_id.clone());
        metadata.insert("expires_at".to_string(), self.expires_at.to_string());
        
        TransactionBuilder::new()
            .with_type(TransactionType::KeyExchange)
            .with_sender(sender)
            .with_data(data)
            .build()
    }
    
    /// Check if key exchange is still valid
    pub fn is_valid(&self) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();
        current_time < self.expires_at
    }
}

/// Smart contract transaction for programmable operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContractTransaction {
    /// Contract code
    pub code: String,
    /// Contract parameters
    pub parameters: Vec<String>,
    /// Gas limit
    pub gas_limit: u64,
    /// Contract address
    pub contract_address: String,
}

impl SmartContractTransaction {
    /// Create new smart contract transaction
    pub fn new(
        code: String,
        parameters: Vec<String>,
        gas_limit: u64,
        contract_address: String,
    ) -> Self {
        Self {
            code,
            parameters,
            gas_limit,
            contract_address,
        }
    }
    
    /// Convert to blockchain transaction
    pub fn to_transaction(&self, sender: String) -> BlockchainResult<Transaction> {
        let data = EncryptedData {
            content: self.code.as_bytes().to_vec(),
            key_ref: self.contract_address.clone(),
            algorithm: "smart-contract".to_string(),
            iv: None,
        };
        
        let mut metadata = HashMap::new();
        metadata.insert("contract_address".to_string(), self.contract_address.clone());
        metadata.insert("gas_limit".to_string(), self.gas_limit.to_string());
        metadata.insert("parameters".to_string(), self.parameters.join(","));
        
        TransactionBuilder::new()
            .with_type(TransactionType::SmartContract)
            .with_sender(sender)
            .with_data(data)
            .build()
    }
}

/// Transaction pool for managing pending transactions
#[derive(Debug, Clone)]
pub struct TransactionPool {
    transactions: Vec<Transaction>,
    max_size: usize,
}

impl TransactionPool {
    /// Create new transaction pool
    pub fn new(max_size: usize) -> Self {
        Self {
            transactions: Vec::new(),
            max_size,
        }
    }
    
    /// Add transaction to pool
    pub fn add_transaction(&mut self, tx: Transaction) -> BlockchainResult<()> {
        if self.transactions.len() >= self.max_size {
            return Err(BlockchainError::InvalidTransaction {
                reason: "Transaction pool is full".to_string(),
            });
        }
        
        // Verify transaction before adding
        if !tx.verify()? {
            return Err(BlockchainError::InvalidTransaction {
                reason: "Transaction verification failed".to_string(),
            });
        }
        
        // Check for duplicate
        if self.transactions.iter().any(|t| t.tx_id == tx.tx_id) {
            return Err(BlockchainError::InvalidTransaction {
                reason: "Transaction already exists in pool".to_string(),
            });
        }
        
        self.transactions.push(tx);
        Ok(())
    }
    
    /// Get transactions for block creation
    pub fn get_transactions(&self, limit: usize) -> Vec<Transaction> {
        self.transactions.iter().take(limit).cloned().collect()
    }
    
    /// Remove transactions from pool
    pub fn remove_transactions(&mut self, tx_ids: &[String]) {
        self.transactions.retain(|tx| !tx_ids.contains(&tx.tx_id));
    }
    
    /// Get pool size
    pub fn size(&self) -> usize {
        self.transactions.len()
    }
    
    /// Clear pool
    pub fn clear(&mut self) {
        self.transactions.clear();
    }
    
    /// Get transactions by type
    pub fn get_transactions_by_type(&self, tx_type: &TransactionType) -> Vec<Transaction> {
        self.transactions.iter()
            .filter(|tx| std::mem::discriminant(&tx.tx_type) == std::mem::discriminant(tx_type))
            .cloned()
            .collect()
    }
    
    /// Get transactions by sender
    pub fn get_transactions_by_sender(&self, sender: &str) -> Vec<Transaction> {
        self.transactions.iter()
            .filter(|tx| tx.sender == sender)
            .cloned()
            .collect()
    }
}

impl Default for TransactionPool {
    fn default() -> Self {
        Self::new(10000)
    }
}

/// Transaction validator for checking transaction validity
#[derive(Debug, Clone)]
pub struct TransactionValidator {
    min_fee: u64,
    max_data_size: usize,
}

impl TransactionValidator {
    /// Create new transaction validator
    pub fn new(min_fee: u64, max_data_size: usize) -> Self {
        Self {
            min_fee,
            max_data_size,
        }
    }
    
    /// Validate transaction
    pub fn validate(&self, tx: &Transaction) -> BlockchainResult<()> {
        // Check transaction size
        if tx.data.content.len() > self.max_data_size {
            return Err(BlockchainError::InvalidTransaction {
                reason: "Transaction data too large".to_string(),
            });
        }
        
        // Verify transaction hash
        if !tx.verify()? {
            return Err(BlockchainError::InvalidTransaction {
                reason: "Transaction hash invalid".to_string(),
            });
        }
        
        // Check sender address format (simplified)
        if tx.sender.len() != 66 { // Assuming 64 chars + 2 char prefix
            return Err(BlockchainError::InvalidTransaction {
                reason: "Invalid sender address format".to_string(),
            });
        }
        
        // Validate recipient if present
        if let Some(ref recipient) = tx.recipient {
            if recipient.len() != 66 {
                return Err(BlockchainError::InvalidTransaction {
                    reason: "Invalid recipient address format".to_string(),
                });
            }
        }
        
        // Check timestamp (not too far in future)
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();
        
        if tx.timestamp > current_time + 3600 { // 1 hour tolerance
            return Err(BlockchainError::InvalidTransaction {
                reason: "Transaction timestamp too far in future".to_string(),
            });
        }
        
        Ok(())
    }
}

impl Default for TransactionValidator {
    fn default() -> Self {
        Self::new(1000, 1024 * 1024) // 1000 min fee, 1MB max data
    }
}
