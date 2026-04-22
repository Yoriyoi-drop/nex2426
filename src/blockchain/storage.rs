//! Storage layer for NEX2426 Blockchain
//! 
//! Provides persistence and retrieval mechanisms for blockchain data

use crate::blockchain::{Block, BlockchainError, BlockchainResult};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Storage backend trait
pub trait ChainStorage: Send + Sync {
    /// Store a block
    fn store_block(&self, block: &Block) -> BlockchainResult<()>;
    
    /// Get block by height
    fn get_block(&self, height: u64) -> BlockchainResult<Option<Block>>;
    
    /// Get block by hash
    fn get_block_by_hash(&self, hash: &str) -> BlockchainResult<Option<Block>>;
    
    /// Get latest block height
    fn get_latest_height(&self) -> BlockchainResult<u64>;
    
    /// Store blockchain metadata
    fn store_metadata(&self, key: &str, value: &[u8]) -> BlockchainResult<()>;
    
    /// Get blockchain metadata
    fn get_metadata(&self, key: &str) -> BlockchainResult<Option<Vec<u8>>>;
}

/// In-memory storage implementation
#[derive(Debug)]
pub struct MemoryStorage {
    blocks: Arc<RwLock<HashMap<u64, Block>>>,
    hash_index: Arc<RwLock<HashMap<String, u64>>>,
    metadata: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    latest_height: Arc<RwLock<u64>>,
}

