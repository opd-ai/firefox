// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! FFI (Foreign Function Interface) layer for C++ interop
//!
//! This module provides C-compatible exports that can be called from Firefox's
//! C++ code. The FFI layer handles:
//! - C-compatible function signatures
//! - Null pointer checks
//! - Safe conversion from raw pointers to Rust slices
//! - Panic boundaries (preventing unwinding into C++)
//!
//! # Safety Model
//!
//! The FFI functions are unsafe by nature (accepting raw pointers from C++),
//! but implement defense-in-depth safety measures:
//! 1. Explicit null pointer checks
//! 2. Zero-length handling
//! 3. Panic catching (although from_utf8 should never panic)
//! 4. Clear documentation of safety requirements

use std::panic;

/// FFI export: Validates UTF-8 byte sequence (C++ interop)
///
/// This function is exported with C linkage and can be called from C++ code.
/// It provides the same behavior as `mozilla::detail::IsValidUtf8()`.
///
/// # Safety
///
/// This function is unsafe because it accepts raw pointers from C++. The caller
/// (C++ code) must ensure:
/// - `a_code_units` points to a valid memory region of at least `a_count` bytes
/// - The memory remains valid for the duration of the call
/// - If `a_code_units` is null, `a_count` must be 0
///
/// # Arguments
///
/// * `a_code_units` - Pointer to the byte sequence to validate (can be null if count is 0)
/// * `a_count` - Number of bytes in the sequence
///
/// # Returns
///
/// * `true` if the byte sequence is valid UTF-8
/// * `false` if the sequence contains invalid UTF-8
/// * `true` if `a_count` is 0 (empty string is valid UTF-8)
/// * `false` if `a_code_units` is null and `a_count > 0`
///
/// # C++ Signature
///
/// ```cpp
/// extern "C" bool IsValidUtf8_RUST(const uint8_t* a_code_units, size_t a_count);
/// ```
///
/// # Examples (from C++)
///
/// ```cpp
/// // Valid UTF-8
/// const char* valid = "Hello, 世界";
/// bool result = IsValidUtf8_RUST(
///     reinterpret_cast<const uint8_t*>(valid),
///     strlen(valid)
/// ); // returns true
///
/// // Invalid UTF-8
/// uint8_t invalid[] = {0xFF, 0xFE};
/// result = IsValidUtf8_RUST(invalid, 2); // returns false
///
/// // Empty string
/// result = IsValidUtf8_RUST(nullptr, 0); // returns true
/// ```
#[no_mangle]
pub unsafe extern "C" fn IsValidUtf8_RUST(a_code_units: *const u8, a_count: usize) -> bool {
    // Catch panics to prevent unwinding into C++ (defense in depth)
    // Note: std::str::from_utf8 should never panic, but this provides
    // an extra safety layer for the FFI boundary
    let result = panic::catch_unwind(|| {
        // Null pointer check
        if a_code_units.is_null() {
            // Empty string (null with count 0) is valid UTF-8
            return a_count == 0;
        }

        // Zero-length check (optimization)
        if a_count == 0 {
            return true; // Empty string is valid UTF-8
        }

        // SAFETY: Caller (C++ code) guarantees that a_code_units points to
        // valid memory of at least a_count bytes. We've already checked for
        // null pointer above.
        let bytes = unsafe { std::slice::from_raw_parts(a_code_units, a_count) };

        // Validate UTF-8 using Rust stdlib
        // This is the core validation logic, equivalent to C++'s
        // DecodeOneUtf8CodePoint loop
        crate::is_valid_utf8(bytes)
    });

    // If panic occurred (shouldn't happen), return false as safe default
    result.unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_null_pointer_zero_length() {
        // Null pointer with zero length should return true (empty string)
        unsafe {
            assert!(IsValidUtf8_RUST(std::ptr::null(), 0));
        }
    }

    #[test]
    fn test_ffi_null_pointer_nonzero_length() {
        // Null pointer with non-zero length should return false
        unsafe {
            assert!(!IsValidUtf8_RUST(std::ptr::null(), 10));
        }
    }

    #[test]
    fn test_ffi_empty_slice() {
        // Empty slice (zero length) should return true
        let empty: [u8; 0] = [];
        unsafe {
            assert!(IsValidUtf8_RUST(empty.as_ptr(), 0));
        }
    }

    #[test]
    fn test_ffi_valid_ascii() {
        let data = b"Hello, world!";
        unsafe {
            assert!(IsValidUtf8_RUST(data.as_ptr(), data.len()));
        }
    }

    #[test]
    fn test_ffi_valid_multibyte() {
        let data = "Café ☕ 日本語 🦀".as_bytes();
        unsafe {
            assert!(IsValidUtf8_RUST(data.as_ptr(), data.len()));
        }
    }

    #[test]
    fn test_ffi_invalid_lead_byte() {
        // 0xFF is an invalid UTF-8 lead byte
        let data: [u8; 1] = [0xFF];
        unsafe {
            assert!(!IsValidUtf8_RUST(data.as_ptr(), data.len()));
        }
    }

    #[test]
    fn test_ffi_invalid_surrogate() {
        // U+D800 (surrogate) encoded as UTF-8: ED A0 80
        let data: [u8; 3] = [0xED, 0xA0, 0x80];
        unsafe {
            assert!(!IsValidUtf8_RUST(data.as_ptr(), data.len()));
        }
    }

    #[test]
    fn test_ffi_overlong_encoding() {
        // Overlong encoding of 'A' (should be 0x41, not C0 81)
        let data: [u8; 2] = [0xC0, 0x81];
        unsafe {
            assert!(!IsValidUtf8_RUST(data.as_ptr(), data.len()));
        }
    }

    #[test]
    fn test_ffi_truncated_sequence() {
        // Truncated 2-byte sequence (missing continuation byte)
        let data: [u8; 1] = [0xC3]; // Should be followed by continuation byte
        unsafe {
            assert!(!IsValidUtf8_RUST(data.as_ptr(), data.len()));
        }
    }

    #[test]
    fn test_ffi_max_codepoint() {
        // U+10FFFF (max valid Unicode codepoint): F4 8F BF BF
        let data: [u8; 4] = [0xF4, 0x8F, 0xBF, 0xBF];
        unsafe {
            assert!(IsValidUtf8_RUST(data.as_ptr(), data.len()));
        }
    }

    #[test]
    fn test_ffi_beyond_max_codepoint() {
        // U+110000 (beyond max Unicode): F4 90 80 80
        let data: [u8; 4] = [0xF4, 0x90, 0x80, 0x80];
        unsafe {
            assert!(!IsValidUtf8_RUST(data.as_ptr(), data.len()));
        }
    }
}
