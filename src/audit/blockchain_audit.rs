//! Blockchain-based audit trail for NEX2426
//! 
//! Provides immutable, tamper-evident logging using blockchain technology

// Import blockchain components directly
use crate::blockchain::block::{Block, Transaction, TransactionType, EncryptedData};
use crate::blockchain::{Blockchain, BlockchainConfig};
use crate::blockchain::storage::MemoryStorage;
use crate::blockchain::consensus::ProofOfWork;
use crate::blockchain::crypto::BlockchainCrypto;
use crate::blockchain::transaction::TransactionBuilder;
use crate::blockchain::BlockchainError;
use crate::audit::trail::{ComplianceChecker, ComplianceStandard};
use crate::kernel::NexKernel;
use serde::{Deserialize, Serialize};

// Type alias for convenience
type BlockchainResult<T> = Result<T, BlockchainError>;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Blockchain-based audit logger
#[derive(Clone)]
pub struct BlockchainAuditLogger {
    /// Blockchain instance
    blockchain: Arc<Blockchain>,
    /// Logger configuration
    config: AuditConfig,
    /// Crypto utilities
    crypto: BlockchainCrypto,
    /// Current session
    session: Arc<RwLock<AuditSession>>,
}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Organization name
    pub organization: String,
    /// Audit retention period in days
    pub retention_days: u32,
    /// Enable encryption
    pub encrypt_entries: bool,
    /// Enable compliance checking
    pub enable_compliance: bool,
    /// Required compliance standards
    pub required_standards: Vec<ComplianceStandard>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            organization: "NEX2426".to_string(),
            retention_days: 2555, // 7 years
            encrypt_entries: true,
            enable_compliance: true,
            required_standards: vec![
                ComplianceStandard::GDPR,
                ComplianceStandard::HIPAA,
                ComplianceStandard::SOX,
            ],
        }
    }
}

/// Audit session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSession {
    /// Session ID
    pub session_id: String,
    /// Session start time
    pub start_time: u64,
    /// Auditor address
    pub auditor: String,
    /// Session metadata
    pub metadata: HashMap<String, String>,
}

impl AuditSession {
    /// Create new audit session
    pub fn new(auditor: String) -> Self {
        let kernel = NexKernel::new(1);
        let session_id_result = kernel.hash_bytes(
            format!("{}{}", auditor, SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs()).as_bytes(),
            "audit-session"
        );
        let session_id = crate::integrity::merkle::hex_util::encode(&session_id_result);

        Self {
            session_id,
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            auditor,
            metadata: HashMap::new(),
        }
    }
}

/// Audit entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Entry ID
    pub entry_id: String,
    /// Session ID
    pub session_id: String,
    /// Timestamp
    pub timestamp: u64,
    /// Event type
    pub event_type: String,
    /// User/actor
    pub user: String,
    /// Action performed
    pub action: String,
    /// Resource affected
    pub resource: Option<String>,
    /// Outcome (success/failure)
    pub outcome: String,
    /// Additional details
    pub details: HashMap<String, String>,
    /// Compliance tags
    pub compliance_tags: Vec<String>,
    /// Previous entry hash (for chaining)
    pub prev_hash: Option<String>,
}

impl AuditEntry {
    /// Create new audit entry
    pub fn new(
        session_id: String,
        event_type: String,
        user: String,
        action: String,
        resource: Option<String>,
        outcome: String,
        details: HashMap<String, String>,
        prev_hash: Option<String>,
    ) -> BlockchainResult<Self> {
        let kernel = NexKernel::new(1);
        
        // Generate entry ID
        let entry_data = format!(
            "{}{}{}{}{}{}",
            session_id,
            event_type,
            user,
            action,
            resource.as_deref().unwrap_or(""),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs()
        );
        
        let entry_id_result = kernel.hash_bytes(entry_data.as_bytes(), "audit-entry");
        let entry_id = crate::integrity::merkle::hex_util::encode(&entry_id_result);

        Ok(Self {
            entry_id,
            session_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            event_type,
            user,
            action,
            resource,
            outcome,
            details,
            compliance_tags: Vec::new(),
            prev_hash,
        })
    }