impl MemoryStorage {
    /// Create new memory storage
    pub fn new() -> Self {
        Self {
            blocks: Arc::new(RwLock::new(HashMap::new())),
            hash_index: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            latest_height: Arc::new(RwLock::new(0)),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ChainStorage for MemoryStorage {
    fn store_block(&self, block: &Block) -> BlockchainResult<()> {
        let height = block.get_height();
        let hash = block.get_hash().unwrap_or(&"".to_string()).clone();
        
        // Store block
        {
            let mut blocks = self.blocks.write()
                .map_err(|_| BlockchainError::StorageError("Failed to acquire write lock".to_string()))?;
            blocks.insert(height, block.clone());
        }
        
        // Update hash index
        {
            let mut hash_index = self.hash_index.write()
                .map_err(|_| BlockchainError::StorageError("Failed to acquire write lock".to_string()))?;
            hash_index.insert(hash.to_string(), height);
        }
        
        // Update latest height
        {
            let mut latest_height = self.latest_height.write()
                .map_err(|_| BlockchainError::StorageError("Failed to acquire write lock".to_string()))?;
            if height > *latest_height {
                *latest_height = height;
            }
        }
        
        Ok(())
    }

    fn get_block(&self, height: u64) -> BlockchainResult<Option<Block>> {
        let blocks = self.blocks.read()
            .map_err(|_| BlockchainError::StorageError("Failed to acquire read lock".to_string()))?;
        Ok(blocks.get(&height).cloned())
    }

    fn get_block_by_hash(&self, hash: &str) -> BlockchainResult<Option<Block>> {
        let hash_index = self.hash_index.read()
            .map_err(|_| BlockchainError::StorageError("Failed to acquire read lock".to_string()))?;
        
        if let Some(&height) = hash_index.get(hash) {
            self.get_block(height)
        } else {
            Ok(None)
        }
    }

    fn get_latest_height(&self) -> BlockchainResult<u64> {
        let latest_height = self.latest_height.read()
            .map_err(|_| BlockchainError::StorageError("Failed to acquire read lock".to_string()))?;
        Ok(*latest_height)
    }

    fn store_metadata(&self, key: &str, value: &[u8]) -> BlockchainResult<()> {
        let mut metadata = self.metadata.write()
            .map_err(|_| BlockchainError::StorageError("Failed to acquire write lock".to_string()))?;
        metadata.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn get_metadata(&self, key: &str) -> BlockchainResult<Option<Vec<u8>>> {
        let metadata = self.metadata.read()
            .map_err(|_| BlockchainError::StorageError("Failed to acquire read lock".to_string()))?;
        Ok(metadata.get(key).cloned())
    }
}

/// File-based storage implementation
#[derive(Debug)]
pub struct FileStorage {
    base_path: PathBuf,
    blocks_dir: PathBuf,
    metadata_dir: PathBuf,
    cache: Arc<RwLock<HashMap<u64, Block>>>,
    hash_index: Arc<RwLock<HashMap<String, u64>>>,
}

impl FileStorage {
    /// Create new file storage
    pub fn new<P: AsRef<Path>>(base_path: P) -> BlockchainResult<Self> {
        let base_path = base_path.as_ref();
        
        // Create directories
        let blocks_dir = base_path.join("blocks");
        let metadata_dir = base_path.join("metadata");
        
        fs::create_dir_all(&blocks_dir)
            .map_err(|e| BlockchainError::StorageError(format!("Failed to create blocks directory: {}", e)))?;
        fs::create_dir_all(&metadata_dir)
            .map_err(|e| BlockchainError::StorageError(format!("Failed to create metadata directory: {}", e)))?;

        let storage = Self {
            base_path: base_path.to_path_buf(),
            blocks_dir,
            metadata_dir,
            cache: Arc::new(RwLock::new(HashMap::new())),
            hash_index: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load existing blocks into cache
        storage.load_cache()?;
        Ok(storage)
    }

    /// Load existing blocks from disk into cache
    fn load_cache(&self) -> BlockchainResult<()> {
        let entries = fs::read_dir(&self.blocks_dir)
            .map_err(|e| BlockchainError::StorageError(format!("Failed to read blocks directory: {}", e)))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| BlockchainError::StorageError(format!("Failed to read directory entry: {}", e)))?;
            
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(height) = filename.parse::<u64>() {
                        if let Ok(block) = self.load_block_from_file(height) {
                            // Update cache
                            {
                                let mut cache = self.cache.write()
                                    .map_err(|_| BlockchainError::StorageError("Failed to acquire write lock".to_string()))?;
                                cache.insert(height, block.clone());
                            }
                            
                            // Update hash index
                            if let Some(ref hash) = block.get_hash() {
                                let mut hash_index = self.hash_index.write()
                                    .map_err(|_| BlockchainError::StorageError("Failed to acquire write lock".to_string()))?;
                                hash_index.insert(hash.to_string(), height);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Load block from file
    fn load_block_from_file(&self, height: u64) -> BlockchainResult<Block> {
        let file_path = self.blocks_dir.join(format!("{}.json", height));
        let content = fs::read_to_string(&file_path)
            .map_err(|e| BlockchainError::StorageError(format!("Failed to read block file: {}", e)))?;
        
        serde_json::from_str(&content)
            .map_err(|e| BlockchainError::StorageError(format!("Failed to deserialize block: {}", e)))
    }

    /// Save block to file
    fn save_block_to_file(&self, block: &Block) -> BlockchainResult<()> {
        let file_path = self.blocks_dir.join(format!("{}.json", block.get_height()));
        let content = serde_json::to_string_pretty(block)
            .map_err(|e| BlockchainError::StorageError(format!("Failed to serialize block: {}", e)))?;
        
        fs::write(&file_path, content)
            .map_err(|e| BlockchainError::StorageError(format!("Failed to write block file: {}", e)))?;
        
        Ok(())
    }

    /// Get metadata file path
    fn get_metadata_path(&self, key: &str) -> PathBuf {
        self.metadata_dir.join(format!("{}.json", key))
    }
}

impl ChainStorage for FileStorage {
    fn store_block(&self, block: &Block) -> BlockchainResult<()> {
        let height = block.get_height();
        let hash = block.get_hash().unwrap_or(&"".to_string()).clone();
        
        // Save to file
        self.save_block_to_file(block)?;
        
        // Update cache
        {
            let mut cache = self.cache.write()
                .map_err(|_| BlockchainError::StorageError("Failed to acquire write lock".to_string()))?;
            cache.insert(height, block.clone());
        }
        
        // Update hash index
        {
            let mut hash_index = self.hash_index.write()
                .map_err(|_| BlockchainError::StorageError("Failed to acquire write lock".to_string()))?;
            hash_index.insert(hash.to_string(), height);
        }
        
        Ok(())
    }

    fn get_block(&self, height: u64) -> BlockchainResult<Option<Block>> {
        // Check cache first
        {
            let cache = self.cache.read()
                .map_err(|_| BlockchainError::StorageError("Failed to acquire read lock".to_string()))?;
            if let Some(block) = cache.get(&height) {
                return Ok(Some(block.clone()));
            }
        }
        
        // Load from file if not in cache
        match self.load_block_from_file(height) {
            Ok(block) => {
                // Update cache
                {
                    let mut cache = self.cache.write()
                        .map_err(|_| BlockchainError::StorageError("Failed to acquire write lock".to_string()))?;
                    cache.insert(height, block.clone());
                }
                Ok(Some(block))
            }
            Err(_) => Ok(None),
        }
    }

    fn get_block_by_hash(&self, hash: &str) -> BlockchainResult<Option<Block>> {
        let hash_index = self.hash_index.read()
            .map_err(|_| BlockchainError::StorageError("Failed to acquire read lock".to_string()))?;
        
        if let Some(&height) = hash_index.get(hash) {
            self.get_block(height)
        } else {
            Ok(None)
        }
    }

    fn get_latest_height(&self) -> BlockchainResult<u64> {
        let cache = self.cache.read()
            .map_err(|_| BlockchainError::StorageError("Failed to acquire read lock".to_string()))?;
        
        if cache.is_empty() {
            Ok(0)
        } else {
            Ok(*cache.keys().max().unwrap_or(&0))
        }
    }

    fn store_metadata(&self, key: &str, value: &[u8]) -> BlockchainResult<()> {
        let file_path = self.get_metadata_path(key);
        let content = serde_json::json!({
            "key": key,
            "value": hex::encode(value),
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs()
        });
        
        let content_str = serde_json::to_string_pretty(&content)
            .map_err(|e| BlockchainError::StorageError(format!("Failed to serialize metadata: {}", e)))?;
        
        fs::write(&file_path, content_str)
            .map_err(|e| BlockchainError::StorageError(format!("Failed to write metadata file: {}", e)))?;
        
        Ok(())
    }

    fn get_metadata(&self, key: &str) -> BlockchainResult<Option<Vec<u8>>> {
        let file_path = self.get_metadata_path(key);
        
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                let parsed: serde_json::Value = serde_json::from_str(&content)
                    .map_err(|e| BlockchainError::StorageError(format!("Failed to parse metadata: {}", e)))?;
                
                if let Some(value_str) = parsed.get("value").and_then(|v| v.as_str()) {
                    hex::decode(value_str)
                        .map(Some)
                        .map_err(|e| BlockchainError::StorageError(format!("Failed to decode metadata value: {}", e)))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }
}

/// Database storage (placeholder for future implementation)
#[derive(Debug)]
pub struct DatabaseStorage {
    connection_string: String,
}

impl DatabaseStorage {
    /// Create new database storage
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
}

impl ChainStorage for DatabaseStorage {
    fn store_block(&self, _block: &Block) -> BlockchainResult<()> {
        // TODO: Implement database storage
        Err(BlockchainError::StorageError("Database storage not implemented".to_string()))
    }

    fn get_block(&self, _height: u64) -> BlockchainResult<Option<Block>> {
        // TODO: Implement database retrieval
        Err(BlockchainError::StorageError("Database storage not implemented".to_string()))
    }

    fn get_block_by_hash(&self, _hash: &str) -> BlockchainResult<Option<Block>> {
        // TODO: Implement database hash lookup
        Err(BlockchainError::StorageError("Database storage not implemented".to_string()))
    }

    fn get_latest_height(&self) -> BlockchainResult<u64> {
        // TODO: Implement database height query
        Err(BlockchainError::StorageError("Database storage not implemented".to_string()))
    }

    fn store_metadata(&self, _key: &str, _value: &[u8]) -> BlockchainResult<()> {
        // TODO: Implement database metadata storage
        Err(BlockchainError::StorageError("Database storage not implemented".to_string()))
    }

    fn get_metadata(&self, _key: &str) -> BlockchainResult<Option<Vec<u8>>> {
        // TODO: Implement database metadata retrieval
        Err(BlockchainError::StorageError("Database storage not implemented".to_string()))
    }
}
