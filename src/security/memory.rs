use std::ptr;
use std::sync::atomic::{compiler_fence, Ordering};

/// Trait for types that can be securely zeroized.
pub trait Zeroize {
    fn zeroize(&mut self);
}

/// Securely clears a byte slice from memory.
/// Uses volatile writes to prevent compiler optimization.
pub fn secure_clean(data: &mut [u8]) {
    unsafe {
        for byte in data {
            ptr::write_volatile(byte, 0);
        }
    }
    compiler_fence(Ordering::SeqCst);
}

/// A wrapper for sensitive data that auto-zeroizes on drop.
pub struct Protected<T: Zeroize> {
    inner: Option<T>,
}

/// Error type for Protected data access
#[derive(Debug, Clone)]
pub enum ProtectedError {
    DataDropped,
}

impl std::fmt::Display for ProtectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtectedError::DataDropped => write!(f, "Attempted to access protected data after it was dropped"),
        }
    }
}

impl std::error::Error for ProtectedError {}

impl<T: Zeroize> Protected<T> {
    pub fn new(data: T) -> Self {
        Self { inner: Some(data) }
    }

    pub fn access(&self) -> Result<&T, ProtectedError> {
        self.inner.as_ref().ok_or(ProtectedError::DataDropped)
    }

    pub fn access_mut(&mut self) -> Result<&mut T, ProtectedError> {
        self.inner.as_mut().ok_or(ProtectedError::DataDropped)
    }
}

impl<T: Zeroize> Drop for Protected<T> {
    fn drop(&mut self) {
        if let Some(ref mut inner) = self.inner {
            inner.zeroize();
        }
    }
}

// Implementations for common types

impl Zeroize for Vec<u8> {
    fn zeroize(&mut self) {
        secure_clean(self);
    }
}

impl Zeroize for [u8; 32] {
    fn zeroize(&mut self) {
        secure_clean(self);
    }
}

impl Zeroize for [u8; 64] {
    fn zeroize(&mut self) {
        secure_clean(self);
    }
}

impl Zeroize for String {
    fn zeroize(&mut self) {
        unsafe {
            let vec = self.as_mut_vec();
            secure_clean(vec);
        }
    }
}

// SAFE implementations for numeric arrays using element-wise clearing
impl Zeroize for [i64; 32] {
    fn zeroize(&mut self) {
        // SAFE: Clear each element individually to avoid unsafe casting
        for element in self.iter_mut() {
            *element = 0;
        }
        // Additional memory barrier
        compiler_fence(Ordering::SeqCst);
    }
}

impl Zeroize for Vec<i64> {
    fn zeroize(&mut self) {
        // SAFE: Clear each element individually
        for element in self.iter_mut() {
            *element = 0;
        }
        // Additional memory barrier
        compiler_fence(Ordering::SeqCst);
    }
}

impl Zeroize for Vec<u64> {
    fn zeroize(&mut self) {
        // SAFE: Clear each element individually  
        for element in self.iter_mut() {
            *element = 0;
        }
        // Additional memory barrier
        compiler_fence(Ordering::SeqCst);
    }
}
