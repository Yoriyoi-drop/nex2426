//! C API bindings for NEX2426
//! 
//! This module provides C-compatible bindings for the NEX2426 encryption engine,
//! enabling integration with C/C++ applications and other language bindings.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_ulonglong};
use std::ptr;
use std::slice;

/// Opaque handle for NEX2426 kernel instance
#[repr(C)]
pub struct NexKernel {
    _private: [u8; 0],
}

/// Result structure for C API operations
#[repr(C)]
#[derive(Clone)]
pub struct NexResult {
    /// Success status (0 = success, non-zero = error)
    pub status: c_int,
    /// Result hash string (null-terminated)
    pub hash: *mut c_char,
    /// Error message (null-terminated, null if no error)
    pub error: *mut c_char,
    /// Timestamp from the operation
    pub timestamp: c_ulonglong,
}

/// Configuration structure for kernel creation
#[repr(C)]
pub struct NexConfig {
    /// Cost parameter (1-10)
    pub cost: c_uint,
    /// Enable temporal binding (0 = false, 1 = true)
    pub temporal_binding: c_int,
    /// Reserved for future use
    pub reserved: [c_uint; 8],
}

impl Default for NexConfig {
    fn default() -> Self {
        Self {
            cost: 3,
            temporal_binding: 0,
            reserved: [0; 8],
        }
    }
}

/// Create a new NEX2426 kernel instance
/// 
/// # Arguments
/// * `config` - Configuration for the kernel
/// 
/// # Returns
/// Pointer to kernel instance, or null on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nex_kernel_create(config: *const NexConfig) -> *mut NexKernel {
    if config.is_null() {
        return ptr::null_mut();
    }
    
    let config = unsafe { &*config };
    let mut kernel = crate::kernel::NexKernel::new(config.cost);
    
    if config.temporal_binding != 0 {
        kernel.enable_temporal_binding();
    }
    
    // Box the kernel to get a stable pointer
    let boxed_kernel = Box::new(kernel);
    Box::into_raw(boxed_kernel) as *mut NexKernel
}

/// Destroy a NEX2426 kernel instance
/// 
/// # Arguments
/// * `kernel` - Pointer to kernel instance
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nex_kernel_destroy(kernel: *mut NexKernel) {
    if !kernel.is_null() {
        unsafe {
            let _ = Box::from_raw(kernel as *mut crate::kernel::NexKernel);
        }
    }
}

/// Hash data using NEX2426 kernel
/// 
/// # Arguments
/// * `kernel` - Pointer to kernel instance
/// * `data` - Pointer to input data
/// * `data_len` - Length of input data in bytes
/// * `key` - Pointer to encryption key (null-terminated string)
/// 
/// # Returns
/// Result structure containing hash or error information
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nex_hash_data(
    kernel: *mut NexKernel,
    data: *const u8,
    data_len: usize,
    key: *const c_char,
) -> NexResult {
    if kernel.is_null() || data.is_null() || key.is_null() {
        return NexResult {
            status: -1,
            hash: ptr::null_mut(),
            error: CString::new("Invalid parameters").expect("Failed to create error string").into_raw(),
            timestamp: 0,
        };
    }
    
    let kernel = unsafe { &*(kernel as *const crate::kernel::NexKernel) };
    let data_slice = unsafe { slice::from_raw_parts(data, data_len) };
    
    let key_str = match unsafe { CStr::from_ptr(key) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            return NexResult {
                status: -2,
                hash: ptr::null_mut(),
                error: CString::new("Invalid key encoding").expect("Failed to create error string").into_raw(),
                timestamp: 0,
            };
        }
    };
    
    let mut cursor = std::io::Cursor::new(data_slice);
    let result = kernel.execute(&mut cursor, key_str);
    
    NexResult {
        status: 0,
        hash: CString::new(result.full_formatted_string).expect("Failed to create hash string").into_raw(),
        error: ptr::null_mut(),
        timestamp: result.timestamp,
    }
}

/// Hash string using NEX2426 kernel
/// 
/// # Arguments
/// * `kernel` - Pointer to kernel instance
/// * `data` - Pointer to input string (null-terminated)
/// * `key` - Pointer to encryption key (null-terminated string)
/// 
/// # Returns
/// Result structure containing hash or error information
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nex_hash_string(
    kernel: *mut NexKernel,
    data: *const c_char,
    key: *const c_char,
) -> NexResult {
    if kernel.is_null() || data.is_null() || key.is_null() {
        return NexResult {
            status: -1,
            hash: ptr::null_mut(),
            error: CString::new("Invalid parameters").expect("Failed to create error string").into_raw(),
            timestamp: 0,
        };
    }
    
    let data_str = match unsafe { CStr::from_ptr(data) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            return NexResult {
                status: -2,
                hash: ptr::null_mut(),
                error: CString::new("Invalid data encoding").expect("Failed to create error string").into_raw(),
                timestamp: 0,
            };
        }
    };
    
    let key_str = match unsafe { CStr::from_ptr(key) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            return NexResult {
                status: -2,
                hash: ptr::null_mut(),
                error: CString::new("Invalid key encoding").expect("Failed to create error string").into_raw(),
                timestamp: 0,
            };
        }
    };
    
    let kernel = unsafe { &*(kernel as *const crate::kernel::NexKernel) };
    let mut cursor = std::io::Cursor::new(data_str);
    let result = kernel.execute(&mut cursor, key_str);
    
    NexResult {
        status: 0,
        hash: CString::new(result.full_formatted_string).expect("Failed to create hash string").into_raw(),
        error: ptr::null_mut(),
        timestamp: result.timestamp,
    }
}

