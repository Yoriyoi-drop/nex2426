//! Blockchain API endpoints for NEX2426
//! 
//! Provides REST API for blockchain operations and management

use crate::blockchain::{Blockchain, BlockchainConfig, Transaction};
use crate::blockchain::consensus::{ProofOfWork, QuantumProofOfWork, ConsensusEngine};
use crate::blockchain::storage::FileStorage;
use crate::blockchain::transaction::{TransactionBuilder, TransactionPool, EncryptionTransaction};
use crate::blockchain::block::EncryptedData;
use crate::audit::blockchain_audit::{BlockchainAuditLogger, AuditConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// Blockchain API server
pub struct BlockchainApiServer {
    /// Blockchain instance
    blockchain: Arc<Blockchain>,
    /// Audit logger
    audit_logger: Arc<BlockchainAuditLogger>,
    /// Transaction pool
    tx_pool: Arc<Mutex<TransactionPool>>,
    /// API statistics
    stats: Arc<RwLock<ApiStats>>,
}

/// API statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub blocks_created: u64,
    pub transactions_processed: u64,
    pub uptime: u64,
    pub start_time: u64,
}

impl Default for ApiStats {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();
        
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            blocks_created: 0,
            transactions_processed: 0,
            uptime: 0,
            start_time: now,
        }
    }
}

impl BlockchainApiServer {
    /// Create new blockchain API server
    pub async fn new(config: BlockchainConfig, audit_config: AuditConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Create storage
        let storage_path = format!("./blockchain_data/{}", config.chain_id);
        let storage = Box::new(FileStorage::new(storage_path)?);

        // Create consensus engine
        let consensus: Box<dyn ConsensusEngine> = if config.quantum_resistant {
            Box::new(QuantumProofOfWork::new(config.difficulty, config.block_time_target, 1024))
        } else {
            Box::new(ProofOfWork::new(config.difficulty, config.block_time_target))
        };

        // Create blockchain
        let blockchain = Arc::new(Blockchain::new(config, storage, consensus)?);
        
        // Create audit logger
        let audit_logger = Arc::new(BlockchainAuditLogger::new(audit_config)?);
        
        // Create transaction pool
        let tx_pool = Arc::new(Mutex::new(TransactionPool::new(10000)));
        
        // Create API stats
        let stats = Arc::new(RwLock::new(ApiStats::default()));

        Ok(Self {
            blockchain,
            audit_logger,
            tx_pool,
            stats,
        })
    }

    /// Get blockchain information
    pub async fn get_blockchain_info(&self) -> Result<BlockchainInfoResponse, Box<dyn std::error::Error>> {
        self.update_stats(|stats| stats.total_requests += 1);
        
        let bc_stats = self.blockchain.get_stats()?;
        let audit_stats = self.audit_logger.get_audit_stats()?;
        let api_stats = self.stats.read().unwrap().clone();

        let response = BlockchainInfoResponse {
            chain_id: bc_stats.chain_id,
            total_blocks: bc_stats.total_blocks,
            total_transactions: bc_stats.total_transactions,
            difficulty: bc_stats.difficulty,
            hash_rate: bc_stats.hash_rate,
            last_block_time: bc_stats.last_block_time,
            chain_weight: bc_stats.chain_weight,
            audit_stats,
            api_stats,
            mempool_size: self.blockchain.get_mempool_size()?,
        };

        self.update_stats(|stats| stats.successful_requests += 1);
        Ok(response)
    }

    /// Get block by height
    pub async fn get_block(&self, height: u64) -> Result<Option<BlockResponse>, Box<dyn std::error::Error>> {
        self.update_stats(|stats| stats.total_requests += 1);

        if let Some(block) = self.blockchain.get_block(height)? {
            let response = BlockResponse {
                height: block.get_height(),
                hash: block.get_hash().unwrap_or(&"".to_string()).clone(),
                timestamp: block.get_timestamp(),
                prev_hash: block.header.prev_hash.clone(),
                merkle_root: block.header.merkle_root.clone(),
                difficulty: block.header.difficulty,
                nonce: block.header.nonce,
                transaction_count: block.get_tx_count(),
                size: block.size,
                transactions: block.transactions.iter().map(|tx| TransactionResponse::from(tx)).collect(),
            };

            self.update_stats(|stats| stats.successful_requests += 1);
            Ok(Some(response))
        } else {
            self.update_stats(|stats| stats.failed_requests += 1);
            Ok(None)
        }
    }

