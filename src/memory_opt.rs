//! Memory optimization utilities for NEX2426
//! 
//! Provides memory-efficient alternatives for large operations and
//! streaming processing to minimize memory footprint.

use std::io::{Read, BufReader};
use crate::error::{NexResult, NexError};

/// Streaming processor for large inputs
/// Processes data in chunks to avoid loading everything into memory
pub struct StreamingProcessor<R: Read> {
    reader: BufReader<R>,
    chunk_size: usize,
    buffer: Vec<u8>,
}

impl<R: Read> StreamingProcessor<R> {
    /// Create new streaming processor
    pub fn new(reader: R, chunk_size: usize) -> Self {
        Self {
            reader: BufReader::with_capacity(chunk_size, reader),
            chunk_size,
            buffer: Vec::with_capacity(chunk_size),
        }
    }
    
    /// Process data in chunks with the given processor function
    pub fn process_chunks<F>(&mut self, mut processor: F) -> NexResult<()>
    where
        F: FnMut(&[u8]) -> NexResult<()>,
    {
        loop {
            self.buffer.clear();
            self.buffer.resize(self.chunk_size, 0);
            
            let bytes_read = self.reader.read(&mut self.buffer)?;
            if bytes_read == 0 {
                break; // EOF
            }
            
            // Process only the bytes that were actually read
            processor(&self.buffer[..bytes_read])?;
        }
        
        Ok(())
    }
    
    /// Get total size of the underlying reader (if available)
    pub fn size_hint(&self) -> Option<u64> {
        // Try to get size from the reader if it supports seeking
        use std::io::Seek;
        
        // This is a best-effort operation - return None if seeking is not supported
        None
    }
}

/// Zero-copy buffer for efficient data processing
/// Reuses the same memory buffer for multiple operations
pub struct ZeroCopyBuffer {
    buffer: Vec<u8>,
    capacity: usize,
    used: usize,
}

impl ZeroCopyBuffer {
    /// Create new zero-copy buffer
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0u8; capacity],
            capacity,
            used: 0,
        }
    }
    
    /// Get a mutable slice of the buffer
    pub fn get_mut(&mut self, size: usize) -> NexResult<&mut [u8]> {
        if size > self.capacity {
            return Err(NexError::memory(format!(
                "Requested size {} exceeds buffer capacity {}", 
                size, self.capacity
            )));
        }
        
        self.used = size;
        Ok(&mut self.buffer[..size])
    }
    
    /// Get the currently used portion
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer[..self.used]
    }
    
    /// Reset the buffer for reuse
    pub fn reset(&mut self) {
        self.used = 0;
    }
    
    /// Get remaining capacity
    pub fn remaining(&self) -> usize {
        self.capacity - self.used
    }
}

/// Memory-efficient chunk processor
/// Processes data in fixed-size chunks with minimal allocation
pub struct ChunkProcessor {
    chunks: Vec<Vec<u8>>,
    chunk_size: usize,
    max_chunks: usize,
}

impl ChunkProcessor {
    /// Create new chunk processor
    pub fn new(chunk_size: usize, max_chunks: usize) -> Self {
        Self {
            chunks: Vec::with_capacity(max_chunks),
            chunk_size,
            max_chunks,
        }
    }
    
    /// Add a chunk to the processor
    pub fn add_chunk(&mut self, data: &[u8]) -> NexResult<()> {
        if self.chunks.len() >= self.max_chunks {
            return Err(NexError::memory("Maximum chunks exceeded"));
        }
        
        if data.len() > self.chunk_size {
            return Err(NexError::memory(format!(
                "Chunk size {} exceeds maximum {}", 
                data.len(), self.chunk_size
            )));
        }
        
        let mut chunk = vec![0u8; self.chunk_size];
        chunk[..data.len()].copy_from_slice(data);
        self.chunks.push(chunk);
        
        Ok(())
    }
    
    /// Process all chunks with the given function
    pub fn process_chunks<F>(&self, mut processor: F) -> NexResult<()>
    where
        F: FnMut(&[u8]) -> NexResult<()>,
    {
        for chunk in &self.chunks {
            processor(chunk)?;
        }
        Ok(())
    }
    
    /// Get total memory usage
    pub fn memory_usage(&self) -> usize {
        self.chunks.len() * self.chunk_size
    }
    
