// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! FFI layer for nsCRT functions
//!
//! This module provides C-compatible exports for the nsCRT utility functions,
//! allowing C++ code to call the Rust implementations.
//!
//! # Safety
//!
//! All FFI functions use `catch_unwind` to prevent panics from unwinding into C++.

use std::panic;
use std::ptr;

/// FFI export for nsCRT::strtok
///
/// Thread-safe string tokenizer that modifies the input string in-place.
///
/// # Safety
///
/// - `string` must point to a valid, mutable, null-terminated C string
/// - `delims` must point to a valid, null-terminated C string
/// - `new_str` must point to a valid mutable pointer location
/// - The input string will be modified (delimiters replaced with '\0')
///
/// # C++ Usage:
///
/// ```cpp
/// char str[] = "a,b,c";
/// char* newStr;
/// char* token = nsCRT_strtok(str, ",", &newStr);
/// while (token != nullptr) {
///     // Use token
///     token = nsCRT_strtok(newStr, ",", &newStr);
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn nsCRT_strtok(
    string: *mut i8,
    delims: *const i8,
    new_str: *mut *mut i8,
) -> *mut i8 {
    panic::catch_unwind(|| {
        crate::strtok(string, delims, new_str)
    }).unwrap_or(ptr::null_mut())
}

/// FFI export for nsCRT::strcmp (char16_t* version)
///
/// Compares two null-terminated UTF-16 strings.
///
/// # Safety
///
/// - `str1` and `str2` must be null or point to valid null-terminated UTF-16 strings
/// - Returns: -1 if str1 < str2, 0 if equal, 1 if str1 > str2
///
/// # C++ Usage:
///
/// ```cpp
/// char16_t* s1 = u"hello";
/// char16_t* s2 = u"world";
/// int32_t result = nsCRT_strcmp_char16(s1, s2);
/// ```
#[no_mangle]
pub unsafe extern "C" fn nsCRT_strcmp_char16(
    str1: *const u16,
    str2: *const u16,
) -> i32 {
    panic::catch_unwind(|| {
        crate::strcmp_char16(str1, str2)
    }).unwrap_or(0)
}

/// FFI export for nsCRT::atoll
///
/// Converts a null-terminated C string to a 64-bit integer.
///
/// # Safety
///
/// - `str` must be null or point to a valid null-terminated C string
/// - Returns 0 if str is null or contains no digits
///
/// # C++ Usage:
///
/// ```cpp
/// char* str = "12345";
/// int64_t value = nsCRT_atoll(str);
/// ```
#[no_mangle]
pub unsafe extern "C" fn nsCRT_atoll(str: *const i8) -> i64 {
    panic::catch_unwind(|| {
        crate::atoll(str)
    }).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_ffi_strtok() {
        unsafe {
            let input = CString::new("x,y,z").unwrap().into_raw();
            let delims = CString::new(",").unwrap().into_raw();
            let mut new_str: *mut i8 = ptr::null_mut();

            let t1 = nsCRT_strtok(input, delims, &mut new_str);
            assert!(!t1.is_null());

            let t2 = nsCRT_strtok(new_str, delims, &mut new_str);
            assert!(!t2.is_null());

            let _ = CString::from_raw(input);
            let _ = CString::from_raw(delims);
        }
    }

    #[test]
    fn test_ffi_strcmp_char16() {
        unsafe {
            let s1: Vec<u16> = "test".encode_utf16().chain(std::iter::once(0)).collect();
            let s2: Vec<u16> = "test".encode_utf16().chain(std::iter::once(0)).collect();
            assert_eq!(nsCRT_strcmp_char16(s1.as_ptr(), s2.as_ptr()), 0);
        }
    }

    #[test]
    fn test_ffi_atoll() {
        unsafe {
            let s = CString::new("9999").unwrap();
            assert_eq!(nsCRT_atoll(s.as_ptr()), 9999);
        }
    }
}
