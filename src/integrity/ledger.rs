use crate::standards::signatures::{NexSigner, Signature, PublicKey, PrivateKey};
use crate::kernel::NexKernel;
use crate::integrity::merkle::MerkleTree;

/// Nex Ledger: A high-integrity, signed blockchain structure.
/// Uses Post-Quantum Signatures (NDSS) for block authorization.
/// Uses Merkle Trees for transaction batching.

#[derive(Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub prev_hash: String,
    pub transactions_root: String, // Merkle Root of data
    pub signature: Option<Signature>, // Signed by Validator
    pub hash: String,
}

pub struct NexLedger {
    pub chain: Vec<Block>,
    signer: NexSigner,
}

impl NexLedger {
    pub fn new() -> Self {
        // Genesis Block
        let genesis  = Block {
            index: 0,
            timestamp: 0,
            prev_hash: "0000000000000000".to_string(),
            transactions_root: "GENESIS".to_string(),
            signature: None,
            hash: "GENESIS_HASH".to_string(), 
        };
        
        Self {
            chain: vec![genesis],
            signer: NexSigner::new(),
        }
    }
    
    pub fn create_block(&self, transactions: Vec<&[u8]>, prev_hash: String, index: u64, validator_key: &PrivateKey) -> Block {
        // 1. Calculate Merkle Root
        let tree = MerkleTree::new(transactions);
        let root = tree.get_root_hash();
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();

        // 2. Prepare Header for Hashing/Signing
        // Format: index|prev|root|time
        let header = format!("{}{}{}{}", index, prev_hash, root, timestamp);
        
        // 3. Sign the Header
        let sig = self.signer.sign(header.as_bytes(), validator_key);
        
        // 4. Calculate Block Hash (Header + Sig)
        // Ideally block hash includes signature to prevent malleability
        // But the sig signs the "content", so the block ID usually is Hash(Header).
        // Let's include everything.
        let mut full_data = header.as_bytes().to_vec();
        // Append signature data for unique block hash
        for x in &sig.z { full_data.extend_from_slice(&x.to_le_bytes()); }
        
        let kernel = NexKernel::new(1);
        let block_hash_bytes = kernel.hash_bytes(&full_data, "Block-Hash");
        let hash_hex = hex_encode(&block_hash_bytes);
        
        Block {
            index,
            timestamp,
            prev_hash,
            transactions_root: root,
            signature: Some(sig),
            hash: hash_hex,
        }
    }
    
    pub fn add_block(&mut self, block: Block, validator_pub: &PublicKey) -> bool {
        // 1. Verify Linkage
        let last = self.chain.last().expect("Cannot add block to empty chain");
        if block.prev_hash != last.hash {
            println!("Linkage Error: {} != {}", block.prev_hash, last.hash);
            return false;
        }
        if block.index != last.index + 1 {
            return false;
        }
        
        // 2. Verify Signature
        if let Some(sig) = &block.signature {
            let header = format!("{}{}{}{}", block.index, block.prev_hash, block.transactions_root, block.timestamp);
            if !self.signer.verify(header.as_bytes(), validator_pub, sig) {
                println!("Signature Verification Failed for Block {}", block.index);
                return false;
            }
        } else {
             return false; // Must be signed
        }
        
        self.chain.push(block);
        true
    }
}

fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).collect()
}
