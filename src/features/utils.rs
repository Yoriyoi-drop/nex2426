//! Utility functions and helpers for NEX2426
//! 
//! Provides common utilities for file operations, data conversion,
//! and other helpful functionality.

use base64::{Engine as _, engine::general_purpose};
use crate::kernel::NexKernel;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// Batch processing utilities
pub struct BatchProcessor {
    /// Default cost parameter
    default_cost: u32,
    /// Enable parallel processing
    parallel: bool,
}

impl BatchProcessor {
    /// Create new batch processor
    pub fn new(default_cost: u32, parallel: bool) -> Self {
        Self {
            default_cost,
            parallel,
        }
    }
    
    /// Process multiple inputs in batch
    pub fn hash_batch(&self, inputs: Vec<(String, String)>) -> Vec<BatchResult> {
        if self.parallel {
            self.hash_batch_parallel(inputs)
        } else {
            self.hash_batch_sequential(inputs)
        }
    }
    
    /// Sequential batch processing
    fn hash_batch_sequential(&self, inputs: Vec<(String, String)>) -> Vec<BatchResult> {
        inputs.into_iter().enumerate().map(|(index, (data, key))| {
            let start_time = std::time::Instant::now();
            
            let result = NexKernel::new(self.default_cost)
                .execute(&mut io::Cursor::new(&data), &key);
            
            BatchResult {
                index,
                success: true,
                result: Some(result.full_formatted_string),
                error: None,
                duration_ms: start_time.elapsed().as_millis() as u64,
            }
        }).collect()
    }
    
    /// Parallel batch processing
    fn hash_batch_parallel(&self, inputs: Vec<(String, String)>) -> Vec<BatchResult> {
        use rayon::prelude::*;
        
        inputs.into_par_iter().enumerate().map(|(index, (data, key))| {
            let start_time = std::time::Instant::now();
            
            let result = NexKernel::new(self.default_cost)
                .execute(&mut io::Cursor::new(&data), &key);
            
            BatchResult {
                index,
                success: true,
                result: Some(result.full_formatted_string),
                error: None,
                duration_ms: start_time.elapsed().as_millis() as u64,
            }
        }).collect()
    }
}

#[derive(Debug, Clone)]
pub struct BatchResult {
    /// Index in batch
    pub index: usize,
    /// Success status
    pub success: bool,
    /// Result hash (if successful)
    pub result: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Processing time in milliseconds
    pub duration_ms: u64,
}

/// Data format conversion utilities
pub struct DataConverter;

impl DataConverter {
    /// Convert hash to different formats
    pub fn convert_hash_format(hash: &str, format: &str) -> Result<String, String> {
        match format.to_lowercase().as_str() {
            "base64" => {
                // Extract hash part and convert to base64
                let hash_part = hash.split('$').next_back().unwrap_or(hash);
                Ok(general_purpose::STANDARD.encode(hash_part))
            },
            "hex" => {
                // Convert to hex representation
                let hash_part = hash.split('$').next_back().unwrap_or(hash);
                Ok(hex::encode(hash_part.as_bytes()))
            },
            "json" => {
                Ok(serde_json::json!({
                    "hash": hash,
                    "format": "nex6",
                    "timestamp": chrono::Utc::now().timestamp()
                }).to_string())
            },
            "raw" => {
                Ok(hash.split('$').next_back().unwrap_or(hash).to_string())
            },
            _ => Err(format!("Unsupported format: {}", format)),
        }
    }
    
    /// Parse input data from various formats
    pub fn parse_input_data(input: &str, format: &str) -> Result<String, String> {
        match format.to_lowercase().as_str() {
            "string" | "text" => Ok(input.to_string()),
            "base64" => {
                general_purpose::STANDARD
                    .decode(input)
                    .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
                    .map_err(|e| format!("Invalid base64: {}", e))
            },
            "hex" => {
                hex::decode(input)
                    .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
                    .map_err(|e| format!("Invalid hex: {}", e))
            },
            "file" => {
                fs::read_to_string(input)
                    .map_err(|e| format!("Cannot read file: {}", e))
            },
            _ => Err(format!("Unsupported input format: {}", format)),
        }
    }
}

/// File utilities
pub struct FileUtils;

impl FileUtils {
    /// Secure file deletion (overwrite multiple times)
    pub fn secure_delete<P: AsRef<Path>>(path: P) -> io::Result<()> {
        let metadata = fs::metadata(&path)?;
        let file_size = metadata.len() as usize;
        
        // Overwrite with random data 3 times
        for _round in 0..3 {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .open(&path)?;
            
            let random_data: Vec<u8> = (0..file_size)
                .map(|_| rand::random::<u8>())
                .collect();
            
            file.write_all(&random_data)?;
            file.sync_all()?;
        }
        
        // Finally, remove the file
        fs::remove_file(path)?;
        Ok(())
    }
    