    /// Clear all chunks
    pub fn clear(&mut self) {
        self.chunks.clear();
    }
}

/// Memory pool for reusable allocations
/// Reduces allocation overhead for frequently used buffer sizes
pub struct MemoryPool {
    pools: std::collections::HashMap<usize, Vec<Vec<u8>>>,
    max_pool_size: usize,
}

impl MemoryPool {
    /// Create new memory pool
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            pools: std::collections::HashMap::new(),
            max_pool_size,
        }
    }
    
    /// Get a buffer of the specified size from the pool
    pub fn get_buffer(&mut self, size: usize) -> Vec<u8> {
        let pool = self.pools.entry(size).or_insert_with(Vec::new);
        
        if let Some(buffer) = pool.pop() {
            buffer
        } else {
            vec![0u8; size]
        }
    }
    
    /// Return a buffer to the pool
    pub fn return_buffer(&mut self, mut buffer: Vec<u8>) {
        let size = buffer.len();
        if let Some(pool) = self.pools.get_mut(&size) {
            if pool.len() < self.max_pool_size {
                // Clear the buffer for security
                buffer.fill(0);
                pool.push(buffer);
            }
        }
    }
    
    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let total_buffers: usize = self.pools.values().map(|v| v.len()).sum();
        let pool_sizes = self.pools.len();
        
        PoolStats {
            total_buffers,
            pool_sizes,
            total_memory: self.pools.iter()
                .map(|(&size, pool)| size * pool.len())
                .sum(),
        }
    }
}

/// Memory pool statistics
#[derive(Debug)]
pub struct PoolStats {
    pub total_buffers: usize,
    pub pool_sizes: usize,
    pub total_memory: usize,
}

/// Streaming hash processor
/// Computes hash of large data without loading everything into memory
pub struct StreamingHasher {
    state: Vec<u8>,
    processed: u64,
}

impl StreamingHasher {
    /// Create new streaming hasher
    pub fn new() -> Self {
        Self {
            state: vec![0u8; 64], // 512-bit state
            processed: 0,
        }
    }
    
    /// Update hash with new data
    pub fn update(&mut self, data: &[u8]) {
        // Simple streaming hash implementation
        for (i, &byte) in data.iter().enumerate() {
            let state_idx = i % self.state.len();
            self.state[state_idx] ^= byte.wrapping_add(i as u8);
        }
        self.processed += data.len() as u64;
    }
    
    /// Finalize the hash and return result
    pub fn finalize(mut self) -> Vec<u8> {
        // Mix the final state
        for i in 0..self.state.len() {
            self.state[i] = self.state[i].rotate_right(3)
                .wrapping_add((self.processed & 0xFF) as u8);
        }
        
        self.state
    }
    
    /// Reset hasher for new computation
    pub fn reset(&mut self) {
        self.state.fill(0);
        self.processed = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_streaming_processor() {
        let data = b"Hello, world! This is a test.";
        let cursor = Cursor::new(data);
        let mut processor = StreamingProcessor::new(cursor, 8);
        
        let mut chunks = Vec::new();
        processor.process_chunks(|chunk| {
            chunks.push(chunk.to_vec());
            Ok(())
        }).unwrap();
        
        assert_eq!(chunks.concat(), data);
    }

    #[test]
    fn test_zero_copy_buffer() {
        let mut buffer = ZeroCopyBuffer::new(100);
        let slice = buffer.get_mut(50).unwrap();
        slice.copy_from_slice(b"Hello, world! This is a test message.");
        
        assert_eq!(buffer.as_slice(), b"Hello, world! This is a test message.");
    }

    #[test]
    fn test_memory_pool() {
        let mut pool = MemoryPool::new(10);
        
        let buf1 = pool.get_buffer(100);
        let buf2 = pool.get_buffer(100);
        
        pool.return_buffer(buf1);
        pool.return_buffer(buf2);
        
        let stats = pool.stats();
        assert_eq!(stats.total_buffers, 2);
        assert_eq!(stats.pool_sizes, 1);
    }

    #[test]
    fn test_streaming_hasher() {
        let mut hasher = StreamingHasher::new();
        
        hasher.update(b"Hello, ");
        hasher.update(b"world!");
        
        let result = hasher.finalize();
        assert!(!result.iter().all(|&x| *x == 0));
    }
}
