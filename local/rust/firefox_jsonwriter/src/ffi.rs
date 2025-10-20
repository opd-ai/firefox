//! FFI layer for JSONWriter gTwoCharEscapes lookup table
//!
//! This module provides C++-compatible FFI exports for the JSON escape lookup table.
//! The table is accessed by C++ code in `mfbt/JSONWriter.h` for JSON string escaping.
//!
//! # C++ Integration
//!
//! The table is exported with C linkage as `mozilla_detail_gTwoCharEscapes`, which
//! C++ code accesses via:
//!
//! ```cpp
//! extern "C" const char mozilla_detail_gTwoCharEscapes[256];
//! ```
//!
//! Or with C++ namespace (via cbindgen):
//!
//! ```cpp
//! namespace mozilla {
//! namespace detail {
//!   extern const char gTwoCharEscapes[256];
//! }
//! }
//! ```
//!
//! # Memory Safety
//!
//! - The table is const/immutable - read-only access from C++
//! - Static lifetime - lives for program duration
//! - No allocation/deallocation - embedded in binary
//! - Thread-safe - no mutable state, no synchronization needed
//!
//! # FFI Safety
//!
//! - Uses `#[no_mangle]` for predictable symbol name
//! - Uses `#[repr(C)]` indirectly via `[i8; 256]` which is C-compatible
//! - Byte-for-byte identical layout to C++ `const char[256]`
//! - No Rust-specific types cross FFI boundary

use crate::TWO_CHAR_ESCAPES;

/// FFI export of the gTwoCharEscapes table for C++ consumption
///
/// This symbol is accessed from C++ code in mfbt/JSONWriter.h as:
/// ```cpp
/// extern "C" const char mozilla_detail_gTwoCharEscapes[256];
/// ```
///
/// # Safety
///
/// This is safe for FFI because:
/// - The table is const (immutable)
/// - It has static lifetime (never freed)
/// - It's a simple array of i8/char (C-compatible)
/// - No pointers, no complex types, no Rust-specific constructs
///
/// # Thread Safety
///
/// Safe for concurrent access - const data, no synchronization needed.
#[no_mangle]
#[used]
pub static mozilla_detail_gTwoCharEscapes: [i8; 256] = TWO_CHAR_ESCAPES;

/// Alternative C++ namespace-compatible export (for cbindgen)
///
/// cbindgen may generate code that expects the table in the mozilla::detail namespace.
/// This ensures both naming conventions work.
#[no_mangle]
#[used]
pub static gTwoCharEscapes: [i8; 256] = TWO_CHAR_ESCAPES;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_symbol_exists() {
        // Verify the FFI symbol is correctly defined
        assert_eq!(mozilla_detail_gTwoCharEscapes.len(), 256);
        assert_eq!(gTwoCharEscapes.len(), 256);
    }

    #[test]
    fn test_ffi_table_identity() {
        // Verify both FFI exports point to same data
        for i in 0..256 {
            assert_eq!(
                mozilla_detail_gTwoCharEscapes[i],
                gTwoCharEscapes[i],
                "FFI exports should have identical data at index {}", i
            );
        }
    }

    #[test]
    fn test_ffi_table_matches_source() {
        // Verify FFI exports match the source table
        for i in 0..256 {
            assert_eq!(
                mozilla_detail_gTwoCharEscapes[i],
                TWO_CHAR_ESCAPES[i],
                "FFI export should match source table at index {}", i
            );
        }
    }

    #[test]
    fn test_ffi_memory_layout() {
        // Verify the FFI table has correct size and alignment
        assert_eq!(
            std::mem::size_of_val(&mozilla_detail_gTwoCharEscapes),
            256,
            "FFI table should be exactly 256 bytes"
        );
        
        // i8 has alignment of 1
        assert_eq!(
            std::mem::align_of_val(&mozilla_detail_gTwoCharEscapes),
            1,
            "FFI table should have 1-byte alignment"
        );
    }

    #[test]
    fn test_ffi_static_lifetime() {
        // Verify the symbols are static (this compiles only if they have 'static lifetime)
        let _ptr: &'static [i8; 256] = &mozilla_detail_gTwoCharEscapes;
        let _ptr2: &'static [i8; 256] = &gTwoCharEscapes;
    }

    #[test]
    fn test_ffi_escape_values() {
        // Test key escape values through FFI exports
        assert_eq!(mozilla_detail_gTwoCharEscapes[0x08], b'b' as i8);  // \b
        assert_eq!(mozilla_detail_gTwoCharEscapes[0x09], b't' as i8);  // \t
        assert_eq!(mozilla_detail_gTwoCharEscapes[0x0A], b'n' as i8);  // \n
        assert_eq!(mozilla_detail_gTwoCharEscapes[0x0C], b'f' as i8);  // \f
        assert_eq!(mozilla_detail_gTwoCharEscapes[0x0D], b'r' as i8);  // \r
        assert_eq!(mozilla_detail_gTwoCharEscapes[0x22], b'"' as i8);  // \"
        assert_eq!(mozilla_detail_gTwoCharEscapes[0x5C], b'\\' as i8); // \\
    }

    #[test]
    fn test_ffi_usage_simulation() {
        // Simulate C++ usage pattern
        unsafe fn check_escape(ch: u8) -> Option<char> {
            let escape_val = mozilla_detail_gTwoCharEscapes[ch as usize];
            if escape_val != 0 {
                Some(escape_val as u8 as char)
            } else {
                None
            }
        }

        // Test the actual usage pattern from JSONWriter.h
        assert_eq!(unsafe { check_escape(b'\t') }, Some('t'));
        assert_eq!(unsafe { check_escape(b'\n') }, Some('n'));
        assert_eq!(unsafe { check_escape(b'"') }, Some('"'));
        assert_eq!(unsafe { check_escape(b'\\') }, Some('\\'));
        assert_eq!(unsafe { check_escape(b'a') }, None);  // Regular char
        assert_eq!(unsafe { check_escape(0x0B) }, None);  // VT - needs \uXXXX
    }
}
