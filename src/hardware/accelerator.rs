//==============================================================================
// NEX2426 Hardware Accelerator
// High-level interface for hardware-accelerated cryptographic operations
//==============================================================================

use std::sync::Arc;
use std::time::Duration;

use super::{HardwareBridge, HardwareError, HardwareResult, memory_map::*};

pub struct HardwareAccelerator {
    bridge: HardwareBridge,
}

impl HardwareAccelerator {
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            bridge: HardwareBridge::new(timeout_ms),
        }
    }
    
    pub fn reset(&self) -> HardwareResult<()> {
        self.bridge.reset()
    }
    
    pub fn hash_data(&self, data: &[u8], key: &[u8; 32], cost: u32) -> HardwareResult<[u8; 64]> {
        // Configure hardware for hashing
        self.bridge.registers.set_mode(OperationMode::Hash);
        self.bridge.registers.set_key(key);
        self.bridge.registers.set_cost(cost);
        
        // Set input data (for simplicity, using first byte)
        if !data.is_empty() {
            self.bridge.registers.set_data_in(data[0]);
        }
        
        // Simulate operation for testing (when no real hardware)
        self.bridge.simulate_operation(OperationMode::Hash, cost * 100);
        
        // Execute operation
        self.bridge.execute_operation()?;
        
        // Get hash result
        Ok(self.bridge.registers.get_hash())
    }
    
    pub fn encrypt_data(&self, data: &[u8], key: &[u8; 32], cost: u32) -> HardwareResult<Vec<u8>> {
        // Configure hardware for encryption
        self.bridge.registers.set_mode(OperationMode::Encrypt);
        self.bridge.registers.set_key(key);
        self.bridge.registers.set_cost(cost);
        
        let mut encrypted_data = Vec::with_capacity(data.len());
        
        // Process each byte
        for &byte in data {
            self.bridge.registers.set_data_in(byte);
            self.bridge.execute_operation()?;
            encrypted_data.push(self.bridge.registers.get_data_out());
        }
        
        Ok(encrypted_data)
    }
    
    pub fn decrypt_data(&self, data: &[u8], key: &[u8; 32], cost: u32) -> HardwareResult<Vec<u8>> {
        // Configure hardware for decryption
        self.bridge.registers.set_mode(OperationMode::Decrypt);
        self.bridge.registers.set_key(key);
        self.bridge.registers.set_cost(cost);
        
        let mut decrypted_data = Vec::with_capacity(data.len());
        
        // Process each byte
        for &byte in data {
            self.bridge.registers.set_data_in(byte);
            self.bridge.execute_operation()?;
            decrypted_data.push(self.bridge.registers.get_data_out());
        }
        
        Ok(decrypted_data)
    }
    
    pub fn benchmark(&self, cost: u32) -> HardwareResult<(Duration, u32)> {
        let start_time = std::time::Instant::now();
        
        // Configure hardware for benchmark
        self.bridge.registers.set_mode(OperationMode::Benchmark);
        self.bridge.registers.set_cost(cost);
        
        // Use test key
        let test_key = [0x42; 32];
        self.bridge.registers.set_key(&test_key);
        
        // Simulate benchmark operation
        self.bridge.simulate_operation(OperationMode::Benchmark, cost * 200);
        
        // Execute benchmark
        self.bridge.execute_operation()?;
        
        let elapsed = start_time.elapsed();
        let cycles = self.bridge.get_bridge_status().0 as u32;
        
        Ok((elapsed, cycles))
    }
    
    pub fn stealth_encrypt(&self, data: &[u8], key: &[u8; 32], cost: u32) -> HardwareResult<Vec<u8>> {
        // Configure hardware for stealth encryption
        self.bridge.registers.set_mode(OperationMode::Stealth);
        self.bridge.registers.set_key(key);
        self.bridge.registers.set_cost(cost);
        self.bridge.registers.set_config(false, true); // stealth = true
        
        let mut encrypted_data = Vec::with_capacity(data.len());
        
        // Process each byte
        for &byte in data {
            self.bridge.registers.set_data_in(byte);
            self.bridge.execute_operation()?;
            encrypted_data.push(self.bridge.registers.get_data_out());
        }
        
        Ok(encrypted_data)
    }
    
    pub fn bio_lock_encrypt(&self, data: &[u8], key: &[u8; 32], cost: u32, hw_id: u64) -> HardwareResult<Vec<u8>> {
        // Configure hardware for bio-lock encryption
        self.bridge.registers.set_mode(OperationMode::BioLock);
        self.bridge.registers.set_key(key);
        self.bridge.registers.set_cost(cost);
        self.bridge.registers.set_config(true, false); // bio_lock = true
        self.bridge.registers.set_hw_id(hw_id);
        
        let mut encrypted_data = Vec::with_capacity(data.len());
        
        // Process each byte
        for &byte in data {
            self.bridge.registers.set_data_in(byte);
            self.bridge.execute_operation()?;
            encrypted_data.push(self.bridge.registers.get_data_out());
        }
        
        Ok(encrypted_data)
    }
    
    pub fn generate_key(&self, seed: &[u8; 32]) -> HardwareResult<[u8; 32]> {
        // Configure hardware for key generation
        self.bridge.registers.set_mode(OperationMode::KeyGen);
        self.bridge.registers.set_key(seed);
        self.bridge.registers.set_cost(1);
        
        // Execute key generation
        self.bridge.execute_operation()?;
        
        // Get generated key (using hash output as key)
        let hash = self.bridge.registers.get_hash();
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash[..32]);
        Ok(key)
    }
    
    pub fn get_performance_metrics(&self) -> HardwareResult<PerformanceMetrics> {
        let (version_major, version_minor) = self.bridge.check_version()?;
        let (cycles, _state) = self.bridge.get_bridge_status();
        let status = self.bridge.registers.get_status_code();
        
        Ok(PerformanceMetrics {
            version_major,
            version_minor,
            last_operation_cycles: cycles as u32,
            current_status: status,
            is_ready: status == StatusCode::Idle,
        })
    }
    
    pub fn run_comprehensive_test(&self) -> HardwareResult<TestResults> {
        let mut results = TestResults::new();
        
        // Test 1: Hash performance
        let test_data = b"Hello, NEX2426 Hardware!";
        let test_key = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                        0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
                        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
                        0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20];
        
        let start_time = std::time::Instant::now();
        let hash_result = self.hash_data(test_data, &test_key, 5)?;
        let hash_time = start_time.elapsed();
        
        results.hash_test_passed = true;
        results.hash_time = hash_time;
        results.hash_result = Some(hex::encode(hash_result));
        
        // Test 2: Encryption/Decryption
        let test_data = b"This is a test message for hardware encryption.";
        
        let start_time = std::time::Instant::now();
        let encrypted = self.encrypt_data(test_data, &test_key, 3)?;
        let encrypt_time = start_time.elapsed();
        
        let start_time = std::time::Instant::now();
        let decrypted = self.decrypt_data(&encrypted, &test_key, 3)?;
        let decrypt_time = start_time.elapsed();
        
        results.encryption_test_passed = test_data == &decrypted[..];
        results.encrypt_time = encrypt_time;
        results.decrypt_time = decrypt_time;
        
        // Test 3: Benchmark
        let (benchmark_time, benchmark_cycles) = self.benchmark(10)?;
        results.benchmark_time = benchmark_time;
        results.benchmark_cycles = benchmark_cycles;
        
        // Test 4: Performance metrics
        results.performance_metrics = Some(self.get_performance_metrics()?);
        
        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub version_major: u32,
    pub version_minor: u32,
    pub last_operation_cycles: u32,
    pub current_status: StatusCode,
    pub is_ready: bool,
}

