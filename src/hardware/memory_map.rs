//==============================================================================
// NEX2426 Hardware Memory Map
// Memory-mapped register definitions for hardware interface
//==============================================================================

use std::sync::atomic::{AtomicU64, Ordering};

// Register addresses (8-bit register map)
pub const REG_CONTROL: u8 = 0x00;
pub const REG_STATUS: u8 = 0x01;
pub const REG_MODE: u8 = 0x02;
pub const REG_KEY0: u8 = 0x10;
pub const REG_KEY1: u8 = 0x11;
pub const REG_KEY2: u8 = 0x12;
pub const REG_KEY3: u8 = 0x13;
pub const REG_COST: u8 = 0x20;
pub const REG_CONFIG: u8 = 0x21;
pub const REG_HW_ID: u8 = 0x22;
pub const REG_DATA_IN: u8 = 0x30;
pub const REG_DATA_OUT: u8 = 0x31;
pub const REG_HASH0: u8 = 0x40;
pub const REG_HASH1: u8 = 0x41;
pub const REG_HASH2: u8 = 0x42;
pub const REG_HASH3: u8 = 0x43;
pub const REG_HASH4: u8 = 0x44;
pub const REG_HASH5: u8 = 0x45;
pub const REG_HASH6: u8 = 0x46;
pub const REG_HASH7: u8 = 0x47;
pub const REG_BRIDGE_STATUS: u8 = 0xFE;
pub const REG_VERSION: u8 = 0xFF;

// Control register bits
pub const CTRL_START_BIT: u64 = 0;
pub const CTRL_START_MASK: u64 = 1 << CTRL_START_BIT;

// Status register bits
pub const STATUS_DONE_BIT: u64 = 0;
pub const STATUS_DONE_MASK: u64 = 1 << STATUS_DONE_BIT;
pub const STATUS_ERROR_SHIFT: u64 = 1;
pub const STATUS_ERROR_MASK: u64 = 0xF << STATUS_ERROR_SHIFT;

// Config register bits
pub const CONFIG_BIO_LOCK_BIT: u64 = 0;
pub const CONFIG_BIO_LOCK_MASK: u64 = 1 << CONFIG_BIO_LOCK_BIT;
pub const CONFIG_STEALTH_BIT: u64 = 1;
pub const CONFIG_STEALTH_MASK: u64 = 1 << CONFIG_STEALTH_BIT;

// Operation modes (matching SystemVerilog enum)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationMode {
    Encrypt = 1,
    Decrypt = 2,
    Hash = 3,
    Benchmark = 4,
    KeyGen = 5,
    Stealth = 6,
    BioLock = 7,
}

impl From<u8> for OperationMode {
    fn from(value: u8) -> Self {
        match value {
            1 => OperationMode::Encrypt,
            2 => OperationMode::Decrypt,
            3 => OperationMode::Hash,
            4 => OperationMode::Benchmark,
            5 => OperationMode::KeyGen,
            6 => OperationMode::Stealth,
            7 => OperationMode::BioLock,
            _ => OperationMode::Hash, // Default
        }
    }
}

impl From<OperationMode> for u8 {
    fn from(mode: OperationMode) -> Self {
        mode as u8
    }
}

// Status codes (matching SystemVerilog enum)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusCode {
    Idle = 0,
    Busy = 1,
    Success = 2,
    ErrorKey = 4,
    ErrorCost = 5,
    ErrorMode = 6,
    ErrorMemory = 7,
    ErrorCrypto = 8,
}

impl From<u8> for StatusCode {
    fn from(value: u8) -> Self {
        match value {
            0 => StatusCode::Idle,
            1 => StatusCode::Busy,
            2 => StatusCode::Success,
            4 => StatusCode::ErrorKey,
            5 => StatusCode::ErrorCost,
            6 => StatusCode::ErrorMode,
            7 => StatusCode::ErrorMemory,
            8 => StatusCode::ErrorCrypto,
            _ => StatusCode::Idle,
        }
    }
}

// Memory-mapped register structure
#[repr(C)]
pub struct HardwareRegisters {
    // Control and status
    pub control: AtomicU64,
    pub status: AtomicU64,
    pub mode: AtomicU64,
    
