//==============================================================================
// NEX2426 Hardware Bridge
// Rust implementation of hardware bridge interface
//==============================================================================

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use super::{HardwareError, HardwareResult, memory_map::*};

pub struct HardwareBridge {
    pub registers: Arc<HardwareRegisters>,
    timeout: Duration,
}

impl HardwareBridge {
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            registers: Arc::new(HardwareRegisters::default()),
            timeout: Duration::from_millis(timeout_ms),
        }
    }
    
    pub fn get_registers(&self) -> &HardwareRegisters {
        &self.registers
    }
    
    pub fn reset(&self) -> HardwareResult<()> {
        // Reset all registers to default values
        self.registers.control.store(0, Ordering::SeqCst);
        self.registers.status.store(0, Ordering::SeqCst);
        self.registers.mode.store(0, Ordering::SeqCst);
        self.registers.cost.store(1, Ordering::SeqCst);
        self.registers.config.store(0, Ordering::SeqCst);
        
        // Clear key registers
        self.registers.key0.store(0, Ordering::SeqCst);
        self.registers.key1.store(0, Ordering::SeqCst);
        self.registers.key2.store(0, Ordering::SeqCst);
        self.registers.key3.store(0, Ordering::SeqCst);
        
        // Clear data registers
        self.registers.data_in.store(0, Ordering::SeqCst);
        self.registers.data_out.store(0, Ordering::SeqCst);
        
        // Clear hash registers
        for i in 0..8 {
            let hash_reg = match i {
                0 => &self.registers.hash0,
                1 => &self.registers.hash1,
                2 => &self.registers.hash2,
                3 => &self.registers.hash3,
                4 => &self.registers.hash4,
                5 => &self.registers.hash5,
                6 => &self.registers.hash6,
                _ => &self.registers.hash7,
            };
            hash_reg.store(0, Ordering::SeqCst);
        }
        
        Ok(())
    }
    
    pub fn start_operation(&self) -> HardwareResult<()> {
        // Check if hardware is ready
        let status = self.registers.get_status_code();
        if status == StatusCode::Busy {
            return Err(HardwareError::AcceleratorError);
        }
        
        // Start the operation
        self.registers.set_start(true);
        
        // Clear start bit after one cycle (simulated)
        thread::sleep(Duration::from_nanos(10));
        self.registers.set_start(false);
        
        Ok(())
    }
    
    pub fn wait_for_completion(&self) -> HardwareResult<()> {
        let start_time = Instant::now();
        
        loop {
            if self.registers.is_done() {
                let status = self.registers.get_status_code();
                match status {
                    StatusCode::Success => return Ok(()),
                    StatusCode::ErrorKey => return Err(HardwareError::InvalidOperation),
                    StatusCode::ErrorCost => return Err(HardwareError::InvalidOperation),
                    StatusCode::ErrorMode => return Err(HardwareError::InvalidOperation),
                    StatusCode::ErrorMemory => return Err(HardwareError::HardwareFault),
                    StatusCode::ErrorCrypto => return Err(HardwareError::HardwareFault),
                    _ => return Err(HardwareError::AcceleratorError),
                }
            }
            
            if start_time.elapsed() > self.timeout {
                return Err(HardwareError::Timeout);
            }
            
            thread::sleep(Duration::from_micros(100));
        }
    }
    
    pub fn execute_operation(&self) -> HardwareResult<()> {
        self.start_operation()?;
        self.wait_for_completion()
    }
    
    pub fn get_bridge_status(&self) -> (u16, u8) {
        let status = self.registers.bridge_status.load(Ordering::SeqCst);
        let cycle_count = (status >> 16) as u16;
        let state = (status >> 8) as u8;
        let _error = (status & 0xFF) as u8;
        (cycle_count, state)
    }
    
    pub fn check_version(&self) -> HardwareResult<(u32, u32)> {
        let (major, minor) = self.registers.get_version();
        if major == 0 && minor == 0 {
            return Err(HardwareError::BridgeError);
        }
        Ok((major, minor))
    }
    
    // Simulate hardware status updates (for testing without actual hardware)
    pub fn simulate_operation(&self, _mode: OperationMode, cycles: u32) {
        // Set status to busy (main status in bits 8-15)
        self.registers.status.store((1 << 8), Ordering::SeqCst);
        
        // Simulate operation delay
        thread::sleep(Duration::from_micros(cycles as u64 * 10));
        
        // Set status to success with done bit
        self.registers.status.store(
            (2 << 8) | STATUS_DONE_MASK,  // Success = 2, in bits 8-15
            Ordering::SeqCst
        );
    }
}

// Clone implementation for sharing the bridge
impl Clone for HardwareBridge {
    fn clone(&self) -> Self {
        Self {
            registers: Arc::clone(&self.registers),
            timeout: self.timeout,
        }
    }
}

unsafe impl Send for HardwareBridge {}
unsafe impl Sync for HardwareBridge {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bridge_creation() {
        let bridge = HardwareBridge::new(1000);
        assert!(bridge.check_version().is_ok());
    }
    
    #[test]
    fn test_register_access() {
        let bridge = HardwareBridge::new(1000);
        
        // Test control register
        bridge.registers.set_start(true);
        assert!(bridge.registers.get_control() & CTRL_START_MASK != 0);
        
        bridge.registers.set_start(false);
        assert!(bridge.registers.get_control() & CTRL_START_MASK == 0);
    }
    
    #[test]
    fn test_key_operations() {
        let bridge = HardwareBridge::new(1000);
        let key = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
                  0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
                  0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00,
                  0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        
        bridge.registers.set_key(&key);
        let retrieved_key = bridge.registers.get_key();
        assert_eq!(key, retrieved_key);
    }
    
    #[test]
    fn test_operation_simulation() {
        let bridge = HardwareBridge::new(1000);
        
        // Configure for hash operation
        bridge.registers.set_mode(OperationMode::Hash);
        bridge.registers.set_cost(5);
        
        // Simulate operation
        bridge.simulate_operation(OperationMode::Hash, 100);
        
        // Check completion
        assert!(bridge.registers.is_done());
        assert_eq!(bridge.registers.get_status_code(), StatusCode::Success);
    }
}
