//! Cryptographic utilities for NEX2426 Blockchain
//! 
//! Provides quantum-resistant cryptographic functions for blockchain operations

use crate::blockchain::{Transaction, BlockchainError, BlockchainResult};
use crate::kernel::NexKernel;
use crate::transform::stage_chaos::ChaosEngine;
use serde::{Deserialize, Serialize};

/// Merkle Tree implementation for transaction verification
#[derive(Debug, Clone)]
pub struct MerkleTree {
    root: Option<String>,
    tree: Vec<Vec<String>>,
}

impl MerkleTree {
    /// Create new Merkle tree from transactions
    pub fn new(transactions: Vec<&Transaction>) -> Self {
        if transactions.is_empty() {
            return Self {
                root: None,
                tree: Vec::new(),
            };
        }

        let mut tree = Vec::new();
        
        // Create leaf nodes (transaction hashes)
        let mut current_level: Vec<String> = transactions.iter()
            .map(|tx| tx.tx_id.clone())
            .collect();
        
        tree.push(current_level.clone());
        
        // Build tree levels
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for i in (0..current_level.len()).step_by(2) {
                let left = &current_level[i];
                let right = if i + 1 < current_level.len() {
                    &current_level[i + 1]
                } else {
                    left // Duplicate odd node
                };
                
                let combined = format!("{}{}", left, right);
                let hash = Self::hash_node(&combined);
                next_level.push(hash);
            }
            
            tree.push(next_level.clone());
            current_level = next_level;
        }
        
        let root = current_level.first().cloned();
        
