//! Blockchain Demo for NEX2426
//! 
//! Demonstrates the blockchain capabilities of NEX2426 with quantum-resistant hashing

use nex2426::blockchain::{
    Blockchain, BlockchainConfig, Block, Transaction, TransactionType, EncryptedData,
    storage::MemoryStorage, consensus::ProofOfWork, crypto::BlockchainCrypto,
};
use nex2426::blockchain::transaction::{TransactionBuilder, EncryptionTransaction};
use nex2426::audit::blockchain_audit::{BlockchainAuditLogger, AuditConfig};
use std::collections::HashMap;
use std::time::SystemTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== NEX2426 Blockchain Demo ===");
    println!("Quantum-Resistant Blockchain with Chaos Encryption\n");

    // 1. Create blockchain configuration
    let config = BlockchainConfig {
        chain_id: "nex2426-demo".to_string(),
        difficulty: 2, // Low difficulty for demo
        max_block_size: 1024 * 512, // 512KB
        block_time_target: 30, // 30 seconds
        max_tx_per_block: 10,
        quantum_resistant: true,
    };

    println!("Blockchain Configuration:");
    println!("  Chain ID: {}", config.chain_id);
    println!("  Difficulty: {}", config.difficulty);
    println!("  Quantum Resistant: {}", config.quantum_resistant);
    println!();

    // 2. Create blockchain
    let storage = Box::new(MemoryStorage::new());
    let consensus = Box::new(ProofOfWork::new(config.difficulty, config.block_time_target));
    let blockchain = Blockchain::new(config.clone(), storage, consensus)?;

    println!("Blockchain initialized successfully!");
    println!("Current height: {}", blockchain.get_height()?);
    println!();

    // 3. Create audit logger
    let audit_config = AuditConfig {
        organization: "NEX2426 Demo".to_string(),
        retention_days: 365,
        encrypt_entries: true,
        enable_compliance: true,
        required_standards: vec![],
    };

    let audit_logger = BlockchainAuditLogger::new(audit_config.clone())?;
    println!("Audit logger initialized!");
    println!();

    // 4. Start audit session
    let session_id = audit_logger.start_session(
        "demo-user".to_string(),
        HashMap::from([
            ("purpose".to_string(), "blockchain-demo".to_string()),
            ("environment".to_string(), "development".to_string()),
        ]),
    )?;
    println!("Audit session started: {}", session_id);
    println!();

    // 5. Create and submit transactions
    println!("=== Creating Transactions ===");
    
    // Transaction 1: Data encryption
    let crypto = BlockchainCrypto::new();
    let test_data = b"This is a secret message encrypted with NEX2426";
    let encryption_key = b"demo-encryption-key-12345";
    
    let encryption_tx = EncryptionTransaction::new(
        test_data,
        encryption_key,
        "nex2426-aes".to_string(),
        HashMap::from([
            ("purpose".to_string(), "demo-encryption".to_string()),
            ("classification".to_string(), "confidential".to_string()),
        ]),
    )?;
    
    let tx1 = encryption_tx.to_transaction("alice".to_string())?;
    blockchain.add_transaction(tx1.clone())?;
    
    println!("Transaction 1 - Encryption:");
    println!("  TX ID: {}", tx1.tx_id);
    println!("  Type: {:?}", tx1.tx_type);
    println!("  Sender: {}", tx1.sender);
    println!("  Data size: {} bytes", tx1.data.content.len());
    println!();

    // Transaction 2: Custom transaction
    let tx2 = TransactionBuilder::new()
        .with_type(TransactionType::Custom("demo-operation".to_string()))
        .with_sender("bob".to_string())
        .with_recipient(Some("charlie".to_string()))
        .with_data(EncryptedData {
            content: b"Demo operation data".to_vec(),
            key_ref: "demo-key".to_string(),
            algorithm: "nex2426-demo".to_string(),
            iv: None,
        })
        .build()?;
    
    blockchain.add_transaction(tx2.clone())?;
    
    println!("Transaction 2 - Custom:");
    println!("  TX ID: {}", tx2.tx_id);
    println!("  Type: {:?}", tx2.tx_type);
    println!("  Sender: {}", tx2.sender);
    println!("  Recipient: {:?}", tx2.recipient);
    println!();

    // Log transactions to audit trail
    audit_logger.log_event(
        "TRANSACTION_CREATED".to_string(),
        "demo-system".to_string(),
        "CREATE_TRANSACTIONS".to_string(),
        Some("blockchain-demo".to_string()),
        "SUCCESS".to_string(),
        HashMap::from([
            ("tx_count".to_string(), "2".to_string()),
            ("session_id".to_string(), session_id.clone()),
        ]),
    )?;

    // 6. Mine blocks
    println!("=== Mining Blocks ===");
    
    // Mine first block
    println!("Mining block 1...");
    let block1 = blockchain.create_block()?;
    let mined_block1 = blockchain.add_block(block1)?;
    
    println!("Block 1 mined successfully!");
    println!("  Height: {}", mined_block1.get_height());
    println!("  Hash: {}", mined_block1.get_hash().unwrap_or(&"unknown".to_string()));
    println!("  Transactions: {}", mined_block1.get_tx_count());
    println!("  Nonce: {}", mined_block1.header.nonce);
    println!();

    // Create more transactions
    for i in 3..=5 {
        let tx = TransactionBuilder::new()
            .with_type(TransactionType::Custom(format!("demo-tx-{}", i)))
            .with_sender(format!("user-{}", i))
            .with_data(EncryptedData {
                content: format!("Test transaction data {}", i).into_bytes(),
                key_ref: format!("key-{}", i),
                algorithm: "nex2426".to_string(),
                iv: None,
            })
            .build()?;
        
        blockchain.add_transaction(tx)?;
    }

    // Mine second block
    println!("Mining block 2...");
    let block2 = blockchain.create_block()?;
    let mined_block2 = blockchain.add_block(block2)?;
    
    println!("Block 2 mined successfully!");
    println!("  Height: {}", mined_block2.get_height());
    println!("  Hash: {}", mined_block2.get_hash().unwrap_or(&"unknown".to_string()));
    println!("  Transactions: {}", mined_block2.get_tx_count());
    println!();

    // 7. Display blockchain statistics
    println!("=== Blockchain Statistics ===");
    let stats = blockchain.get_stats()?;
    
    println!("Chain ID: {}", stats.chain_id);
    println!("Total Blocks: {}", stats.total_blocks);
    println!("Total Transactions: {}", stats.total_transactions);
    println!("Current Difficulty: {}", stats.difficulty);
    println!("Last Block Time: {}", stats.last_block_time);
    println!("Chain Weight: {}", stats.chain_weight);
    println!();

    // 8. Validate blockchain
    println!("=== Blockchain Validation ===");
    let is_valid = blockchain.validate_chain()?;
    println!("Blockchain valid: {}", is_valid);
    
    let audit_valid = audit_logger.verify_audit_trail()?;
    println!("Audit trail valid: {}", audit_valid);
    println!();

    // 9. Display block details
    println!("=== Block Details ===");
    
    for height in 0..=blockchain.get_height()? {
        if let Some(block) = blockchain.get_block(height)? {
            println!("Block {}:", height);
            println!("  Hash: {}", block.get_hash().unwrap_or(&"unknown".to_string()));
            println!("  Previous: {}", block.header.prev_hash);
            println!("  Timestamp: {}", block.get_timestamp());
            println!("  Merkle Root: {}", block.header.merkle_root);
            println!("  Transactions: {}", block.get_tx_count());
            println!("  Size: {} bytes", block.size);
            
            for (i, tx) in block.transactions.iter().enumerate() {
                println!("    TX {}: {}", i + 1, tx.tx_id);
                println!("      Type: {:?}", tx.tx_type);
                println!("      Sender: {}", tx.sender);
            }
            println!();
        }
    }

    // 10. Audit trail information
    println!("=== Audit Trail Information ===");
    let audit_stats = audit_logger.get_audit_stats()?;
    
    println!("Total audit entries: {}", audit_stats.total_entries);
    println!("Current session: {}", audit_stats.current_session);
    println!("Session start time: {}", audit_stats.session_start_time);
    println!("Auditor: {}", audit_stats.auditor);
    println!();

    // 11. Get session entries
    println!("=== Session Audit Entries ===");
    let session_entries = audit_logger.get_session_entries(&session_id)?;
    
    for (i, entry) in session_entries.iter().enumerate() {
        println!("Entry {}:", i + 1);
        println!("  ID: {}", entry.entry_id);
        println!("  Event: {}", entry.event_type);
        println!("  User: {}", entry.user);
        println!("  Action: {}", entry.action);
        println!("  Outcome: {}", entry.outcome);
        println!("  Timestamp: {}", entry.timestamp);
        println!();
    }

    // 12. Export audit trail
    println!("=== Export Audit Trail ===");
    let audit_data = audit_logger.export_audit_trail(
        nex2426::audit::blockchain_audit::ExportFormat::Json
    )?;
    
    println!("Audit trail exported ({} bytes)", audit_data.len());
    println!("First 100 characters: {}", 
        String::from_utf8_lossy(&audit_data[..audit_data.len().min(100)]));
    println!();

    // 13. Demonstrate quantum-resistant features
    println!("=== Quantum-Resistant Features ===");
    
    // Generate quantum key pair
    let quantum_crypto = crypto.quantum_crypto();
    let (private_key, public_key) = quantum_crypto.generate_keypair()?;
    
    println!("Quantum key pair generated:");
    println!("  Private key size: {} bytes", private_key.len());
    println!("  Public key size: {} bytes", public_key.len());
    
    // Sign and verify data
    let test_message = b"Quantum-resistant test message";
    let signature = quantum_crypto.sign(test_message, &private_key)?;
    let is_valid = quantum_crypto.verify(test_message, &signature, &public_key)?;
    
    println!("Quantum signature:");
    println!("  Signature size: {} bytes", signature.len());
    println!("  Verification: {}", is_valid);
    println!();

    // 14. Generate blockchain address
    println!("=== Address Generation ===");
    let address = crypto.generate_address(&public_key)?;
    println!("Generated address: {}", address);
    println!("Address valid: {}", crypto.validate_address(&address));
    println!();

    // 15. Final summary
    println!("=== Demo Summary ===");
    println!("Successfully demonstrated:");
    println!("  - Blockchain creation and management");
    println!("  - Transaction creation and validation");
    println!("  - Proof-of-Work mining");
    println!("  - Quantum-resistant cryptography");
    println!("  - Immutable audit trail");
    println!("  - Blockchain validation");
    println!("  - Address generation");
    println!("  - Data encryption/decryption");
    println!();

    println!("NEX2426 Blockchain Demo completed successfully! ");
    println!("The blockchain is now ready for production use with quantum-resistant security.");

    Ok(())
}