    /// Calculate file hash
    pub fn file_hash<P: AsRef<Path>>(path: P, key: &str, cost: u32) -> io::Result<String> {
        let mut file = fs::File::open(path)?;
        let kernel = NexKernel::new(cost);
        
        let kernel_result = kernel.execute(&mut file, key);
        Ok(kernel_result.full_formatted_string)
    }
    
    /// Verify file integrity
    pub fn verify_file<P: AsRef<Path>>(
        path: P, 
        expected_hash: &str, 
        key: &str, 
        cost: u32
    ) -> io::Result<bool> {
        let actual_hash = Self::file_hash(path, key, cost)?;
        Ok(actual_hash == expected_hash)
    }
    
    /// Split large file into chunks for processing
    pub fn split_file<P: AsRef<Path>>(
        path: P, 
        chunk_size: usize
    ) -> io::Result<Vec<Vec<u8>>> {
        let mut file = fs::File::open(path)?;
        let mut chunks = Vec::new();
        
        loop {
            let mut chunk = vec![0u8; chunk_size];
            let bytes_read = file.read(&mut chunk)?;
            
            if bytes_read == 0 {
                break;
            }
            
            chunk.truncate(bytes_read);
            chunks.push(chunk);
        }
        
        Ok(chunks)
    }
}

/// Performance monitoring utilities
pub struct PerformanceMonitor {
    metrics: HashMap<String, Vec<u64>>,
}

impl PerformanceMonitor {
    /// Create new performance monitor
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }
    
    /// Record operation timing
    pub fn record_timing(&mut self, operation: &str, duration_ms: u64) {
        self.metrics
            .entry(operation.to_string())
            .or_default()
            .push(duration_ms);
    }
    
    /// Get statistics for an operation
    pub fn get_stats(&self, operation: &str) -> Option<PerformanceStats> {
        let timings = self.metrics.get(operation)?;
        
        if timings.is_empty() {
            return None;
        }
        
        let total: u64 = timings.iter().sum();
        let avg = total / timings.len() as u64;
        let min = *timings.iter().min().unwrap_or(&0);
        let max = *timings.iter().max().unwrap_or(&0);
        
        // Calculate median
        let mut sorted_timings = timings.clone();
        sorted_timings.sort();
        let median = if sorted_timings.len() % 2 == 0 {
            let mid = sorted_timings.len() / 2;
            (sorted_timings[mid - 1] + sorted_timings[mid]) / 2
        } else {
            sorted_timings[sorted_timings.len() / 2]
        };
        
        Some(PerformanceStats {
            operation: operation.to_string(),
            count: timings.len(),
            total_ms: total,
            avg_ms: avg,
            min_ms: min,
            max_ms: max,
            median_ms: median,
        })
    }
    
    /// Get all statistics
    pub fn get_all_stats(&self) -> HashMap<String, PerformanceStats> {
        self.metrics.keys()
            .filter_map(|op| self.get_stats(op).map(|stats| (op.clone(), stats)))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// Operation name
    pub operation: String,
    /// Number of operations
    pub count: usize,
    /// Total time in milliseconds
    pub total_ms: u64,
    /// Average time in milliseconds
    pub avg_ms: u64,
    /// Minimum time in milliseconds
    pub min_ms: u64,
    /// Maximum time in milliseconds
    pub max_ms: u64,
    /// Median time in milliseconds
    pub median_ms: u64,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_batch_processor() {
        let processor = BatchProcessor::new(1, false);
        let inputs = vec![
            ("test1".to_string(), "key1".to_string()),
            ("test2".to_string(), "key2".to_string()),
        ];
        
        let results = processor.hash_batch(inputs);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.success));
    }
    
    #[test]
    fn test_data_converter() {
        let hash = "$nex6$1$abc123$def456";
        
        let raw = DataConverter::convert_hash_format(hash, "raw").expect("Failed to convert to raw format");
        assert_eq!(raw, "def456");
        
        let json = DataConverter::convert_hash_format(hash, "json").expect("Failed to convert to json format");
        assert!(json.contains("hash"));
        assert!(json.contains("nex6"));
    }
    
    #[test]
    fn test_file_utils() {
        let dir = tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test.txt");
        
        // Create test file
        fs::write(&file_path, "test content").expect("Failed to write test file");
        
        // Calculate hash
        let hash = FileUtils::file_hash(&file_path, "test_key", 1).expect("Failed to calculate file hash");
        assert!(!hash.is_empty());
        
        // Verify file
        let is_valid = FileUtils::verify_file(&file_path, &hash, "test_key", 1).expect("Failed to verify file");
        assert!(is_valid);
    }
    
    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        
        monitor.record_timing("test_op", 100);
        monitor.record_timing("test_op", 200);
        monitor.record_timing("test_op", 150);
        
        let stats = monitor.get_stats("test_op").expect("Failed to get stats");
        assert_eq!(stats.count, 3);
        assert_eq!(stats.total_ms, 450);
        assert_eq!(stats.avg_ms, 150);
        assert_eq!(stats.min_ms, 100);
        assert_eq!(stats.max_ms, 200);
    }
}