    /// Convert to blockchain transaction
    pub fn to_transaction(&self, sender: String, encrypt: bool) -> BlockchainResult<Transaction> {
        let crypto = BlockchainCrypto::new();
        
        // Serialize entry data
        let entry_data = serde_json::to_string(self).map_err(|e| BlockchainError::SerializationError(e.to_string()))?;
        
        // Encrypt if required
        let content = if encrypt {
            let encryption_key = format!("audit_key_{}", self.session_id);
            crypto.encrypt_data(entry_data.as_bytes(), encryption_key.as_bytes())?
        } else {
            entry_data.as_bytes().to_vec()
        };

        let data = EncryptedData {
            content,
            key_ref: self.session_id.clone(),
            algorithm: if encrypt { "nex2426-audit" } else { "none" }.to_string(),
            iv: None,
        };

        let mut metadata = HashMap::new();
        metadata.insert("entry_id".to_string(), self.entry_id.clone());
        metadata.insert("event_type".to_string(), self.event_type.clone());
        metadata.insert("user".to_string(), self.user.clone());
        metadata.insert("encrypted".to_string(), encrypt.to_string());

        TransactionBuilder::new()
            .with_type(TransactionType::Custom("audit-entry".to_string()))
            .with_sender(sender)
            .with_optional_recipient(Some("audit-system".to_string()))
            .with_data(data)
            .build()
    }

    /// Verify entry integrity
    pub fn verify(&self) -> BlockchainResult<bool> {
        let kernel = NexKernel::new(1);
        
        // Recalculate entry ID
        let entry_data = format!(
            "{}{}{}{}{}{}",
            self.session_id,
            self.event_type,
            self.user,
            self.action,
            self.resource.as_deref().unwrap_or(""),
            self.timestamp
        );
        
        let entry_id_result = kernel.hash_bytes(entry_data.as_bytes(), "audit-entry");
        let calculated_id = crate::integrity::merkle::hex_util::encode(&entry_id_result);
        
        Ok(calculated_id == self.entry_id)
    }
}

impl BlockchainAuditLogger {
    /// Create new blockchain audit logger
    pub fn new(config: AuditConfig) -> BlockchainResult<Self> {
        // Create blockchain configuration
        let bc_config = BlockchainConfig {
            chain_id: format!("{}-audit", config.organization),
            difficulty: 2, // Lower difficulty for audit chain
            max_block_size: 512 * 1024, // 512KB blocks
            block_time_target: 300, // 5 minutes
            max_tx_per_block: 500,
            quantum_resistant: true,
        };

        // Create blockchain components
        let storage = Box::new(MemoryStorage::new());
        let consensus = Box::new(ProofOfWork::new(bc_config.difficulty, bc_config.block_time_target));
        let blockchain = Arc::new(Blockchain::new(bc_config, storage, consensus)?);

        Ok(Self {
            blockchain,
            config,
            crypto: BlockchainCrypto::new(),
            session: Arc::new(RwLock::new(AuditSession::new("system".to_string()))),
        })
    }

    /// Start new audit session
    pub fn start_session(&self, auditor: String, metadata: HashMap<String, String>) -> BlockchainResult<String> {
        let mut session = self.session.write()
            .map_err(|_| BlockchainError::ChainValidationFailed {
                reason: "Failed to acquire session lock".to_string(),
            })?;

        *session = AuditSession::new(auditor);
        session.metadata = metadata;

        // Log session start
        self.log_event(
            "SESSION_START".to_string(),
            session.auditor.clone(),
            "START_AUDIT_SESSION".to_string(),
            Some("audit_system".to_string()),
            "SUCCESS".to_string(),
            session.metadata.clone(),
        )?;

        Ok(session.session_id.clone())
    }

    /// Log audit event
    pub fn log_event(
        &self,
        event_type: String,
        user: String,
        action: String,
        resource: Option<String>,
        outcome: String,
        details: HashMap<String, String>,
    ) -> BlockchainResult<()> {
        let session = self.session.read()
            .map_err(|_| BlockchainError::ChainValidationFailed {
                reason: "Failed to acquire session lock".to_string(),
            })?;

        // Get previous entry hash from blockchain
        let prev_hash = self.get_last_entry_hash()?;

        // Create audit entry
        let entry = AuditEntry::new(
            session.session_id.clone(),
            event_type,
            user,
            action,
            resource,
            outcome,
            details,
            prev_hash,
        )?;

        // Convert to transaction
        let tx = entry.to_transaction("audit-logger".to_string(), self.config.encrypt_entries)?;

        // Add to blockchain
        self.blockchain.add_transaction(tx)?;

        Ok(())
    }

