//==============================================================================
// NEX2426 Hardware Interface Module
// Rust interface for SystemVerilog hardware acceleration
//==============================================================================

pub mod bridge;
pub mod accelerator;
pub mod memory_map;

pub use bridge::HardwareBridge;
pub use accelerator::HardwareAccelerator;

#[derive(Debug, Clone, Copy)]
pub enum HardwareError {
    BridgeError,
    AcceleratorError,
    Timeout,
    InvalidOperation,
    HardwareFault,
}

impl std::fmt::Display for HardwareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HardwareError::BridgeError => write!(f, "Hardware bridge communication error"),
            HardwareError::AcceleratorError => write!(f, "Hardware accelerator error"),
            HardwareError::Timeout => write!(f, "Hardware operation timeout"),
            HardwareError::InvalidOperation => write!(f, "Invalid hardware operation"),
            HardwareError::HardwareFault => write!(f, "Hardware fault detected"),
        }
    }
}

impl std::error::Error for HardwareError {}

pub type HardwareResult<T> = Result<T, HardwareError>;
