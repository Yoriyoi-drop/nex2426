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
        for i in 0..data.len() {
            ptr::write_volatile(&mut data[i], 0);
        }
    }
    compiler_fence(Ordering::SeqCst);
}

/// A wrapper for sensitive data that auto-zeroizes on drop.
pub struct Protected<T: Zeroize> {
    inner: Option<T>,
}

impl<T: Zeroize> Protected<T> {
    pub fn new(data: T) -> Self {
        Self { inner: Some(data) }
    }

    pub fn access(&self) -> &T {
        self.inner.as_ref().unwrap()
    }

    pub fn access_mut(&mut self) -> &mut T {
        self.inner.as_mut().unwrap()
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

// New implementations for Signature Keys
impl Zeroize for [i64; 32] {
    fn zeroize(&mut self) {
        unsafe {
             let ptr = self.as_mut_ptr() as *mut u8;
             let len = self.len() * 8; // i64 is 8 bytes
             let slice = std::slice::from_raw_parts_mut(ptr, len);
             secure_clean(slice);
        }
    }
}

impl Zeroize for Vec<i64> {
     fn zeroize(&mut self) {
        unsafe {
             let ptr = self.as_mut_ptr() as *mut u8;
             let len = self.len() * 8; 
             let slice = std::slice::from_raw_parts_mut(ptr, len);
             secure_clean(slice);
        }
    }
}