    /// Get block by hash
    pub async fn get_block_by_hash(&self, hash: &str) -> Result<Option<BlockResponse>, Box<dyn std::error::Error>> {
        self.update_stats(|stats| stats.total_requests += 1);

        if let Some(block) = self.blockchain.get_block_by_hash(hash)? {
            let response = BlockResponse {
                height: block.get_height(),
                hash: block.get_hash().unwrap_or(&"".to_string()).clone(),
                timestamp: block.get_timestamp(),
                prev_hash: block.header.prev_hash.clone(),
                merkle_root: block.header.merkle_root.clone(),
                difficulty: block.header.difficulty,
                nonce: block.header.nonce,
                transaction_count: block.get_tx_count(),
                size: block.size,
                transactions: block.transactions.iter().map(|tx| TransactionResponse::from(tx)).collect(),
            };

            self.update_stats(|stats| stats.successful_requests += 1);
            Ok(Some(response))
        } else {
            self.update_stats(|stats| stats.failed_requests += 1);
            Ok(None)
        }
    }

    /// Create and submit transaction
    pub async fn submit_transaction(&self, request: SubmitTransactionRequest) -> Result<TransactionResponse, Box<dyn std::error::Error>> {
        self.update_stats(|stats| stats.total_requests += 1);

        // Create transaction
        let tx = match request.tx_type.as_str() {
            "encryption" => {
                let encryption_tx = EncryptionTransaction::new(
                    &request.data,
                    &request.key,
                    request.algorithm.unwrap_or_else(|| "nex2426".to_string()),
                    request.metadata.unwrap_or_default(),
                )?;
                encryption_tx.to_transaction(request.sender)?
            }
            "custom" => {
                let data = EncryptedData {
                    content: request.data,
                    key_ref: request.key_ref.unwrap_or_else(|| "custom".to_string()),
                    algorithm: request.algorithm.unwrap_or_else(|| "custom".to_string()),
                    iv: None,
                };
                TransactionBuilder::new()
                    .with_type(crate::blockchain::TransactionType::Custom(request.custom_type.unwrap_or_else(|| "custom".to_string())))
                    .with_sender(request.sender)
                    .with_optional_recipient(request.recipient)
                    .with_data(data)
                    .build()?
            }
            _ => {
                return Err("Unsupported transaction type".into());
            }
        };

        // Add to blockchain
        self.blockchain.add_transaction(tx.clone())?;
        
        // Add to pool
        {
            let mut pool = self.tx_pool.lock().await;
            pool.add_transaction(tx.clone())?;
        }

        // Log transaction
        self.audit_logger.log_event(
            "TRANSACTION_SUBMIT".to_string(),
            tx.sender.clone(),
            format!("SUBMIT_{}", request.tx_type),
            Some(tx.tx_id.clone()),
            "SUCCESS".to_string(),
            HashMap::from([
                ("tx_id".to_string(), tx.tx_id.clone()),
                ("tx_type".to_string(), request.tx_type),
            ]),
        )?;

        self.update_stats(|stats| {
            stats.successful_requests += 1;
            stats.transactions_processed += 1;
        });

        Ok(TransactionResponse::from(&tx))
    }

    /// Mine new block
    pub async fn mine_block(&self) -> Result<BlockResponse, Box<dyn std::error::Error>> {
        self.update_stats(|stats| stats.total_requests += 1);

        // Create block from mempool
        let block = self.blockchain.create_block()?;
        
        // Mine block
        self.blockchain.add_block(block)?;
        let mined_block = self.blockchain.get_block(self.blockchain.get_height()?)?.unwrap();

        // Update pool
        {
            let mut pool = self.tx_pool.lock().await;
            let tx_ids: Vec<String> = mined_block.transactions.iter()
                .map(|tx| tx.tx_id.clone())
                .collect();
            pool.remove_transactions(&tx_ids);
        }

        // Log mining
        self.audit_logger.log_event(
            "BLOCK_MINED".to_string(),
            "miner".to_string(),
            "MINE_BLOCK".to_string(),
            Some(mined_block.get_hash().unwrap_or(&"".to_string()).clone()),
            "SUCCESS".to_string(),
            HashMap::from([
                ("block_height".to_string(), mined_block.get_height().to_string()),
                ("tx_count".to_string(), mined_block.get_tx_count().to_string()),
            ]),
        )?;

        let response = BlockResponse {
            height: mined_block.get_height(),
            hash: mined_block.get_hash().unwrap_or(&"".to_string()).clone(),
            timestamp: mined_block.get_timestamp(),
            prev_hash: mined_block.header.prev_hash.clone(),
            merkle_root: mined_block.header.merkle_root.clone(),
            difficulty: mined_block.header.difficulty,
            nonce: mined_block.header.nonce,
            transaction_count: mined_block.get_tx_count(),
            size: mined_block.size,
            transactions: mined_block.transactions.iter().map(|tx| TransactionResponse::from(tx)).collect(),
        };

        self.update_stats(|stats| {
            stats.successful_requests += 1;
            stats.blocks_created += 1;
        });

        Ok(response)
    }

