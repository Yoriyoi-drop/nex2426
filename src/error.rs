//! Comprehensive error handling for NEX2426
//! 
//! Provides centralized error types for all modules to ensure consistent error handling
//! and better debugging capabilities.

use thiserror::Error;

/// Main error type for NEX2426 operations
#[derive(Debug, Error)]
pub enum NexError {
    /// Configuration related errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Cryptographic operation errors
    #[error("Cryptographic error: {0}")]
    Crypto(String),
    
    /// Input validation errors
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// Memory allocation errors
    #[error("Memory error: {0}")]
    Memory(String),
    
    /// I/O related errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Performance or resource errors
    #[error("Resource error: {0}")]
    Resource(String),
    
    /// Threading or concurrency errors
    #[error("Concurrency error: {0}")]
    Concurrency(String),
    
    /// Validation errors
    #[error("Validation failed: {0}")]
    Validation(String),
    
    /// Protocol specific errors
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    /// Hardware related errors
    #[error("Hardware error: {0}")]
    Hardware(String),
    
    /// Generic error for unexpected cases
    #[error("Internal error: {0}")]
    Internal(String),
}

impl NexError {
    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }
    
    /// Create a cryptographic error
    pub fn crypto(msg: impl Into<String>) -> Self {
        Self::Crypto(msg.into())
    }
    
    /// Create an invalid input error
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }
    
    /// Create a memory error
    pub fn memory(msg: impl Into<String>) -> Self {
        Self::Memory(msg.into())
    }
    
    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
    
    /// Create a protocol error
    pub fn protocol(msg: impl Into<String>) -> Self {
        Self::Protocol(msg.into())
    }
    
    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

impl From<serde_json::Error> for NexError {
    fn from(err: serde_json::Error) -> Self {
        NexError::Serialization(err.to_string())
    }
}

impl From<crate::security::memory::ProtectedError> for NexError {
    fn from(err: crate::security::memory::ProtectedError) -> Self {
        NexError::Crypto(err.to_string())
    }
}

impl From<crate::standards::kdf::KdfError> for NexError {
    fn from(err: crate::standards::kdf::KdfError) -> Self {
        NexError::Crypto(err.to_string())
    }
}

/// Result type alias for NEX2426 operations
pub type NexResult<T> = Result<T, NexError>;

/// Macro for convenient error creation with context
#[macro_export]
macro_rules! nex_err {
    (config, $msg:expr) => {
        $crate::error::NexError::config($msg)
    };
    (crypto, $msg:expr) => {
        $crate::error::NexError::crypto($msg)
    };
    (invalid_input, $msg:expr) => {
        $crate::error::NexError::invalid_input($msg)
    };
    (memory, $msg:expr) => {
        $crate::error::NexError::memory($msg)
    };
    (validation, $msg:expr) => {
        $crate::error::NexError::validation($msg)
    };
    (protocol, $msg:expr) => {
        $crate::error::NexError::protocol($msg)
    };
    (internal, $msg:expr) => {
        $crate::error::NexError::internal($msg)
    };
}

/// Convenience macros for common error patterns
#[macro_export]
macro_rules! ensure {
    ($condition:expr, $error_type:ident, $msg:expr) => {
        if !$condition {
            return Err($crate::nex_err!($error_type, $msg))
        }
    };
    ($condition:expr, $error_type:ident, $msg:expr, $($arg:tt)*) => {
        if !$condition {
            return Err($crate::nex_err!($error_type, format!($msg, $($arg)*)))
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = NexError::config("test message");
        assert!(matches!(err, NexError::Config(_)));
        
        let err = NexError::crypto("crypto failed");
        assert!(matches!(err, NexError::Crypto(_)));
    }

    #[test]
    fn test_error_macros() {
        let result: NexResult<()> = Err(nex_err!(validation, "test validation"));
        assert!(result.is_err());
        
        let value = ensure!(true, validation, "should not fail", 42);
        assert_eq!(value, 42);
    }
}
