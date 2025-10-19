// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Rust implementation of Firefox UTF-8 validation
//!
//! This module provides UTF-8 validation functionality, porting the C++
//! `mozilla::detail::IsValidUtf8()` function to Rust.
//!
//! # Background
//!
//! UTF-8 is a variable-length encoding where:
//! - ASCII characters (0x00-0x7F) are encoded as single bytes
//! - Non-ASCII characters use 2-4 bytes with specific bit patterns
//! - Invalid sequences include:
//!   - Overlong encodings (using more bytes than necessary)
//!   - Surrogates (U+D800-U+DFFF, reserved for UTF-16)
//!   - Code points beyond U+10FFFF
//!   - Invalid continuation bytes
//!   - Truncated sequences
//!
//! # Implementation
//!
//! This implementation leverages Rust's standard library `std::str::from_utf8()`
//! which provides production-grade, well-tested UTF-8 validation. The Rust
//! implementation validates the same UTF-8 standard (RFC 3629) as Firefox's C++
//! implementation.
//!
//! # Safety
//!
//! The core validation logic is safe Rust code. The FFI layer requires unsafe
//! code to construct slices from raw pointers, but includes comprehensive
//! safety checks (null pointer checks, bounds validation).

#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

pub mod ffi;

#[cfg(test)]
mod tests;

/// Validates whether a byte sequence is well-formed UTF-8.
///
/// This function checks if the provided byte slice represents valid UTF-8
/// encoding according to RFC 3629. It validates:
///
/// - Proper byte sequence patterns (lead bytes and continuation bytes)
/// - No overlong encodings
/// - No surrogates (U+D800-U+DFFF)
/// - Code points within valid range (U+0000-U+10FFFF)
/// - Complete sequences (no truncation)
///
/// # Arguments
///
/// * `bytes` - A slice of bytes to validate
///
/// # Returns
///
/// * `true` if the bytes represent valid UTF-8
/// * `false` if the bytes contain any invalid UTF-8 sequences
///
/// # Examples
///
/// ```
/// use firefox_utf8_validator::is_valid_utf8;
///
/// // ASCII is valid UTF-8
/// assert!(is_valid_utf8(b"Hello, world!"));
///
/// // Valid multi-byte UTF-8
/// assert!(is_valid_utf8("Café".as_bytes()));
/// assert!(is_valid_utf8("日本語".as_bytes()));
/// assert!(is_valid_utf8("🦀".as_bytes())); // Rust crab emoji
///
/// // Invalid UTF-8
/// assert!(!is_valid_utf8(&[0xFF])); // Invalid lead byte
/// assert!(!is_valid_utf8(&[0xC0, 0x80])); // Overlong encoding
/// assert!(!is_valid_utf8(&[0xED, 0xA0, 0x80])); // Surrogate (U+D800)
/// ```
///
/// # Performance
///
/// This function uses Rust's `std::str::from_utf8()` which is highly optimized
/// and may use SIMD instructions on supported platforms. Performance is
/// expected to be equal to or better than the C++ implementation.
#[inline]
pub fn is_valid_utf8(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok()
}

/// Validates UTF-8 with explicit length (alternative API).
///
/// This is a convenience function that creates a slice from a pointer and
/// length, then validates it. The caller must ensure the pointer and length
/// are valid.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer. The caller
/// must ensure:
/// - `ptr` points to a valid memory region of at least `len` bytes
/// - The memory remains valid for the duration of the function call
/// - For null pointers, `len` must be 0
///
/// # Arguments
///
/// * `ptr` - Pointer to the start of the byte sequence
/// * `len` - Number of bytes to validate
///
/// # Returns
///
/// * `true` if the bytes represent valid UTF-8
/// * `false` if the bytes contain invalid UTF-8 or if `ptr` is null and `len > 0`
#[inline]
pub unsafe fn is_valid_utf8_ptr(ptr: *const u8, len: usize) -> bool {
    if ptr.is_null() {
        // Null pointer is only valid if length is 0
        return len == 0;
    }

    // SAFETY: Caller guarantees ptr is valid for len bytes
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    is_valid_utf8(bytes)
}