    /// Validate blockchain
    pub async fn validate_blockchain(&self) -> Result<ValidationResponse, Box<dyn std::error::Error>> {
        self.update_stats(|stats| stats.total_requests += 1);

        let is_valid = self.blockchain.validate_chain()?;
        let audit_valid = self.audit_logger.verify_audit_trail()?;

        let response = ValidationResponse {
            blockchain_valid: is_valid,
            audit_trail_valid: audit_valid,
            overall_valid: is_valid && audit_valid,
            validation_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
        };

        self.update_stats(|stats| stats.successful_requests += 1);
        Ok(response)
    }

    /// Get mempool transactions
    pub async fn get_mempool(&self, limit: Option<usize>) -> Result<Vec<TransactionResponse>, Box<dyn std::error::Error>> {
        self.update_stats(|stats| stats.total_requests += 1);

        let pool = self.tx_pool.lock().await;
        let transactions = pool.get_transactions(limit.unwrap_or(100));
        let responses: Vec<TransactionResponse> = transactions.iter().map(TransactionResponse::from).collect();

        self.update_stats(|stats| stats.successful_requests += 1);
        Ok(responses)
    }

    /// Get audit trail
    pub async fn get_audit_trail(&self, session_id: Option<String>, format: Option<String>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.update_stats(|stats| stats.total_requests += 1);

        let export_format = match format.as_deref() {
            Some("csv") => crate::audit::blockchain_audit::ExportFormat::Csv,
            _ => crate::audit::blockchain_audit::ExportFormat::Json,
        };

        let data = if let Some(session_id) = session_id {
            let entries = self.audit_logger.get_session_entries(&session_id)?;
            serde_json::to_vec_pretty(&entries)?
        } else {
            self.audit_logger.export_audit_trail(export_format)?
        };

        self.update_stats(|stats| stats.successful_requests += 1);
        Ok(data)
    }

    /// Update API statistics
    fn update_stats<F>(&self, updater: F)
    where
        F: FnOnce(&mut ApiStats),
    {
        let mut stats = self.stats.write().unwrap();
        updater(&mut stats);
        stats.uptime = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs() - stats.start_time;
    }
}

/// API Response types

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainInfoResponse {
    pub chain_id: String,
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub difficulty: u32,
    pub hash_rate: f64,
    pub last_block_time: u64,
    pub chain_weight: u64,
    pub audit_stats: crate::audit::blockchain_audit::AuditStats,
    pub api_stats: ApiStats,
    pub mempool_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockResponse {
    pub height: u64,
    pub hash: String,
    pub timestamp: u64,
    pub prev_hash: String,
    pub merkle_root: String,
    pub difficulty: u32,
    pub nonce: u64,
    pub transaction_count: usize,
    pub size: usize,
    pub transactions: Vec<TransactionResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub tx_id: String,
    pub tx_type: String,
    pub sender: String,
    pub recipient: Option<String>,
    pub timestamp: u64,
    pub data_hash: String,
    pub algorithm: String,
    pub signature: Option<String>,
    pub quantum_signature: Option<String>,
}

impl From<&Transaction> for TransactionResponse {
    fn from(tx: &Transaction) -> Self {
        Self {
            tx_id: tx.tx_id.clone(),
            tx_type: format!("{:?}", tx.tx_type),
            sender: tx.sender.clone(),
            recipient: tx.recipient.clone(),
            timestamp: tx.timestamp,
            data_hash: crate::integrity::merkle::hex_util::encode(&crate::kernel::NexKernel::new(1).hash_bytes(&tx.data.content, "tx-data-hash")),
            algorithm: tx.data.algorithm.clone(),
            signature: tx.signature.clone(),
            quantum_signature: tx.quantum_signature.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitTransactionRequest {
    pub tx_type: String,
    pub sender: String,
    pub recipient: Option<String>,
    pub data: Vec<u8>,
    pub key: Vec<u8>,
    pub key_ref: Option<String>,
    pub algorithm: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub custom_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub blockchain_valid: bool,
    pub audit_trail_valid: bool,
    pub overall_valid: bool,
    pub validation_time: u64,
}

/// API error types
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub code: u16,
    pub timestamp: u64,
}

impl ApiError {
    pub fn new(error: String, code: u16) -> Self {
        Self {
            error,
            code,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
        }
    }
}

/// API routes helper
pub struct ApiRoutes;

impl ApiRoutes {
    /// Create API routes for the blockchain server
    pub fn create_routes() -> Vec<(&'static str, &'static str)> {
        vec![
            ("GET", "/blockchain/info"),
            ("GET", "/blockchain/block/{height}"),
            ("GET", "/blockchain/block/hash/{hash}"),
            ("POST", "/blockchain/transaction"),
            ("POST", "/blockchain/mine"),
            ("GET", "/blockchain/validate"),
            ("GET", "/blockchain/mempool"),
            ("GET", "/audit/trail"),
            ("GET", "/audit/trail/{session_id}"),
        ]
    }
}