    // Key registers (256-bit key split into 4x64-bit registers)
    pub key0: AtomicU64,
    pub key1: AtomicU64,
    pub key2: AtomicU64,
    pub key3: AtomicU64,
    
    // Configuration
    pub cost: AtomicU64,
    pub config: AtomicU64,
    pub hw_id: AtomicU64,
    
    // Data I/O
    pub data_in: AtomicU64,
    pub data_out: AtomicU64,
    
    // Hash output (512-bit hash split into 8x64-bit registers)
    pub hash0: AtomicU64,
    pub hash1: AtomicU64,
    pub hash2: AtomicU64,
    pub hash3: AtomicU64,
    pub hash4: AtomicU64,
    pub hash5: AtomicU64,
    pub hash6: AtomicU64,
    pub hash7: AtomicU64,
    
    // Bridge status and version
    pub bridge_status: AtomicU64,
    pub version: AtomicU64,
}

impl Default for HardwareRegisters {
    fn default() -> Self {
        Self {
            control: AtomicU64::new(0),
            status: AtomicU64::new(0),
            mode: AtomicU64::new(0),
            key0: AtomicU64::new(0),
            key1: AtomicU64::new(0),
            key2: AtomicU64::new(0),
            key3: AtomicU64::new(0),
            cost: AtomicU64::new(1),
            config: AtomicU64::new(0),
            hw_id: AtomicU64::new(0),
            data_in: AtomicU64::new(0),
            data_out: AtomicU64::new(0),
            hash0: AtomicU64::new(0),
            hash1: AtomicU64::new(0),
            hash2: AtomicU64::new(0),
            hash3: AtomicU64::new(0),
            hash4: AtomicU64::new(0),
            hash5: AtomicU64::new(0),
            hash6: AtomicU64::new(0),
            hash7: AtomicU64::new(0),
            bridge_status: AtomicU64::new(0),
            version: AtomicU64::new(0x4E45583200060000), // "NEX2" version 6.0.0
        }
    }
}

// Helper functions for register access
impl HardwareRegisters {
    pub fn set_control(&self, value: u64) {
        self.control.store(value, Ordering::SeqCst);
    }
    
    pub fn get_control(&self) -> u64 {
        self.control.load(Ordering::SeqCst)
    }
    
    pub fn set_start(&self, start: bool) {
        let current = self.control.load(Ordering::SeqCst);
        let new = if start { current | CTRL_START_MASK } else { current & !CTRL_START_MASK };
        self.control.store(new, Ordering::SeqCst);
    }
    
    pub fn get_status(&self) -> u64 {
        self.status.load(Ordering::SeqCst)
    }
    
    pub fn is_done(&self) -> bool {
        (self.status.load(Ordering::SeqCst) & STATUS_DONE_MASK) != 0
    }
    
    pub fn get_status_code(&self) -> StatusCode {
        let status = self.status.load(Ordering::SeqCst);
        // Check main status first (bits 8-15), then error bits
        let main_status = (status >> 8) as u8;
        if main_status == 0 {
            StatusCode::Idle
        } else if main_status == 1 {
            StatusCode::Busy
        } else if main_status == 2 {
            StatusCode::Success
        } else {
            // Error codes in bits 1-4
            StatusCode::from(((status & STATUS_ERROR_MASK) >> STATUS_ERROR_SHIFT) as u8)
        }
    }
    
    pub fn set_mode(&self, mode: OperationMode) {
        self.mode.store(mode as u64, Ordering::SeqCst);
    }
    
    pub fn get_mode(&self) -> OperationMode {
        OperationMode::from(self.mode.load(Ordering::SeqCst) as u8)
    }
    