/// Free result structure memory
/// 
/// # Arguments
/// * `result` - Pointer to result structure
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nex_result_free(result: *mut NexResult) {
    if result.is_null() {
        return;
    }
    
    let result = unsafe { &mut *result };
    
    if !result.hash.is_null() {
        unsafe {
            let _ = CString::from_raw(result.hash);
        }
        result.hash = ptr::null_mut();
    }
    
    if !result.error.is_null() {
        unsafe {
            let _ = CString::from_raw(result.error);
        }
        result.error = ptr::null_mut();
    }
}

/// Get library version information
/// 
/// # Returns
/// Pointer to version string (null-terminated)
#[unsafe(no_mangle)]
pub extern "C" fn nex_get_version() -> *const c_char {
    CString::new(env!("CARGO_PKG_VERSION")).expect("Failed to create version string").into_raw()
}

/// Get library build information
/// 
/// # Returns
/// Pointer to build info string (null-terminated)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nex_get_build_info() -> *const c_char {
    let build_info = format!(
        "NEX2426 v{} (Rust {}-{}-{})",
        env!("CARGO_PKG_VERSION"),
        std::env::var("CARGO_PKG_RUST_VERSION").unwrap_or_default(),
        std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default(),
        std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default()
    );
    CString::new(build_info).expect("Failed to create build info string").into_raw()
}

/// Free string allocated by library
/// 
/// # Arguments
/// * `string` - Pointer to string (null-terminated)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nex_free_string(string: *mut c_char) {
    if !string.is_null() {
        let _ = unsafe { CString::from_raw(string) };
    }
}

/// Performance benchmark function
/// 
/// # Arguments
/// * `kernel` - Pointer to kernel instance
/// * `iterations` - Number of iterations to run
/// * `data_size` - Size of test data in bytes
/// 
/// # Returns
/// Average time per iteration in nanoseconds
#[unsafe(no_mangle)]
pub extern "C" fn nex_benchmark(
    kernel: *mut NexKernel,
    iterations: c_uint,
    data_size: usize,
) -> c_ulonglong {
    if kernel.is_null() || iterations == 0 || data_size == 0 {
        return 0;
    }
    
    let kernel = unsafe { &*(kernel as *const crate::kernel::NexKernel) };
    let test_data = vec![b'x'; data_size];
    let test_key = "benchmark_key";
    
    let start = std::time::Instant::now();
    
    for _ in 0..iterations {
        let mut cursor = std::io::Cursor::new(&test_data);
        let _ = kernel.execute(&mut cursor, test_key);
    }
    
    let duration = start.elapsed();
    duration.as_nanos() as c_ulonglong / iterations as c_ulonglong
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_c_api_basic() {
        let config = NexConfig::default();
        let kernel = unsafe { nex_kernel_create(&config) };
        assert!(!kernel.is_null());
        
        let result = unsafe {
            nex_hash_string(
                kernel,
                "test data\0".as_ptr() as *const c_char,
                "test key\0".as_ptr() as *const c_char,
            )
        };
        
        assert_eq!(result.status, 0);
        assert!(!result.hash.is_null());
        assert!(result.error.is_null());
        
        unsafe {
            nex_result_free(&mut result.clone() as *mut NexResult);
            nex_kernel_destroy(kernel);
        }
    }
    
    #[test]
    fn test_c_api_data() {
        let config = NexConfig::default();
        let kernel = unsafe { nex_kernel_create(&config) };
        assert!(!kernel.is_null());
        
        let test_data = b"test data";
        let result = unsafe {
            nex_hash_data(
                kernel,
                test_data.as_ptr(),
                test_data.len(),
                "test key\0".as_ptr() as *const c_char,
            )
        };
        
        assert_eq!(result.status, 0);
        assert!(!result.hash.is_null());
        assert!(result.error.is_null());
        
        unsafe {
            nex_result_free(&mut result.clone() as *mut NexResult);
            nex_kernel_destroy(kernel);
        }
    }
    
    #[test]
    fn test_c_api_error() {
        let result = unsafe {
            nex_hash_string(
                ptr::null_mut(),
                "test data\0".as_ptr() as *const c_char,
                "test key\0".as_ptr() as *const c_char,
            )
        };
        
        assert_eq!(result.status, -1);
        assert!(result.hash.is_null());
        assert!(!result.error.is_null());
        
        unsafe {
            nex_result_free(&mut result.clone() as *mut NexResult);
        }
    }
}
