use crate::kernel::NexKernel;

/// Merkle Tree Implementation for Large Dataset Integrity.
/// Allows verifying 1 TB of data by checking only the Root Hash.
/// 
/// Structure:
/// - Leaves: Hash of data chunks (e.g., 1MB blocks)
/// - Nodes: Hash(Left Child || Right Child)
/// - Root: Topmost hash

#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub hash: Vec<u8>,
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
}

pub struct MerkleTree {
    pub root: Option<MerkleNode>,
    pub leaves: Vec<Vec<u8>>, // Storing all leaf hashes
    kernel: NexKernel,
}

impl MerkleTree {
    pub fn new(data_chunks: Vec<&[u8]>) -> Self {
        let kernel = NexKernel::new(1);
        let mut leaves = Vec::new();

        // 1. Hash all data chunks to create leaves
        for chunk in data_chunks {
            let hash = kernel.hash_bytes(chunk, "Merkle-Leaf");
            leaves.push(hash);
        }

        // 2. Pad to Next Power of 2 (Perfect Binary Tree)
        if !leaves.is_empty() {
            let mut next_pow2 = 1;
            while next_pow2 < leaves.len() {
                next_pow2 *= 2;
            }

            if next_pow2 > leaves.len() {
                // Create a deterministic "Empty/Pad" hash
                let pad_hash = kernel.hash_bytes(&[], "Merkle-Pad");
                while leaves.len() < next_pow2 {
                    leaves.push(pad_hash.clone());
                }
            }
        }

        let mut tree = MerkleTree {
            root: None,
            leaves: leaves.clone(), // Keep track of leaves (including pads)
            kernel: NexKernel::new(1),
        };

        if !leaves.is_empty() {
             tree.root = Some(tree.build_tree(&leaves));
        }

        tree
    }

    /// Recursive build function
    fn build_tree(&self, hashes: &[Vec<u8>]) -> MerkleNode {
        if hashes.len() == 1 {
            return MerkleNode {
                hash: hashes[0].clone(),
                left: None,
                right: None,
            };
        }

        // Divide and Conquer
        let mid = hashes.len() / 2;
        let left_child = self.build_tree(&hashes[0..mid]);
        let right_child = self.build_tree(&hashes[mid..]);

        // Combine hashes: Hash(Left || Right)
        let mut combined = left_child.hash.clone();
        combined.extend_from_slice(&right_child.hash);
        
        // NexKernel hash
        let node_hash = self.kernel.hash_bytes(&combined, "Merkle-Node");

        MerkleNode {
            hash: node_hash,
            left: Some(Box::new(left_child)),
            right: Some(Box::new(right_child)),
        }
    }

    pub fn get_root_hash(&self) -> String {
        match &self.root {
            Some(node) => hex_util::encode(&node.hash),
            None => String::from(""),
        }
    }
}

// Helper for hex encoding since we removed hex crate dependency earlier or might not have it
// We implement a simple hex encoder here to avoid external deps if needed, 
// strictly user asked for "No generated python", but used Rust.
// Assuming 'hex' might not be in Cargo.toml given previous context (only cc).
pub mod hex_util {
    pub fn encode(data: &[u8]) -> String {
        let mut s = String::with_capacity(data.len() * 2);
        for &b in data {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }
}