    pub fn set_key(&self, key: &[u8; 32]) {
        // Convert 32-byte key to 4x64-bit words
        let key0 = u64::from_le_bytes([key[0], key[1], key[2], key[3], key[4], key[5], key[6], key[7]]);
        let key1 = u64::from_le_bytes([key[8], key[9], key[10], key[11], key[12], key[13], key[14], key[15]]);
        let key2 = u64::from_le_bytes([key[16], key[17], key[18], key[19], key[20], key[21], key[22], key[23]]);
        let key3 = u64::from_le_bytes([key[24], key[25], key[26], key[27], key[28], key[29], key[30], key[31]]);
        
        self.key0.store(key0, Ordering::SeqCst);
        self.key1.store(key1, Ordering::SeqCst);
        self.key2.store(key2, Ordering::SeqCst);
        self.key3.store(key3, Ordering::SeqCst);
    }
    
    pub fn get_key(&self) -> [u8; 32] {
        let mut key = [0u8; 32];
        
        let key0 = self.key0.load(Ordering::SeqCst).to_le_bytes();
        let key1 = self.key1.load(Ordering::SeqCst).to_le_bytes();
        let key2 = self.key2.load(Ordering::SeqCst).to_le_bytes();
        let key3 = self.key3.load(Ordering::SeqCst).to_le_bytes();
        
        key[0..8].copy_from_slice(&key0);
        key[8..16].copy_from_slice(&key1);
        key[16..24].copy_from_slice(&key2);
        key[24..32].copy_from_slice(&key3);
        
        key
    }
    
    pub fn set_cost(&self, cost: u32) {
        self.cost.store(cost as u64, Ordering::SeqCst);
    }
    
    pub fn get_cost(&self) -> u32 {
        self.cost.load(Ordering::SeqCst) as u32
    }
    
    pub fn set_config(&self, bio_lock: bool, stealth: bool) {
        let mut config = 0u64;
        if bio_lock { config |= CONFIG_BIO_LOCK_MASK; }
        if stealth { config |= CONFIG_STEALTH_MASK; }
        self.config.store(config, Ordering::SeqCst);
    }
    
    pub fn get_config(&self) -> (bool, bool) {
        let config = self.config.load(Ordering::SeqCst);
        let bio_lock = (config & CONFIG_BIO_LOCK_MASK) != 0;
        let stealth = (config & CONFIG_STEALTH_MASK) != 0;
        (bio_lock, stealth)
    }
    
    pub fn set_hw_id(&self, hw_id: u64) {
        self.hw_id.store(hw_id, Ordering::SeqCst);
    }
    
    pub fn get_hw_id(&self) -> u64 {
        self.hw_id.load(Ordering::SeqCst)
    }
    
    pub fn set_data_in(&self, data: u8) {
        self.data_in.store(data as u64, Ordering::SeqCst);
    }
    
    pub fn get_data_out(&self) -> u8 {
        self.data_out.load(Ordering::SeqCst) as u8
    }
    
    pub fn get_hash(&self) -> [u8; 64] {
        let mut hash = [0u8; 64];
        
        let hash0 = self.hash0.load(Ordering::SeqCst).to_le_bytes();
        let hash1 = self.hash1.load(Ordering::SeqCst).to_le_bytes();
        let hash2 = self.hash2.load(Ordering::SeqCst).to_le_bytes();
        let hash3 = self.hash3.load(Ordering::SeqCst).to_le_bytes();
        let hash4 = self.hash4.load(Ordering::SeqCst).to_le_bytes();
        let hash5 = self.hash5.load(Ordering::SeqCst).to_le_bytes();
        let hash6 = self.hash6.load(Ordering::SeqCst).to_le_bytes();
        let hash7 = self.hash7.load(Ordering::SeqCst).to_le_bytes();
        
        hash[0..8].copy_from_slice(&hash0);
        hash[8..16].copy_from_slice(&hash1);
        hash[16..24].copy_from_slice(&hash2);
        hash[24..32].copy_from_slice(&hash3);
        hash[32..40].copy_from_slice(&hash4);
        hash[40..48].copy_from_slice(&hash5);
        hash[48..56].copy_from_slice(&hash6);
        hash[56..64].copy_from_slice(&hash7);
        
        hash
    }
    
    pub fn get_version(&self) -> (u32, u32) {
        let version = self.version.load(Ordering::SeqCst);
        let major = (version >> 48) as u32;
        let minor = ((version >> 32) & 0xFFFF) as u32;
        (major, minor)
    }
}