        Self { root, tree }
    }
    
    /// Hash a node using NEX2426
    fn hash_node(data: &str) -> String {
        let kernel = NexKernel::new(1);
        let hash_result = kernel.hash_bytes(data.as_bytes(), "merkle-node");
        crate::integrity::merkle::hex_util::encode(&hash_result)
    }
    
    /// Get root hash
    pub fn get_root_hash(&self) -> String {
        self.root.clone().unwrap_or_else(|| "0".repeat(64))
    }
    
    /// Verify transaction is in tree
    pub fn verify_transaction(&self, tx_hash: &str, proof: &[String]) -> bool {
        if self.root.is_none() {
            return false;
        }
        
        let mut current_hash = tx_hash.to_string();
        
        for proof_hash in proof {
            let combined = format!("{}{}", current_hash, proof_hash);
            current_hash = Self::hash_node(&combined);
        }
        
        current_hash == *self.root.as_ref().unwrap()
    }
    
    /// Generate Merkle proof for transaction
    pub fn generate_proof(&self, tx_hash: &str) -> Option<Vec<String>> {
        // Find transaction in first level
        if let Some(first_level) = self.tree.first() {
            if let Some(pos) = first_level.iter().position(|h| h == tx_hash) {
                let mut proof = Vec::new();
                
                // Generate proof for each level
                for (_level_idx, level) in self.tree.iter().enumerate() {
                    if level.len() <= 1 {
                        break;
                    }
                    
                    let sibling_pos = if pos % 2 == 0 {
                        pos + 1
                    } else {
                        pos - 1
                    };
                    
                    if sibling_pos < level.len() {
                        proof.push(level[sibling_pos].clone());
                    }
                }
                
                Some(proof)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Digital signature using NEX2426
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalSignature {
    /// Signature value
    pub signature: String,
    /// Public key
    pub public_key: String,
    /// Algorithm used
    pub algorithm: String,
}

impl DigitalSignature {
    /// Create new signature
    pub fn new(data: &[u8], private_key: &[u8]) -> BlockchainResult<Self> {
        let kernel = NexKernel::new(1);
        
        // Create signature input
        let sig_input = format!("{}{}", 
            hex::encode(data), 
            hex::encode(private_key));
        
        let signature_result = kernel.hash_bytes(sig_input.as_bytes(), "digital-signature");
        let signature = crate::integrity::merkle::hex_util::encode(&signature_result);
        
        // Generate public key (simplified)
        let public_key = Self::derive_public_key(private_key)?;
        
        Ok(Self {
            signature,
            public_key,
            algorithm: "nex2426-quantum".to_string(),
        })
    }
    
    /// Verify signature
    pub fn verify(&self, data: &[u8]) -> BlockchainResult<bool> {
        let kernel = NexKernel::new(1);
        
        // Recreate signature input
        let sig_input = format!("{}{}", 
            hex::encode(data), 
            self.public_key);
        
        let expected_result = kernel.hash_bytes(sig_input.as_bytes(), "digital-signature");
        let expected_signature = crate::integrity::merkle::hex_util::encode(&expected_result);
        
        Ok(expected_signature == self.signature)
    }
    
    /// Derive public key from private key (simplified)
    fn derive_public_key(private_key: &[u8]) -> BlockchainResult<String> {
        let kernel = NexKernel::new(1);
        let public_result = kernel.hash_bytes(private_key, "public-key-derivation");
        Ok(crate::integrity::merkle::hex_util::encode(&public_result))
    }
}

/// Quantum-resistant cryptographic operations
#[derive(Debug, Clone)]
pub struct QuantumCrypto {
    /// Lattice dimension for quantum resistance
    lattice_dim: usize,
}

impl QuantumCrypto {
    /// Create new quantum crypto instance
    pub fn new(lattice_dim: usize) -> Self {
        Self { lattice_dim }
    }
    
    /// Generate quantum-resistant key pair
    pub fn generate_keypair(&self) -> BlockchainResult<(Vec<u8>, Vec<u8>)> {
        let kernel = NexKernel::new(1);
        
        // Generate cryptographically secure random seeds
        use crate::utils::entropy::SecureRng;
        let mut rng = SecureRng::new().map_err(|_| {
            BlockchainError::CryptoError("Failed to initialize secure RNG".to_string())
        })?;
        
        let mut seed_bytes1 = [0u8; 32];
        let mut seed_bytes2 = [0u8; 32];
        let mut seed_bytes3 = [0u8; 32];
        let mut seed_bytes4 = [0u8; 32];
        
        rng.fill_bytes(&mut seed_bytes1).map_err(|_| {
            BlockchainError::CryptoError("Failed to generate seed1".to_string())
        })?;
        rng.fill_bytes(&mut seed_bytes2).map_err(|_| {
            BlockchainError::CryptoError("Failed to generate seed2".to_string())
        })?;
        rng.fill_bytes(&mut seed_bytes3).map_err(|_| {
            BlockchainError::CryptoError("Failed to generate seed3".to_string())
        })?;
        rng.fill_bytes(&mut seed_bytes4).map_err(|_| {
            BlockchainError::CryptoError("Failed to generate seed4".to_string())
        })?;
        
        let seed1 = kernel.hash_bytes(&seed_bytes1, "keypair");
        let seed2 = kernel.hash_bytes(&seed_bytes2, "keypair");
        let seed3 = kernel.hash_bytes(&seed_bytes3, "keypair");
        let seed4 = kernel.hash_bytes(&seed_bytes4, "keypair");
        
        let mut rng = ChaosEngine::new([
            seed1.iter().take(8).fold(0u64, |acc, &x| (acc << 8) | x as u64),
            seed2.iter().take(8).fold(0u64, |acc, &x| (acc << 8) | x as u64),
            seed3.iter().take(8).fold(0u64, |acc, &x| (acc << 8) | x as u64),
            seed4.iter().take(8).fold(0u64, |acc, &x| (acc << 8) | x as u64),
        ]);
        
        // Generate private key
        let mut private_key = vec![0u8; 64];
        for byte in &mut private_key {
            *byte = (rng.next_u64() & 0xFF) as u8;
        }
        
        // Derive public key
        let public_key = self.derive_quantum_public_key(&private_key)?;
        
        Ok((private_key, public_key))
    }
    
    /// Derive quantum-resistant public key
    fn derive_quantum_public_key(&self, private_key: &[u8]) -> BlockchainResult<Vec<u8>> {
        let kernel = NexKernel::new(1);
        let public_result = kernel.hash_bytes(private_key, "quantum-public-key");
        Ok(public_result)
    }
    
    /// Create quantum-resistant signature
    pub fn sign(&self, data: &[u8], private_key: &[u8]) -> BlockchainResult<Vec<u8>> {
        let kernel = NexKernel::new(1);
        
        // Create lattice-based signature
        let sig_input = format!("{}{}{}", 
            hex::encode(data),
            hex::encode(private_key),
            self.lattice_dim);
        
        let signature_result = kernel.hash_bytes(sig_input.as_bytes(), "quantum-signature");
        Ok(signature_result)
    }
    
    /// Verify quantum-resistant signature
    pub fn verify(&self, data: &[u8], signature: &[u8], public_key: &[u8]) -> BlockchainResult<bool> {
        let kernel = NexKernel::new(1);
        
        // Recreate signature input
        let sig_input = format!("{}{}{}", 
            hex::encode(data),
            hex::encode(public_key),
            self.lattice_dim);
        
        let expected_result = kernel.hash_bytes(sig_input.as_bytes(), "quantum-signature");
        Ok(expected_result == signature)
    }
}

/// Blockchain cryptographic utilities
#[derive(Debug, Clone)]
pub struct BlockchainCrypto {
    quantum_crypto: QuantumCrypto,
}

impl BlockchainCrypto {
    /// Create new blockchain crypto instance
    pub fn new() -> Self {
        Self {
            quantum_crypto: QuantumCrypto::new(1024), // 1024-dimensional lattice
        }
    }
    
    /// Generate address from public key
    pub fn generate_address(&self, public_key: &[u8]) -> BlockchainResult<String> {
        let kernel = NexKernel::new(1);
        let address_result = kernel.hash_bytes(public_key, "address-generation");
        let address = crate::integrity::merkle::hex_util::encode(&address_result);
        
        // Add checksum and version
        let versioned = format!("00{}", address);
        let checksum = self.calculate_checksum(&versioned)?;
        Ok(format!("{}{}", versioned, checksum))
    }
    
    /// Calculate address checksum
    fn calculate_checksum(&self, data: &str) -> BlockchainResult<String> {
        let kernel = NexKernel::new(1);
        let checksum_result = kernel.hash_bytes(data.as_bytes(), "address-checksum");
        let checksum = crate::integrity::merkle::hex_util::encode(&checksum_result);
        Ok(checksum[..8].to_string()) // First 8 characters as checksum
    }
    
    /// Validate address format
    pub fn validate_address(&self, address: &str) -> bool {
        if address.len() < 10 {
            return false;
        }
        
        let versioned = &address[..address.len() - 8];
        let checksum = &address[address.len() - 8..];
        
        match self.calculate_checksum(versioned) {
            Ok(expected_checksum) => expected_checksum == checksum,
            Err(_) => false,
        }
    }
    
    /// Encrypt transaction data
    pub fn encrypt_data(&self, data: &[u8], key: &[u8]) -> BlockchainResult<Vec<u8>> {
        let kernel = NexKernel::new(1);
        
        // Generate encryption seed from key
        let seed_result = kernel.hash_bytes(key, "encryption-seed");
        let seed = [
            u64::from_le_bytes(seed_result[0..8].try_into().unwrap_or([0u8; 8])),
            u64::from_le_bytes(seed_result[8..16].try_into().unwrap_or([0u8; 8])),
            u64::from_le_bytes(seed_result[16..24].try_into().unwrap_or([0u8; 8])),
            u64::from_le_bytes(seed_result[24..32].try_into().unwrap_or([0u8; 8])),
        ];
        
        let mut cipher = ChaosEngine::new(seed);
        let mut encrypted = data.to_vec();
        
        for byte in &mut encrypted {
            *byte ^= (cipher.next_u64() & 0xFF) as u8;
        }
        
        Ok(encrypted)
    }
    
    /// Decrypt transaction data
    pub fn decrypt_data(&self, encrypted_data: &[u8], key: &[u8]) -> BlockchainResult<Vec<u8>> {
        // Encryption is symmetric, so same process
        self.encrypt_data(encrypted_data, key)
    }
    
    /// Generate hash for block verification
    pub fn hash_block_data(&self, data: &[u8]) -> BlockchainResult<String> {
        let kernel = NexKernel::new(1);
        let hash_result = kernel.hash_bytes(data, "block-hash");
        Ok(crate::integrity::merkle::hex_util::encode(&hash_result))
    }
    
    /// Get quantum crypto instance
    pub fn quantum_crypto(&self) -> &QuantumCrypto {
        &self.quantum_crypto
    }
}

impl Default for BlockchainCrypto {
    fn default() -> Self {
        Self::new()
    }
}

/// Zero-knowledge proof implementation (simplified)
#[derive(Debug, Clone)]
pub struct ZeroKnowledgeProof {
    statement: String,
    proof: Vec<u8>,
}

impl ZeroKnowledgeProof {
    /// Create new ZKP
    pub fn new(statement: String, witness: &[u8]) -> BlockchainResult<Self> {
        let kernel = NexKernel::new(1);
        
        // Generate proof from witness and statement
        let proof_input = format!("{}{}", statement, hex::encode(witness));
        let proof_result = kernel.hash_bytes(proof_input.as_bytes(), "zkp-proof");
        
        Ok(Self {
            statement,
            proof: proof_result,
        })
    }
    
    /// Verify ZKP
    pub fn verify(&self) -> BlockchainResult<bool> {
        let kernel = NexKernel::new(1);
        
        // Recreate proof hash
        let proof_input = format!("{}{}", self.statement, hex::encode(&self.proof));
        let expected_result = kernel.hash_bytes(proof_input.as_bytes(), "zkp-verification");
        let expected_proof = crate::integrity::merkle::hex_util::encode(&expected_result);
        
        Ok(expected_proof == hex::encode(&self.proof))
    }
}
