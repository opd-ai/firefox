/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! FFI layer for C++ interoperability
//!
//! This module exports the ASCII masks to C++ code with a stable ABI.
//! The C++ code can call these functions to get pointers to the static mask arrays.
//!
//! ## C++ Usage
//! ```cpp
//! extern "C" {
//!   const ASCIIMaskArray* ASCIIMask_MaskWhitespace();
//!   const ASCIIMaskArray* ASCIIMask_MaskCRLF();
//!   const ASCIIMaskArray* ASCIIMask_MaskCRLFTab();
//!   const ASCIIMaskArray* ASCIIMask_Mask0to9();
//! }
//!
//! const ASCIIMaskArray& ASCIIMask::MaskWhitespace() {
//!   return *ASCIIMask_MaskWhitespace();
//! }
//! ```

use crate::{ASCIIMaskArray, WHITESPACE_MASK, CRLF_MASK, CRLF_TAB_MASK, ZERO_TO_NINE_MASK};

// ============================================================================
// FFI Exports (C ABI)
// ============================================================================

/// FFI: Get pointer to whitespace mask (\f, \t, \r, \n, space)
///
/// # Safety
/// Returns a pointer to static data with 'static lifetime.
/// Safe to call from C++. The returned pointer is never null.
///
/// # C++ Signature
/// ```cpp
/// extern "C" const ASCIIMaskArray* ASCIIMask_MaskWhitespace();
/// ```
#[no_mangle]
pub extern "C" fn ASCIIMask_MaskWhitespace() -> *const ASCIIMaskArray {
    &WHITESPACE_MASK as *const ASCIIMaskArray
}

/// FFI: Get pointer to CRLF mask (\r, \n)
///
/// # Safety
/// Returns a pointer to static data with 'static lifetime.
/// Safe to call from C++. The returned pointer is never null.
///
/// # C++ Signature
/// ```cpp
/// extern "C" const ASCIIMaskArray* ASCIIMask_MaskCRLF();
/// ```
#[no_mangle]
pub extern "C" fn ASCIIMask_MaskCRLF() -> *const ASCIIMaskArray {
    &CRLF_MASK as *const ASCIIMaskArray
}

/// FFI: Get pointer to CRLF+Tab mask (\r, \n, \t)
///
/// # Safety
/// Returns a pointer to static data with 'static lifetime.
/// Safe to call from C++. The returned pointer is never null.
///
/// # C++ Signature
/// ```cpp
/// extern "C" const ASCIIMaskArray* ASCIIMask_MaskCRLFTab();
/// ```
#[no_mangle]
pub extern "C" fn ASCIIMask_MaskCRLFTab() -> *const ASCIIMaskArray {
    &CRLF_TAB_MASK as *const ASCIIMaskArray
}

/// FFI: Get pointer to digit mask (0-9)
///
/// # Safety
/// Returns a pointer to static data with 'static lifetime.
/// Safe to call from C++. The returned pointer is never null.
///
/// # C++ Signature
/// ```cpp
/// extern "C" const ASCIIMaskArray* ASCIIMask_Mask0to9();
/// ```
#[no_mangle]
pub extern "C" fn ASCIIMask_Mask0to9() -> *const ASCIIMaskArray {
    &ZERO_TO_NINE_MASK as *const ASCIIMaskArray
}

// ============================================================================
// FFI Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_pointers_not_null() {
        assert!(!ASCIIMask_MaskWhitespace().is_null());
        assert!(!ASCIIMask_MaskCRLF().is_null());
        assert!(!ASCIIMask_MaskCRLFTab().is_null());
        assert!(!ASCIIMask_Mask0to9().is_null());
    }

    #[test]
    fn test_ffi_pointer_validity() {
        // Dereference pointers and check some values
        unsafe {
            let ws_mask = &*ASCIIMask_MaskWhitespace();
            assert!(ws_mask[b' ' as usize]);
            assert!(ws_mask[b'\n' as usize]);
            
            let crlf_mask = &*ASCIIMask_MaskCRLF();
            assert!(crlf_mask[b'\n' as usize]);
            assert!(crlf_mask[b'\r' as usize]);
            assert!(!crlf_mask[b' ' as usize]);
            
            let digit_mask = &*ASCIIMask_Mask0to9();
            assert!(digit_mask[b'0' as usize]);
            assert!(digit_mask[b'9' as usize]);
            assert!(!digit_mask[b'a' as usize]);
        }
    }

    #[test]
    fn test_ffi_pointers_stable() {
        // Calling multiple times should return the same pointer (static data)
        let p1 = ASCIIMask_MaskWhitespace();
        let p2 = ASCIIMask_MaskWhitespace();
        assert_eq!(p1, p2, "FFI should return same pointer to static data");
    }
}
