//! FFI (Foreign Function Interface) layer for HashBytes
//!
//! This module provides C-compatible exports for the HashBytes function
//! so that existing C++ code can call the Rust implementation.
//!
//! # Safety
//!
//! All FFI functions are marked `unsafe` because they accept raw pointers
//! from C++. The caller must ensure:
//! - `bytes` pointer is valid and points to at least `length` bytes
//! - `bytes` pointer is not null (unless length is 0)
//! - Memory pointed to by `bytes` remains valid for the duration of the call
//!
//! # Panic Safety
//!
//! FFI functions catch panics to prevent unwinding into C++, which would
//! cause undefined behavior.

use crate::{hash_bytes, HashNumber};
use std::panic;
use std::slice;

/// FFI wrapper for hash_bytes function.
///
/// This function is exported with C linkage so it can be called from C++.
///
/// # Arguments
///
/// * `bytes` - Pointer to byte array (can be null if length is 0)
/// * `length` - Number of bytes to hash
/// * `starting_hash` - Starting hash value for chaining
///
/// # Returns
///
/// 32-bit hash value
///
/// # Safety
///
/// Caller must ensure:
/// - If `length > 0`, `bytes` must be a valid pointer to at least `length` bytes
/// - If `length == 0`, `bytes` can be null or any value (it won't be dereferenced)
/// - The memory pointed to by `bytes` must remain valid for the duration of this call
///
/// # Panics
///
/// If this function panics (which should not happen in normal operation), the panic
/// is caught and a hash of 0 is returned to prevent unwinding into C++.
#[no_mangle]
pub unsafe extern "C" fn mozilla_HashBytes(
    bytes: *const u8,
    length: usize,
    starting_hash: HashNumber,
) -> HashNumber {
    // Catch panics to prevent unwinding into C++
    let result = panic::catch_unwind(|| {
        // Handle null pointer or zero length
        if length == 0 {
            return starting_hash;
        }

        // Verify pointer is not null
        if bytes.is_null() {
            // In C++, this would be undefined behavior, but we handle it gracefully
            return starting_hash;
        }

        // Create slice from raw pointer
        // SAFETY: Caller guarantees that bytes points to at least length bytes
        let slice = unsafe { slice::from_raw_parts(bytes, length) };

        // Call the safe Rust implementation
        hash_bytes(slice, starting_hash)
    });

    // Return the hash, or 0 if we panicked
    result.unwrap_or(0)
}

/// Alternative name for compatibility with different naming conventions.
///
/// Some C++ code may use this name instead of mozilla_HashBytes.
#[no_mangle]
pub unsafe extern "C" fn HashBytes(
    bytes: *const u8,
    length: usize,
    starting_hash: HashNumber,
) -> HashNumber {
    mozilla_HashBytes(bytes, length, starting_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_null_pointer_zero_length() {
        unsafe {
            let hash = mozilla_HashBytes(std::ptr::null(), 0, 0);
            assert_eq!(hash, 0);
        }
    }

    #[test]
    fn test_ffi_null_pointer_with_nonzero_length() {
        unsafe {
            // This should handle null pointer gracefully
            let hash = mozilla_HashBytes(std::ptr::null(), 10, 42);
            assert_eq!(hash, 42); // Returns starting hash
        }
    }

    #[test]
    fn test_ffi_basic_hash() {
        let data = b"hello";
        unsafe {
            let hash = mozilla_HashBytes(data.as_ptr(), data.len(), 0);
            // Just verify it returns a non-zero hash
            assert_ne!(hash, 0);
        }
    }

    #[test]
    fn test_ffi_hash_chaining() {
        let part1 = b"hello";
        let part2 = b" world";

        unsafe {
            let hash1 = mozilla_HashBytes(part1.as_ptr(), part1.len(), 0);
            let hash2 = mozilla_HashBytes(part2.as_ptr(), part2.len(), hash1);

            // Combined hash should be different from individual hashes
            assert_ne!(hash2, hash1);
            assert_ne!(hash2, 0);
        }
    }

    #[test]
    fn test_ffi_matches_safe_implementation() {
        let data = b"test data for hashing";

        let safe_hash = hash_bytes(data, 0);
        let ffi_hash = unsafe { mozilla_HashBytes(data.as_ptr(), data.len(), 0) };

        assert_eq!(safe_hash, ffi_hash, "FFI wrapper should match safe implementation");
    }

    #[test]
    fn test_ffi_alternative_name() {
        let data = b"test";
        unsafe {
            let hash1 = mozilla_HashBytes(data.as_ptr(), data.len(), 0);
            let hash2 = HashBytes(data.as_ptr(), data.len(), 0);
            assert_eq!(hash1, hash2, "Both function names should produce same result");
        }
    }
}