    /// Get last entry hash from blockchain
    fn get_last_entry_hash(&self) -> BlockchainResult<Option<String>> {
        if let Some(last_block) = self.blockchain.get_block(self.blockchain.get_height()?)? {
            if let Some(last_tx) = last_block.transactions.last() {
                return Ok(Some(last_tx.tx_id.clone()));
            }
        }
        Ok(None)
    }

    /// Create audit block from pending transactions
    pub fn create_audit_block(&self) -> BlockchainResult<Block> {
        self.blockchain.create_block()
    }

    /// Mine and add audit block
    pub fn mine_audit_block(&self) -> BlockchainResult<()> {
        let block = self.create_audit_block()?;
        self.blockchain.add_block(block)?;
        Ok(())
    }

    /// Verify audit trail integrity
    pub fn verify_audit_trail(&self) -> BlockchainResult<bool> {
        self.blockchain.validate_chain()
    }

    /// Get audit entries by session
    pub fn get_session_entries(&self, session_id: &str) -> BlockchainResult<Vec<AuditEntry>> {
        let mut entries = Vec::new();
        let current_height = self.blockchain.get_height()?;

        for height in 0..=current_height {
            if let Some(block) = self.blockchain.get_block(height)? {
                for tx in &block.transactions {
                    if let TransactionType::Custom(ref tx_type) = tx.tx_type {
                        if tx_type == "audit-entry" {
                            // Decrypt and deserialize entry
                            let entry_data = if self.config.encrypt_entries {
                                self.crypto.decrypt_data(&tx.data.content, session_id.as_bytes())?
                            } else {
                                tx.data.content.clone()
                            };

                            if let Ok(entry) = serde_json::from_slice::<AuditEntry>(&entry_data) {
                                if entry.session_id == session_id {
                                    entries.push(entry);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(entries)
    }

    /// Get audit entries by user
    pub fn get_user_entries(&self, user: &str) -> BlockchainResult<Vec<AuditEntry>> {
        let mut entries = Vec::new();
        let current_height = self.blockchain.get_height()?;

        for height in 0..=current_height {
            if let Some(block) = self.blockchain.get_block(height)? {
                for tx in &block.transactions {
                    if let TransactionType::Custom(ref tx_type) = tx.tx_type {
                        if tx_type == "audit-entry" {
                            // Decrypt and deserialize entry
                            let session = self.session.read()
                                .map_err(|_| BlockchainError::ChainValidationFailed {
                                    reason: "Failed to acquire session lock".to_string(),
                                })?;

                            let entry_data = if self.config.encrypt_entries {
                                self.crypto.decrypt_data(&tx.data.content, session.session_id.as_bytes())?
                            } else {
                                tx.data.content.clone()
                            };

                            if let Ok(entry) = serde_json::from_slice::<AuditEntry>(&entry_data) {
                                if entry.user == user {
                                    entries.push(entry);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(entries)
    }

    /// Get audit statistics
    pub fn get_audit_stats(&self) -> BlockchainResult<AuditStats> {
        let stats = self.blockchain.get_stats()?;
        let session = self.session.read()
            .map_err(|_| BlockchainError::ChainValidationFailed {
                reason: "Failed to acquire session lock".to_string(),
            })?;

        Ok(AuditStats {
            total_entries: stats.total_transactions,
            total_blocks: stats.total_blocks,
            current_session: session.session_id.clone(),
            session_start_time: session.start_time,
            auditor: session.auditor.clone(),
            chain_height: stats.total_blocks - 1,
            last_block_time: stats.last_block_time,
        })
    }

    /// Export audit trail
    pub fn export_audit_trail(&self, format: ExportFormat) -> BlockchainResult<Vec<u8>> {
        let current_height = self.blockchain.get_height()?;
        let mut all_entries = Vec::new();

        for height in 0..=current_height {
            if let Some(block) = self.blockchain.get_block(height)? {
                for tx in &block.transactions {
                    if let TransactionType::Custom(ref tx_type) = tx.tx_type {
                        if tx_type == "audit-entry" {
                            // Decrypt and deserialize entry
                            let session = self.session.read()
                                .map_err(|_| BlockchainError::ChainValidationFailed {
                                    reason: "Failed to acquire session lock".to_string(),
                                })?;

                            let entry_data = if self.config.encrypt_entries {
                                self.crypto.decrypt_data(&tx.data.content, session.session_id.as_bytes())?
                            } else {
                                tx.data.content.clone()
                            };

                            if let Ok(entry) = serde_json::from_slice::<AuditEntry>(&entry_data) {
                                all_entries.push(entry);
                            }
                        }
                    }
                }
            }
        }

        match format {
            ExportFormat::Json => serde_json::to_vec_pretty(&all_entries)
                .map_err(|e| BlockchainError::SerializationError(e.to_string())),
            ExportFormat::Csv => self.export_to_csv(&all_entries),
        }
    }

    /// Export entries to CSV format
    fn export_to_csv(&self, entries: &[AuditEntry]) -> BlockchainResult<Vec<u8>> {
        let mut csv_data = Vec::new();
        
        // CSV header
        csv_data.extend_from_slice(b"Entry ID,Session ID,Timestamp,Event Type,User,Action,Resource,Outcome\n");
        
        // CSV rows
        for entry in entries {
            let row = format!(
                "{},{},{},{},{},{},{},{}\n",
                entry.entry_id,
                entry.session_id,
                entry.timestamp,
                entry.event_type,
                entry.user,
                entry.action,
                entry.resource.as_deref().unwrap_or(""),
                entry.outcome
            );
            csv_data.extend_from_slice(row.as_bytes());
        }
        
        Ok(csv_data)
    }
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_entries: u64,
    pub total_blocks: u64,
    pub current_session: String,
    pub session_start_time: u64,
    pub auditor: String,
    pub chain_height: u64,
    pub last_block_time: u64,
}

/// Export format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
}

/// Blockchain compliance checker
#[derive(Clone)]
pub struct BlockchainComplianceChecker {
    blockchain: Arc<Blockchain>,
    standards: Vec<ComplianceStandard>,
}

impl BlockchainComplianceChecker {
    /// Create new blockchain compliance checker
    pub fn new(blockchain: Arc<Blockchain>, standards: Vec<ComplianceStandard>) -> Self {
        Self { blockchain, standards }
    }

    /// Check compliance for audit trail
    pub fn check_compliance(&self) -> BlockchainResult<ComplianceReport> {
        let mut report = ComplianceReport {
            chain_id: self.blockchain.get_stats()?.chain_id,
            total_entries: 0,
            compliant_entries: 0,
            violations: Vec::new(),
            overall_compliant: true,
            check_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
        };

        let current_height = self.blockchain.get_height()?;

        for height in 0..=current_height {
            if let Some(block) = self.blockchain.get_block(height)? {
                for tx in &block.transactions {
                    report.total_entries += 1;

                    // Check transaction compliance
                    let is_compliant = self.check_transaction_compliance(tx)?;
                    if is_compliant {
                        report.compliant_entries += 1;
                    } else {
                        report.overall_compliant = false;
                        report.violations.push(ComplianceViolation {
                            transaction_id: tx.tx_id.clone(),
                            block_height: height,
                            violation_type: "Transaction compliance check failed".to_string(),
                            description: "Transaction does not meet required standards".to_string(),
                        });
                    }
                }
            }
        }

        Ok(report)
    }

    /// Check individual transaction compliance
    fn check_transaction_compliance(&self, tx: &Transaction) -> BlockchainResult<bool> {
        // Verify transaction integrity
        if !tx.verify()? {
            return Ok(false);
        }

        // Check timestamp (not too old)
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();

        if current_time.saturating_sub(tx.timestamp) > 86400 * 365 { // 1 year
            return Ok(false);
        }

        // Check data size limits
        if tx.data.content.len() > 1024 * 1024 { // 1MB limit
            return Ok(false);
        }

        Ok(true)
    }
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub chain_id: String,
    pub total_entries: u64,
    pub compliant_entries: u64,
    pub violations: Vec<ComplianceViolation>,
    pub overall_compliant: bool,
    pub check_timestamp: u64,
}

/// Compliance violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub transaction_id: String,
    pub block_height: u64,
    pub violation_type: String,
    pub description: String,
}