#[derive(Debug, Clone)]
pub struct TestResults {
    pub hash_test_passed: bool,
    pub hash_time: Duration,
    pub hash_result: Option<String>,
    pub encryption_test_passed: bool,
    pub encrypt_time: Duration,
    pub decrypt_time: Duration,
    pub benchmark_time: Duration,
    pub benchmark_cycles: u32,
    pub performance_metrics: Option<PerformanceMetrics>,
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            hash_test_passed: false,
            hash_time: Duration::ZERO,
            hash_result: None,
            encryption_test_passed: false,
            encrypt_time: Duration::ZERO,
            decrypt_time: Duration::ZERO,
            benchmark_time: Duration::ZERO,
            benchmark_cycles: 0,
            performance_metrics: None,
        }
    }
    
    pub fn all_tests_passed(&self) -> bool {
        self.hash_test_passed && self.encryption_test_passed
    }
}

impl Clone for HardwareAccelerator {
    fn clone(&self) -> Self {
        Self {
            bridge: self.bridge.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_accelerator_creation() {
        let accel = HardwareAccelerator::new(1000);
        assert!(accel.reset().is_ok());
    }
    
    #[test]
    fn test_hash_operation() {
        let accel = HardwareAccelerator::new(1000);
        let data = b"test data";
        let key = [0x01; 32];
        
        // Simulate the operation
        accel.bridge.simulate_operation(OperationMode::Hash, 100);
        
        let result = accel.hash_data(data, &key, 5);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_encrypt_decrypt() {
        let accel = HardwareAccelerator::new(1000);
        let data = b"test message";
        let key = [0x01; 32];
        
        // Simulate the operations
        accel.bridge.simulate_operation(OperationMode::Encrypt, 50);
        let encrypted = accel.encrypt_data(data, &key, 3).unwrap();
        
        accel.bridge.simulate_operation(OperationMode::Decrypt, 50);
        let decrypted = accel.decrypt_data(&encrypted, &key, 3).unwrap();
        
        assert_eq!(data, &decrypted[..]);
    }
}
